use flux_syntax::ast::ApplyDeclAssocType;

use crate::hir::{ApplyDecl, AssociatedTypeDef, FnDecl, GenericParamList, WhereClause};

use super::*;

impl LoweringCtx {
    pub(crate) fn lower_apply_decl(&mut self, apply_decl: ast::ApplyDecl) -> ApplyDecl {
        let generic_param_list = apply_decl.generic_param_list().map_or(
            GenericParamList::empty().at(apply_decl.range().to_span()),
            |generic_param_list| self.lower_generic_param_list(generic_param_list),
        );
        let trt = apply_decl.trt().map(|trt| {
            let path = self.lower_node(
                trt.path(),
                |_, _| Path::poisoned(),
                |this, trt| this.lower_path(trt.segments()),
            );
            let args = trt.generic_arg_list().map_or(vec![], |arg_list| {
                arg_list
                    .args()
                    .map(|arg| self.lower_type(arg, &generic_param_list))
                    .collect()
            });
            (path, args)
        });
        let ty = self.lower_node(
            apply_decl.to_ty(),
            |this, ty| this.types.alloc(Type::Error.at(ty.range().to_span())),
            |this, ty| this.lower_apply_decl_type(ty, &generic_param_list),
        );
        let where_clause = apply_decl
            .where_clause()
            .map_or(WhereClause::EMPTY, |where_clause| {
                self.lower_where_clause(where_clause, &generic_param_list)
            });
        let associated_types =
            self.lower_apply_decl_assoc_types(apply_decl.associated_types(), &generic_param_list);
        let methods = self.lower_apply_decl_methods(apply_decl.methods());
        ApplyDecl::new(trt, ty, where_clause, associated_types, methods)
    }

    fn lower_apply_decl_type(
        &mut self,
        ty: ast::ApplyDeclType,
        generic_param_list: &GenericParamList,
    ) -> TypeIdx {
        self.lower_node(
            ty.ty(),
            |this, _| this.types.alloc(Type::Error.at(ty.range().to_span())),
            |this, ty| this.lower_type(ty, generic_param_list),
        )
    }

    fn lower_apply_decl_assoc_types(
        &mut self,
        assoc_types: impl Iterator<Item = ApplyDeclAssocType>,
        generic_param_list: &GenericParamList,
    ) -> Vec<AssociatedTypeDef> {
        assoc_types
            .map(|ty| {
                let name = self.unwrap_token(
                    ty.name(),
                    "expected name in associated type definition",
                    ty.range(),
                );
                let ty = self.lower_node(
                    ty.ty(),
                    |this, ty| this.types.alloc(Type::Error.at(ty.range().to_span())),
                    |this, ty| this.lower_type(ty, generic_param_list),
                );
                AssociatedTypeDef::new(name, ty)
            })
            .collect()
    }

    fn lower_apply_decl_methods(
        &mut self,
        methods: impl Iterator<Item = ast::FnDecl>,
    ) -> Vec<FnDecl> {
        methods
            .map(|fn_decl| {
                let fn_decl_first_pass = self.lower_fn_signature(fn_decl.clone());
                self.lower_fn_decl(&fn_decl_first_pass)
            })
            .collect()
    }
}
