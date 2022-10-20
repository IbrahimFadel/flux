use flux_diagnostics::{Diagnostic, ToDiagnostic};
use flux_span::{FileId, FileSpanned, InFile, Span, Spanned};
use flux_syntax::ast;
use flux_syntax::{ast::AstNode, SyntaxToken};
use flux_typesystem as ts;
use la_arena::Arena;
use lasso::{Spur, ThreadedRodeo};
use text_size::TextRange;
use tinyvec::TinyVec;
use ts::{ConcreteKind, TChecker, TypeId, TypeKind};

use crate::hir::{Path, Type};
use crate::{diagnostics::LoweringDiagnostic, hir::Expr};

mod expr;
pub(crate) mod fn_decl;
mod stmt;
mod r#type;

static POISONED_STRING_VALUE: &str = "poisoned";

#[derive(Debug)]
pub(super) struct LoweringCtx {
    pub tchk: TChecker,
    pub exprs: Arena<Spanned<Expr>>,
    pub diagnostics: Vec<Diagnostic>,
    // pub function_signatures: Vec<(TinyVec<[TypeId; 2]>, TypeId)>,
    file_id: FileId,
    interner: &'static ThreadedRodeo,
}

impl LoweringCtx {
    pub fn new(file_id: FileId, interner: &'static ThreadedRodeo) -> Self {
        Self {
            tchk: TChecker::new(interner),
            exprs: Arena::new(),
            diagnostics: vec![],
            // function_signatures: vec![],
            file_id,
            interner,
        }
    }

    /// Lower an AST node to its HIR equivalent
    ///
    /// This exists to help clean up the lowering process due to the optional nature of the AST layer.
    /// We want certain nodes to **ALWAYS** be emitted even when there's a parsing error, but be marked as poisoned.
    /// For this reason, we can `unwrap`/`expect` safely (panics are ICEs), then carry on.
    ///
    /// If the node is poisoned, use the supplied closure to provide a poisoned value.
    /// If the node is not poisoned, use the supplied closure to carry out the regular lowering process.
    pub(crate) fn lower_node<N, T, P, F>(
        &mut self,
        node: Option<N>,
        poison_function: P,
        normal_function: F,
    ) -> T
    where
        N: AstNode,
        P: FnOnce(&mut Self, N) -> T,
        F: FnOnce(&mut Self, N) -> T,
    {
        let n = node.expect("internal compiler error: missing node that should always be emitted");
        if n.is_poisoned() {
            poison_function(self, n)
        } else {
            normal_function(self, n)
        }
    }

    fn unwrap_token(
        &mut self,
        tok: Option<&SyntaxToken>,
        msg: String,
        range: TextRange,
    ) -> Spanned<Spur> {
        match tok {
            Some(tok) => Spanned::new(tok.text_key(), Span::new(tok.text_range())),
            None => {
                self.emit_diagnostic(
                    LoweringDiagnostic::Missing {
                        msg: FileSpanned::new(
                            Spanned {
                                inner: msg,
                                span: Span::new(range),
                            },
                            self.file_id,
                        ),
                    }
                    .to_diagnostic(),
                );
                Spanned::new(
                    self.interner.get_or_intern(POISONED_STRING_VALUE),
                    Span::new(range),
                )
            }
        }
    }

    fn unwrap<T: AstNode>(&mut self, node: Option<T>, msg: String, span: Span) -> Option<T> {
        match node {
            Some(node) => Some(node),
            None => {
                self.emit_diagnostic(
                    LoweringDiagnostic::Missing {
                        msg: FileSpanned::new(Spanned { inner: msg, span }, self.file_id),
                    }
                    .to_diagnostic(),
                );
                None
            }
        }
    }

    fn file_spanned<T>(&self, spanned: Spanned<T>) -> FileSpanned<T> {
        FileSpanned::new(spanned, self.file_id)
    }

    fn file_span(&self, span: Span) -> InFile<Span> {
        InFile::new(span, self.file_id)
    }

    fn span_node<N: AstNode>(&self, node: &N) -> Span {
        Span::new(node.range())
    }

    fn maybe_emit_diagnostic<T>(&mut self, result: Result<T, Diagnostic>) {
        if let Err(err) = result {
            self.diagnostics.push(err);
        }
    }

    fn maybe_emit_diagnostic_with<T, P, F, B>(
        &mut self,
        result: Result<T, Diagnostic>,
        poison_function: P,
        normal_function: F,
    ) -> B
    where
        P: FnOnce(&mut Self) -> B,
        F: FnOnce(&mut Self, T) -> B,
    {
        match result {
            Ok(v) => normal_function(self, v),
            Err(err) => {
                self.diagnostics.push(err);
                poison_function(self)
            }
        }
    }

    fn emit_diagnostic(&mut self, diagnostic: Diagnostic) {
        self.diagnostics.push(diagnostic);
    }

    fn lower_path<'a>(&mut self, segments: impl Iterator<Item = &'a SyntaxToken>) -> Path {
        Path::from_syntax_tokens(segments)
    }

    pub(crate) fn to_ts_ty(&self, ty: &Spanned<Type>) -> Spanned<ts::Type> {
        ty.map_ref(|ty| {
            ts::Type::new(match ty {
                Type::Path(path) => TypeKind::Concrete(ConcreteKind::Path(path.get_spurs())),
                Type::Tuple(ids) => TypeKind::Concrete(ConcreteKind::Tuple(ids.clone())),
                Type::Error => TypeKind::Unknown,
            })
        })
    }
}
