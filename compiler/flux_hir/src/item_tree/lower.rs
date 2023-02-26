use std::marker::PhantomData;

use flux_diagnostics::{ice, Diagnostic, ToDiagnostic};
use flux_span::{FileId, Span, ToSpan, WithSpan};
use flux_syntax::ast::{self, AstNode, Root};
use la_arena::Idx;
use lasso::ThreadedRodeo;

use crate::{
    diagnostics::LowerError,
    hir::{Function, GenericParams, Mod, Name, Param, Type, Visibility, WherePredicate},
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

pub(super) struct Ctx {
    tree: ItemTree,
    string_interner: &'static ThreadedRodeo,
    type_interner: &'static TypeInterner,
    body_ctx: crate::body::LowerCtx,
    diagnostics: Vec<Diagnostic>,
    file_id: FileId,
}

impl Ctx {
    pub fn new(
        file_id: FileId,
        string_interner: &'static ThreadedRodeo,
        type_interner: &'static TypeInterner,
    ) -> Self {
        Self {
            tree: ItemTree::default(),
            string_interner,
            type_interner,
            body_ctx: crate::body::LowerCtx::new(),
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
        }
    }

    fn lower_function(&mut self, function: &ast::FnDecl) -> FileItemTreeId<Function> {
        let visibility = self.lower_visibility(function.visibility());
        let name = self.lower_name(function.name());
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
        let visibility = self.lower_visibility(m.visibility());
        let name = self.lower_name(m.name());
        let res = Mod { visibility, name };
        id(self.tree.mods.alloc(res))
    }

    fn lower_visibility(&self, visibility: Option<ast::Visibility>) -> Visibility {
        lower_node(
            visibility,
            |_| Visibility::Private,
            |visibility| {
                visibility
                    .public()
                    .map_or(Visibility::Private, |_| Visibility::Public)
            },
        )
    }

    fn lower_name(&self, name: Option<ast::Name>) -> Name {
        lower_node(
            name,
            |name| {
                self.string_interner
                    .get_or_intern_static("poisoned_name")
                    .at(name.range().to_span())
            },
            |name| {
                let name = name
                    .ident()
                    .unwrap_or_else(|| ice("name parsed without identifier token"));
                name.text_key().at(name.text_range().to_span())
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
                    let name = self.lower_name(param.name());
                    generic_params.types.alloc(name.inner);
                });
                generic_params.at(ast_generic_params.range().to_span())
            }
            None => GenericParams::poisoned().at(fallback_span),
        };

        if let Some(where_clause) = where_clause {
            where_clause.predicates().for_each(|predicate| {
                let name = self.lower_name(predicate.name());
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
                                bound: path,
                            })
                        })
                    },
                );
            });
        }
        generic_params.inner
    }

    fn lower_params(&self, params: Option<ast::ParamList>) -> Vec<Param> {
        lower_node(
            params,
            |_| vec![],
            |param_list| {
                param_list
                    .params()
                    .map(|param| {
                        let name = self.lower_name(param.name());
                        let ty = self.lower_type(param.ty());
                        Param { name, ty }
                    })
                    .collect()
            },
        )
    }

    fn lower_return_type(&self, ty: Option<ast::Type>) -> TypeIdx {
        ty.map_or_else(
            || self.type_interner.intern(Type::Tuple(vec![])),
            |ty| self.lower_type(Some(ty)),
        )
    }

    fn lower_type(&self, ty: Option<ast::Type>) -> TypeIdx {
        self.type_interner.intern(lower_node(
            ty,
            |_| Type::Unknown,
            |ty| match ty {
                ast::Type::PathType(path) => Type::Path(self.body_ctx.lower_path(path.path())),
                ast::Type::TupleType(_) => todo!(),
                ast::Type::ArrayType(_) => todo!(),
                ast::Type::PtrType(_) => todo!(),
            },
        ))
    }
}

fn lower_node<N, T, P, F>(node: Option<N>, poison_function: P, normal_function: F) -> T
where
    N: AstNode,
    P: FnOnce(N) -> T,
    F: FnOnce(N) -> T,
{
    let n = node.unwrap_or_else(|| ice("missing node that should always be emitted"));
    if n.is_poisoned() {
        poison_function(n)
    } else {
        normal_function(n)
    }
}
