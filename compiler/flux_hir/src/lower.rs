use flux_span::{FileId, FileSpanned, Span, Spanned};
use flux_syntax::ast;
use flux_syntax::{ast::AstNode, SyntaxToken};
use flux_typesystem as ts;
use flux_typesystem::TEnv;
use la_arena::Arena;
use lasso::{Spur, ThreadedRodeo};
use text_size::TextRange;
use ts::{ConcreteKind, TypeId, TypeKind};

use crate::hir::{Path, Type};
use crate::{diagnostics::LoweringDiagnostic, hir::Expr};

mod expr;
pub(crate) mod fn_decl;
mod r#type;

static POISONED_STRING_VALUE: &'static str = "poisoned";

pub(super) struct LoweringCtx {
    pub tenv: TEnv,
    exprs: Arena<Expr>,
    pub diagnostics: Vec<LoweringDiagnostic>,
    file_id: FileId,
    interner: &'static ThreadedRodeo,
}

impl LoweringCtx {
    pub fn new(file_id: FileId, interner: &'static ThreadedRodeo) -> Self {
        Self {
            tenv: TEnv::new(interner),
            exprs: Arena::new(),
            diagnostics: vec![],
            file_id,
            interner,
        }
    }

    /// Lower an AST node to its HIR equivalent
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
                self.emit_diagnostic(LoweringDiagnostic::Missing {
                    msg: FileSpanned::new(
                        Spanned {
                            inner: msg,
                            span: Span::new(range),
                        },
                        self.file_id,
                    ),
                });
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
                self.emit_diagnostic(LoweringDiagnostic::Missing {
                    msg: FileSpanned::new(Spanned { inner: msg, span }, self.file_id),
                });
                None
            }
        }
    }

    fn span_node<N: AstNode>(&self, node: &N) -> Span {
        Span::new(node.range())
    }

    fn emit_diagnostic(&mut self, diagnostic: LoweringDiagnostic) {
        self.diagnostics.push(diagnostic);
    }

    fn lower_path<'a>(&mut self, segments: impl Iterator<Item = &'a SyntaxToken>) -> Path {
        let segments = segments
            .map(|segment| Spanned::new(segment.text_key(), Span::new(segment.text_range())));
        Path::from_segments(segments)
    }

    fn to_ts_ty(&self, ty: &Type, span: Span) -> ts::Type {
        let ty_kind = match ty {
            Type::Path(path) => TypeKind::Concrete(ConcreteKind::Path(path.get_spurs())),
            Type::Error => TypeKind::Unknown,
        };
        ts::Type::new(ty_kind, span)
    }
}
