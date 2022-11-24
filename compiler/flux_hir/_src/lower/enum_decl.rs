use crate::hir::{EnumDecl, EnumVariant, GenericParamList, WhereClause};

use super::*;

impl LoweringCtx {
    pub(crate) fn lower_enum_decl(&mut self, enum_decl: ast::EnumDecl) -> EnumDecl {
        let name = self.lower_node(
            enum_decl.name(),
            |this, _| {
                this.interner
                    .get_or_intern_static(POISONED_STRING_VALUE)
                    .at(enum_decl.range().to_span())
            },
            |_, name| {
                name.ident()
                    .unwrap()
                    .text_key()
                    .at(enum_decl.range().to_span())
            },
        );
        let generic_param_list = enum_decl.generic_param_list().map_or(
            GenericParamList::empty().at(name.span),
            |generic_param_list| self.lower_generic_param_list(generic_param_list),
        );
        let where_clause = enum_decl
            .where_clause()
            .map_or(WhereClause::EMPTY, |where_clause| {
                self.lower_where_clause(where_clause, &generic_param_list)
            });
        let variants = self.lower_enum_variants(enum_decl.variants(), &generic_param_list);
        EnumDecl::new(name, where_clause, variants)
    }

    fn lower_enum_variants(
        &mut self,
        variants: impl Iterator<Item = ast::EnumDeclVariant>,
        generic_param_list: &GenericParamList,
    ) -> Vec<EnumVariant> {
        variants
            .map(|variant| {
                let name = self.unwrap_token(
                    variant.name(),
                    "expected enum variant name",
                    variant.range(),
                );
                let ty = variant
                    .ty()
                    .map(|ty| self.lower_type(ty, generic_param_list));
                EnumVariant::new(name, ty)
            })
            .collect()
    }
}
