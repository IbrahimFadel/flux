use std::collections::HashSet;

use lasso::Spur;
use tinyvec::tiny_vec;
use ts::r#type::StructConcreteKind;

use crate::hir::{GenericParamList, StructDecl, StructField, StructFieldList, WhereClause};

use super::*;

impl LoweringCtx {
    pub(crate) fn lower_struct_decl(&mut self, struct_decl: ast::StructDecl) -> StructDecl {
        let span = struct_decl.range().to_span();
        let name = self.lower_node(
            struct_decl.name(),
            |this, _| {
                this.interner
                    .get_or_intern_static(POISONED_STRING_VALUE)
                    .at(span)
            },
            |this, name| name.ident().unwrap().text_key().at(span),
        );

        let generic_param_list = struct_decl.generic_param_list().map_or(
            GenericParamList::empty().at(name.span),
            |generic_param_list| self.lower_generic_param_list(generic_param_list),
        );

        let where_clause = struct_decl
            .where_clause()
            .map_or(WhereClause::EMPTY, |where_clause| {
                self.lower_where_clause(where_clause, &generic_param_list)
            });

        let mut generics_used_in_fields = HashSet::new();
        let field_list = self.lower_node(
            struct_decl.field_list(),
            |_, _| StructFieldList::empty().at(generic_param_list.span),
            |this, struct_field_list| {
                this.lower_struct_field_list(
                    struct_field_list,
                    &generic_param_list,
                    &mut generics_used_in_fields,
                )
            },
        );

        let unused_generic_params: Vec<String> = generic_param_list
            .iter()
            .map(|spur| spur.inner)
            .filter(|param| !generics_used_in_fields.contains(param))
            .map(|spur| self.interner.resolve(&spur).to_string())
            .collect();

        let generic_param_list_string: Vec<_> = generic_param_list
            .iter()
            .map(|name| self.interner.resolve(&name.inner).to_string())
            .collect();
        let generic_param_list_string =
            generic_param_list_string.in_file(self.file_id, generic_param_list.span);

        let unused_generic_params_len = unused_generic_params.len();
        if unused_generic_params_len > 0 {
            self.emit_diagnostic(
                LoweringDiagnostic::UnusedGenericParams {
                    declared_params: generic_param_list_string,
                    unused_params: unused_generic_params.in_file(self.file_id, field_list.span),
                }
                .to_diagnostic(),
            )
        }

        let field_types = field_list
            .iter()
            .map(|field| {
                let name = field.name.inner;
                let ty = self
                    .tchk
                    .tenv
                    .insert(self.to_ts_ty(field.ty).in_file(self.file_id));
                (name, ty)
            })
            .collect();
        let struct_ty_kind = TypeKind::Concrete(ConcreteKind::Struct(StructConcreteKind::new(
            generic_param_list
                .iter()
                .map(|name| {
                    self.tchk
                        .tenv
                        .insert(ts::Type::new(TypeKind::Generic).in_file(self.file_id, name.span))
                })
                .collect(),
            field_types,
        )));
        let struct_ty_id = self
            .tchk
            .tenv
            .insert(ts::Type::new(struct_ty_kind).in_file(self.file_id, span));
        self.tchk
            .tenv
            .insert_struct_type(std::iter::once(name.inner), struct_ty_id);
        StructDecl::new(name, where_clause, field_list)
    }

    fn lower_struct_field_list(
        &mut self,
        struct_field_list: ast::StructDeclFieldList,
        generic_param_list: &GenericParamList,
        generics_used_in_fields: &mut HashSet<Spur>,
    ) -> Spanned<StructFieldList> {
        Spanned::new(
            StructFieldList::new(
                struct_field_list
                    .fields()
                    .map(|field| {
                        self.lower_struct_field(field, generic_param_list, generics_used_in_fields)
                    })
                    .collect(),
            ),
            struct_field_list.range().to_span(),
        )
    }

    fn lower_struct_field(
        &mut self,
        struct_field: ast::StructDeclField,
        generic_param_list: &GenericParamList,
        generics_used_in_fields: &mut HashSet<Spur>,
    ) -> StructField {
        let span = struct_field.range().to_span();
        let name = self.lower_node(
            struct_field.name(),
            |this, _| {
                this.interner
                    .get_or_intern_static(POISONED_STRING_VALUE)
                    .at(span)
            },
            |this, name| name.ident().unwrap().text_key().at(span),
        );
        let ty = self.lower_node(
            struct_field.ty(),
            |this, _| this.types.alloc(Type::Tuple(tiny_vec!()).at(name.span)),
            |this, ty| {
                let ty = this.lower_type(ty, generic_param_list);
                if let Type::Generic(name) = &this.types[ty].inner {
                    generics_used_in_fields.insert(*name);
                }
                ty
            },
        );
        StructField::new(name, ty)
    }
}
