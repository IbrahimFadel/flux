use crate::hir::{GenericParamList, TypeBound, TypeBoundList, WhereClause, WherePredicate};

use super::*;

impl LoweringCtx {
    pub(crate) fn lower_generic_param_list(
        &mut self,
        generic_param_list: ast::GenericParamList,
    ) -> GenericParamList {
        let type_params = generic_param_list
            .type_params()
            .map(|type_param| {
                self.unwrap_token(
                    type_param.name(),
                    "expected type parameter to have a name",
                    type_param.range(),
                )
            })
            .collect();
        GenericParamList::new(type_params)
    }

    pub(crate) fn lower_where_clause(
        &mut self,
        where_clause: ast::WhereClause,
        generic_params_list: &GenericParamList,
    ) -> WhereClause {
        let predicates = where_clause
            .predicates()
            .map(|where_predicate| self.lower_where_predicate(where_predicate, generic_params_list))
            .collect();
        WhereClause::new(predicates)
    }

    fn lower_where_predicate(
        &mut self,
        where_predicate: ast::WherePredicate,
        generic_params_list: &GenericParamList,
    ) -> WherePredicate {
        let generic = self.unwrap_token(
            where_predicate.name(),
            "expected trait name in where clause predicate",
            where_predicate.range(),
        );
        let trait_restrictions = self.lower_node(
            where_predicate.type_bound_list(),
            |_, _| TypeBoundList::EMPTY,
            |this, type_bound_list| {
                this.lower_type_bound_list(type_bound_list, generic_params_list)
            },
        );
        WherePredicate::with_trait_restrictions(generic, trait_restrictions)
    }

    fn lower_type_bound_list(
        &mut self,
        type_bound_list: ast::TypeBoundList,
        generic_params_list: &GenericParamList,
    ) -> TypeBoundList {
        TypeBoundList::new(
            type_bound_list
                .type_bounds()
                .map(|type_bound| self.lower_type_bound(type_bound, generic_params_list))
                .collect(),
        )
    }

    fn lower_type_bound(
        &mut self,
        type_bound: ast::TypeBound,
        generic_params_list: &GenericParamList,
    ) -> TypeBound {
        let name = self.unwrap_token(
            type_bound.trait_name(),
            "expected trait name in type bound",
            type_bound.range(),
        );
        let generic_arg_list = type_bound
            .generic_arg_list()
            .map_or(vec![], |generic_arg_list| {
                self.lower_generic_arg_list(generic_arg_list, generic_params_list)
            });
        TypeBound::with_args(name, generic_arg_list)
    }

    pub(crate) fn lower_generic_arg_list(
        &mut self,
        generic_arg_list: ast::GenericArgList,
        generic_params_list: &GenericParamList,
    ) -> Vec<TypeIdx> {
        generic_arg_list
            .args()
            .map(|ty| self.lower_type(ty, generic_params_list))
            .collect()
    }
}
