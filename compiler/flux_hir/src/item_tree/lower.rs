use std::marker::PhantomData;

use flux_diagnostics::{Diagnostic, ToDiagnostic};
use flux_span::{FileId, Span, Spanned, ToSpan, WithSpan};
use flux_syntax::ast::{self, AstNode, Root};
use la_arena::Idx;
use lasso::ThreadedRodeo;

use crate::{
    diagnostics::LowerError,
    hir::{
        Apply, Function, GenericParams, Mod, Name, Param, Params, Trait, Type, Use, Visibility,
        WherePredicate,
    },
    lower_node,
    type_interner::TypeIdx,
    FunctionId, TypeInterner,
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
            ast::Item::ApplyDecl(a) => self.lower_apply(a).into(),
            ast::Item::EnumDecl(_) => todo!(),
            ast::Item::FnDecl(function) => self.lower_function(function).into(),
            ast::Item::ModDecl(m) => self.lower_mod(m).into(),
            ast::Item::StructDecl(_) => todo!(),
            ast::Item::TraitDecl(t) => self.lower_trait(t).into(),
            ast::Item::UseDecl(u) => self.lower_use(u).into(),
        }
    }

    fn lower_apply(&mut self, apply: &ast::ApplyDecl) -> FileItemTreeId<Apply> {
        let visibility = self.lower_visibility(apply.visibility());
        let generic_params = self.lower_generic_params(
            apply.generic_param_list(),
            apply.where_clause(),
            visibility.span,
        );
        let trt = apply.trt().map(|trt| self.body_ctx.lower_path(trt.path()));
        let ty = lower_node(
            apply.to_ty(),
            |ty| {
                self.type_interner
                    .intern(Type::Unknown)
                    .at(ty.range().to_span())
            },
            |ty| self.body_ctx.lower_type(ty.ty()),
        );
        let assoc_types = self.lower_apply_assoc_types(apply.associated_types());
        let methods: Vec<FunctionId> = apply
            .methods()
            .map(|method| self.lower_function(&method).index)
            .collect();
        let res = Apply {
            visibility,
            generic_params,
            trt,
            ty,
            assoc_types,
            methods,
        };
        id(self.tree.applies.alloc(res))
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
            ast: Some(function.clone()),
        };
        id(self.tree.functions.alloc(res))
    }

    fn lower_mod(&mut self, m: &ast::ModDecl) -> FileItemTreeId<Mod> {
        let visibility = self.lower_visibility(m.visibility()).inner;
        let name = self.body_ctx.lower_name(m.name());
        let res = Mod { visibility, name };
        id(self.tree.mods.alloc(res))
    }

    fn lower_trait(&mut self, t: &ast::TraitDecl) -> FileItemTreeId<Trait> {
        let visibility = self.lower_visibility(t.visibility());
        let name = self.body_ctx.lower_name(t.name());
        let generic_params =
            self.lower_generic_params(t.generic_param_list(), t.where_clause(), name.span);
        let assoc_types = self.lower_trait_assoc_types(t.associated_types());
        let methods = self.lower_trait_methods(t.method_decls());
        let res = Trait {
            visibility,
            name,
            generic_params,
            assoc_types,
            methods,
        };
        id(self.tree.traits.alloc(res))
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

    fn lower_trait_assoc_types(
        &self,
        assoc_types: impl Iterator<Item = ast::TraitAssocTypeDecl>,
    ) -> Vec<Name> {
        assoc_types
            .map(|ty| self.body_ctx.lower_name(ty.name()))
            .collect()
    }

    fn lower_apply_assoc_types(
        &self,
        assoc_types: impl Iterator<Item = ast::ApplyDeclAssocType>,
    ) -> Vec<(Name, Spanned<TypeIdx>)> {
        assoc_types
            .map(|ty| {
                let name = self.body_ctx.lower_name(ty.name());
                let ty = self.body_ctx.lower_type(ty.ty());
                (name, ty)
            })
            .collect()
    }

    fn lower_trait_methods(
        &mut self,
        methods: impl Iterator<Item = ast::TraitMethodDecl>,
    ) -> Vec<FunctionId> {
        methods
            .map(|method| {
                let name = self.body_ctx.lower_name(method.name());
                let generic_params = self.lower_generic_params(
                    method.generic_param_list(),
                    method.where_clause(),
                    name.span,
                );
                let params = self.lower_params(method.param_list());
                let ret_ty = self.lower_return_type(method.return_ty());
                let f = Function {
                    visibility: Visibility::Public.at(name.span), // Arbitrary really... since theres no real concept of visibility for trait methods, but i guess they are public
                    name,
                    generic_params,
                    params,
                    ret_ty,
                    ast: None,
                };
                self.tree.functions.alloc(f)
            })
            .collect()
    }

    fn lower_params(&self, params: Option<ast::ParamList>) -> Spanned<Params> {
        lower_node(
            params,
            |param_list| Params::new(vec![]).at(param_list.range().to_span()),
            |param_list| {
                Params::new(
                    param_list
                        .params()
                        .map(|param| {
                            let name = self.body_ctx.lower_name(param.name());
                            let ty = self.body_ctx.lower_type(param.ty());
                            Param { name, ty }
                        })
                        .collect(),
                )
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
