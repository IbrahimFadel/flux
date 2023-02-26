use std::marker::PhantomData;

use flux_diagnostics::{Diagnostic, ToDiagnostic};
use flux_span::{FileId, Span, Spanned, ToSpan, WithSpan};
use flux_syntax::ast::{self, AstNode, Root};
use la_arena::Idx;
use lasso::ThreadedRodeo;

use crate::{
    diagnostics::LowerError,
    hir::{Function, GenericParams, Mod, Param, Type, Use, Visibility, WherePredicate},
    lower_node,
    type_interner::TypeIdx,
    TypeInterner,
};

use super::{FileItemTreeId, ItemTree, ItemTreeNode, ModItem};

fn id<N: ItemTreeNode>(index: Idx<N>) -> FileItemTreeId<N> {
    FileItemTreeId {
        index,
        _p: PhantomData,
    }
}

pub(super) struct Ctx<'a> {
    tree: ItemTree,
    string_interner: &'static ThreadedRodeo,
    type_interner: &'static TypeInterner,
    body_ctx: crate::body::LowerCtx<'a>,
    diagnostics: Vec<Diagnostic>,
    file_id: FileId,
}

impl<'a> Ctx<'a> {
    pub fn new(
        file_id: FileId,
        string_interner: &'static ThreadedRodeo,
        type_interner: &'static TypeInterner,
    ) -> Self {
        Self {
            tree: ItemTree::default(),
            string_interner,
            type_interner,
            body_ctx: crate::body::LowerCtx::new(string_interner, type_interner),
            diagnostics: Vec::new(),
            file_id,
        }
    }

    pub(super) fn lower_module_items(mut self, root: &Root) -> ItemTree {
        self.tree.top_level = root.items().map(|item| self.lower_item(&item)).collect();
        self.tree
    }

    fn lower_item(&mut self, item: &ast::Item) -> ModItem {
        match item {
            ast::Item::ApplyDecl(_) => todo!(),
            ast::Item::EnumDecl(_) => todo!(),
            ast::Item::FnDecl(function) => self.lower_function(function).into(),
            ast::Item::ModDecl(m) => self.lower_mod(m).into(),
            ast::Item::StructDecl(_) => todo!(),
            ast::Item::TraitDecl(_) => todo!(),
            ast::Item::UseDecl(u) => self.lower_use(u).into(),
        }
    }

    fn lower_function(&mut self, function: &ast::FnDecl) -> FileItemTreeId<Function> {
        let visibility = self.lower_visibility(function.visibility());
        let name = self.body_ctx.lower_name(function.name());
        let generic_params = self.lower_generic_params(
            function.generic_param_list(),
            function.where_clause(),
            name.span,
        );
        let params = self.lower_params(function.param_list());
        let ret_ty = self.lower_return_type(function.return_type());
        let res = Function {
            visibility,
            name,
            generic_params,
            params,
            ret_ty,
            ast: function.clone(),
        };
        id(self.tree.functions.alloc(res))
    }

    fn lower_mod(&mut self, m: &ast::ModDecl) -> FileItemTreeId<Mod> {
        let visibility = self.lower_visibility(m.visibility()).inner;
        let name = self.body_ctx.lower_name(m.name());
        let res = Mod { visibility, name };
        id(self.tree.mods.alloc(res))
    }

    fn lower_use(&mut self, u: &ast::UseDecl) -> FileItemTreeId<Use> {
        let visibility = self.lower_visibility(u.visibility()).inner;
        let path = self.body_ctx.lower_path(u.path());
        let alias = u.alias().map(|alias| self.body_ctx.lower_name(Some(alias)));
        let res = Use {
            visibility,
            path,
            alias,
        };
        id(self.tree.uses.alloc(res))
    }

    fn lower_visibility(&self, visibility: Option<ast::Visibility>) -> Spanned<Visibility> {
        lower_node(
            visibility,
            |visibility| Visibility::Private.at(visibility.range().to_span()),
            |visibility| {
                visibility
                    .public()
                    .map_or(Visibility::Private, |_| Visibility::Public)
                    .at(visibility.range().to_span())
            },
        )
    }

    fn lower_generic_params(
        &mut self,
        generic_params: Option<ast::GenericParamList>,
        where_clause: Option<ast::WhereClause>,
        fallback_span: Span,
    ) -> GenericParams {
        let mut generic_params = match generic_params {
            Some(ast_generic_params) => {
                let mut generic_params = GenericParams::default();
                ast_generic_params.type_params().for_each(|param| {
                    let name = self.body_ctx.lower_name(param.name());
                    generic_params.types.alloc(name.inner);
                });
                generic_params.at(ast_generic_params.range().to_span())
            }
            None => GenericParams::poisoned().at(fallback_span),
        };

        if let Some(where_clause) = where_clause {
            where_clause.predicates().for_each(|predicate| {
                let name = self.body_ctx.lower_name(predicate.name());
                let idx = generic_params
                    .types
                    .iter()
                    .find(|(_, n)| **n == name.inner)
                    .map(|(idx, _)| idx)
                    .unwrap_or_else(|| {
                        self.diagnostics.push(
                            LowerError::UnknownGenericInWherePredicate {
                                generic: name
                                    .map_ref(|spur| self.string_interner.resolve(spur).to_string())
                                    .in_file(self.file_id),
                                generic_params: generic_params.span.in_file(self.file_id),
                            }
                            .to_diagnostic(),
                        );
                        generic_params.invalid_idx()
                    });
                lower_node(
                    predicate.type_bound_list(),
                    |_| todo!(),
                    |type_bound_list| {
                        type_bound_list.type_bounds().for_each(|bound| {
                            let path = self.body_ctx.lower_path(bound.trait_path());
                            generic_params.inner.where_predicates.push(WherePredicate {
                                ty: idx,
                                bound: path.inner,
                            })
                        })
                    },
                );
            });
        }
        generic_params.inner
    }

    fn lower_params(&self, params: Option<ast::ParamList>) -> Spanned<Vec<Param>> {
        lower_node(
            params,
            |param_list| vec![].at(param_list.range().to_span()),
            |param_list| {
                param_list
                    .params()
                    .map(|param| {
                        let name = self.body_ctx.lower_name(param.name());
                        let ty = self.body_ctx.lower_type(param.ty());
                        Param { name, ty }
                    })
                    .collect::<Vec<_>>()
                    .at(param_list.range().to_span())
            },
        )
    }

    fn lower_return_type(&self, ty: Option<ast::FnReturnType>) -> Spanned<TypeIdx> {
        lower_node(
            ty,
            |ty| {
                self.type_interner
                    .intern(Type::Tuple(vec![]))
                    .at(ty.range().to_span())
            },
            |ty| match ty.ty() {
                Some(ty) => self.body_ctx.lower_type(Some(ty)),
                None => self
                    .type_interner
                    .intern(Type::Tuple(vec![]))
                    .at(ty.range().to_span()),
            },
        )
    }
}
