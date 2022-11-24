use tinyvec::tiny_vec;

use crate::hir::{
    AssociatedType, GenericParamList, ParamList, TraitDecl, TraitMethod, WhereClause,
};

use super::*;

impl LoweringCtx {
    pub(crate) fn lower_trait_decl(&mut self, trait_decl: ast::TraitDecl) -> TraitDecl {
        let span = trait_decl.range().to_span();
        let name = self.lower_node(
            trait_decl.name(),
            |this, _| {
                this.interner.get_or_intern_static(POISONED_STRING_VALUE).at(span)
            },
            |this, name| {
                name.ident().unwrap().text_key().at(span)
            },
        );
        let generic_param_list = trait_decl.generic_param_list().map_or(
            GenericParamList::empty().at(name.span),
            |generic_param_list| self.lower_generic_param_list(generic_param_list),
        );
        let where_clause = trait_decl
            .where_clause()
            .map_or(WhereClause::EMPTY, |where_clause| {
                self.lower_where_clause(where_clause, &generic_param_list)
            });
        let associated_types = self.lower_associated_types(trait_decl.associated_types());
        let methods = self.lower_trait_method_decls(trait_decl.method_decls());
        TraitDecl::new(name, where_clause, associated_types, methods)
    }

    fn lower_associated_types(
        &mut self,
        associated_types: impl Iterator<Item = ast::TraitAssocTypeDecl>,
    ) -> Vec<AssociatedType> {
        associated_types
            .map(|associated_type| {
                self.unwrap_token(
                    associated_type.name(),
                    "expected name in associated type declaration",
                    associated_type.range(),
                )
            })
            .collect()
    }

    fn lower_trait_method_decls(
        &mut self,
        trait_method_decls: impl Iterator<Item = ast::TraitMethodDecl>,
    ) -> Vec<TraitMethod> {
        trait_method_decls
            .map(|trait_method_decl| self.lower_trait_method_decl(trait_method_decl))
            .collect()
    }

    fn lower_trait_method_decl(&mut self, trait_method_decl: ast::TraitMethodDecl) -> TraitMethod {
        let span = trait_method_decl.range().to_span();
        let name = self.lower_node(
            trait_method_decl.name(),
            |this, _| {
                this.interner.get_or_intern_static(POISONED_STRING_VALUE).at(span)
            },
            |this, name| {
                name.ident().unwrap().text_key().at(span)
            },
        );
        let generic_param_list = trait_method_decl.generic_param_list().map_or(
            GenericParamList::empty().at(name.span),
            |generic_param_list| self.lower_generic_param_list(generic_param_list),
        );
        let param_list = self.lower_node(
            trait_method_decl.param_list(),
            |_, _| ParamList::new(vec![]).at(name.span),
            |this, param_list| this.lower_param_list(param_list, &generic_param_list),
        );
        let where_clause = trait_method_decl
            .where_clause()
            .map_or(WhereClause::EMPTY, |where_clause| {
                self.lower_where_clause(where_clause, &generic_param_list)
            });
        let return_ty = if let Some(return_ty) = trait_method_decl.return_ty() {
            self.lower_type(return_ty, &generic_param_list)
        } else {
            self.types
                .alloc(Type::Tuple(tiny_vec!()).at(param_list.span))
        };
        TraitMethod::new(name, param_list, return_ty, where_clause)
    }
}
