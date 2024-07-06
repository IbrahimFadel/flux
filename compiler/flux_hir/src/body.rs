use flux_diagnostics::{ice, Diagnostic};
use flux_span::{FileId, Interner, Spanned, ToSpan, WithSpan, Word};
use flux_syntax::{
    ast::{self, AstNode},
    SyntaxToken,
};
use flux_typesystem::{Insert, TChecker, TEnv, TypeId, Typed, WithType};
use la_arena::{Arena, Idx};

use crate::{
    hir::{ApplyDecl, Expr, ExprIdx, FnDecl, GenericParams, Op, Path, Type},
    item::ItemTreeIdx,
    item_tree::ItemTree,
    module::ModuleId,
    name_res::item::ItemResolver,
    POISONED_NAME,
};

mod expr;
mod stmt;
mod r#type;

pub(crate) struct LowerCtx<'a> {
    diagnostics: Vec<Diagnostic>,
    item_resolver: ItemResolver<'a>,
    pub interner: &'static Interner,
    pub tckh: TChecker<'a>,
    pub(crate) item_tree: Option<&'a ItemTree>,
    exprs: Arena<Expr>,
    pub(super) file_id: FileId,
    pub(crate) module_id: ModuleId,
}

impl<'a> LowerCtx<'a> {
    pub(crate) fn new(
        item_resolver: ItemResolver<'a>,
        item_tree: Option<&'a ItemTree>,
        tenv: &'a mut TEnv,
        interner: &'static Interner,
        file_id: FileId,
        module_id: ModuleId,
    ) -> Self {
        Self {
            diagnostics: vec![],
            item_resolver,
            interner,
            item_tree,
            tckh: TChecker::new(tenv),
            exprs: Arena::new(),
            file_id,
            module_id,
        }
    }

    pub(crate) fn lower_module_bodies(mut self) -> Vec<Diagnostic> {
        debug_assert!(self.item_tree.is_some());
        let this_mod_id = self.module_id;
        let item_tree = self.item_tree.as_ref().unwrap();
        for item in item_tree
            .top_level
            .iter()
            .filter(|item| item.mod_id == this_mod_id)
        {
            match &item.idx {
                ItemTreeIdx::Apply(apply_idx) => self.handle_apply_decl(*apply_idx),
                ItemTreeIdx::Function(fn_idx) => {
                    self.lower_function_body(*fn_idx);
                }
                _ => {}
            }
        }
        self.diagnostics
    }

    fn handle_apply_decl(&mut self, apply_idx: Idx<ApplyDecl>) {
        let apply_decl = &self.item_tree.as_ref().unwrap().applies[apply_idx];
        self.tckh.tenv.insert_local(
            self.interner.get_or_intern_static("this"),
            apply_decl.to_ty.inner,
        );
        apply_decl.methods.iter().for_each(|method| {
            self.lower_function_body(*method);
        });
    }

    fn lower_function_body(&mut self, fn_idx: Idx<FnDecl>) {
        let fn_decl = &self.item_tree.as_ref().unwrap().functions[fn_idx];
        if let Some(ast) = &fn_decl.ast {
            fn_decl.params.iter().for_each(|param| {
                self.tckh
                    .tenv
                    .insert_local(param.name.inner, param.ty.inner);
            });

            let body = self.lower_expr(ast.body(), &fn_decl.generic_params);
            self.tckh
                .unify(
                    fn_decl.return_ty.inner,
                    body.tid,
                    self.tckh.tenv.get_filespan(&body.tid),
                )
                .unwrap_or_else(|err| self.diagnostics.push(err));
        }
    }

    pub fn lower_node<N, T, P, F>(
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
        let n = node.unwrap_or_else(|| ice("missing node that should always be emitted"));
        if n.is_poisoned() {
            poison_function(self, n)
        } else {
            normal_function(self, n)
        }
    }

    pub fn lower_optional_node<N, T, P, F>(
        &mut self,
        node: Option<N>,
        poison_function: P,
        normal_function: F,
    ) -> T
    where
        N: AstNode,
        P: FnOnce(&mut Self) -> T,
        F: FnOnce(&mut Self, N) -> T,
    {
        match node {
            Some(n) => {
                if n.is_poisoned() {
                    poison_function(self)
                } else {
                    normal_function(self, n)
                }
            }
            None => poison_function(self),
        }
    }

    pub(crate) fn lower_name(&mut self, name: Option<ast::Name>) -> Spanned<Word> {
        self.lower_node(
            name,
            |this, name| {
                this.interner
                    .get_or_intern_static(POISONED_NAME)
                    .at(name.range().to_span())
            },
            |_, name| {
                let name = name
                    .ident()
                    .unwrap_or_else(|| ice("name parsed without identifier token"));
                let key = name.text_key().unwrap_or_else(|| ice("parsed empty name"));
                key.at(name.text_range().to_span())
            },
        )
    }

    pub(crate) fn lower_path(
        &mut self,
        path: Option<ast::Path>,
        generic_params: &GenericParams,
    ) -> Spanned<Path> {
        self.lower_node(
            path,
            |_, path| Path::poisoned().at(path.range().to_span()),
            |this, path| {
                let segments = path
                    .segments()
                    .map(|segment| {
                        segment
                            .text_key()
                            .unwrap_or_else(|| ice("text key contained no text"))
                    })
                    .collect();
                let generic_args = path
                    .generic_arg_list()
                    .map(|arg_list| {
                        arg_list
                            .args()
                            .map(|arg| this.lower_type(Some(arg), generic_params).inner)
                            .collect()
                    })
                    .unwrap_or(vec![]);
                Path::new(segments, generic_args).at(path.range().to_span())
            },
        )
    }

    fn lower_op(&mut self, op: Option<&SyntaxToken>) -> Spanned<Op> {
        use flux_syntax::SyntaxKind::*;
        let op = op.unwrap_or_else(|| ice("there should always be an op token"));
        match op.kind() {
            Eq => Op::Eq,
            Plus => Op::Add,
            Minus => Op::Sub,
            Star => Op::Mul,
            Slash => Op::Div,
            CmpAnd => Op::CmpAnd,
            CmpEq => Op::CmpEq,
            CmpGt => Op::CmpGt,
            CmpGte => Op::CmpGte,
            CmpLt => Op::CmpLt,
            CmpLte => Op::CmpLte,
            CmpNeq => Op::CmpNeq,
            CmpOr => Op::CmpOr,
            _ => ice("invalid op token encountered"),
        }
        .at(op.text_range().to_span())
    }
}
