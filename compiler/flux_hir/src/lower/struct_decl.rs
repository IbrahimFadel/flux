use tinyvec::tiny_vec;
use ts::r#type::StructConcreteKind;

use crate::hir::{GenericParamList, StructDecl, StructField, StructFieldList, WhereClause};

use super::*;

impl LoweringCtx {
    pub(crate) fn lower_struct_decl(&mut self, struct_decl: ast::StructDecl) -> StructDecl {
        let name = self.lower_node(
            struct_decl.name(),
            |this, _| {
                Spanned::new(
                    this.interner.get_or_intern_static(POISONED_STRING_VALUE),
                    this.span_node(&struct_decl),
                )
            },
            |this, name| {
                Spanned::new(
                    name.ident().unwrap().text_key(),
                    this.span_node(&struct_decl),
                )
            },
        );

        let generic_param_list = struct_decl
            .generic_param_list()
            .map_or(GenericParamList::empty(), |generic_param_list| {
                self.lower_generic_param_list(generic_param_list)
            });

        let where_clause = struct_decl
            .where_clause()
            .map_or(WhereClause::EMPTY, |where_clause| {
                self.lower_where_clause(where_clause, &generic_param_list)
            });

        let field_list = self.lower_node(
            struct_decl.field_list(),
            |_, _| StructFieldList::empty(),
            |this, struct_field_list| {
                this.lower_struct_field_list(struct_field_list, &generic_param_list)
            },
        );

        let field_types = field_list
            .iter()
            .map(|field| {
                let name = field.name.inner;
                let ty = self
                    .tchk
                    .tenv
                    .insert(self.file_spanned(self.to_ts_ty(field.ty)));
                (name, ty)
            })
            .collect();
        let struct_ty_kind =
            TypeKind::Concrete(ConcreteKind::Struct(StructConcreteKind::new(
                generic_param_list
                    .iter()
                    .map(|name| {
                        self.tchk.tenv.insert(self.file_spanned(Spanned::new(
                            ts::Type::new(TypeKind::Generic),
                            name.span,
                        )))
                    })
                    .collect(),
                field_types,
            )));
        let struct_ty_id = self.tchk.tenv.insert(self.file_spanned(Spanned::new(
            ts::Type::new(struct_ty_kind),
            self.span_node(&struct_decl),
        )));
        self.tchk
            .tenv
            .insert_struct_type(std::iter::once(name.inner), struct_ty_id);
        StructDecl::new(name, generic_param_list, where_clause, field_list)
    }

    fn lower_struct_field_list(
        &mut self,
        struct_field_list: ast::StructDeclFieldList,
        generic_param_list: &GenericParamList,
    ) -> StructFieldList {
        StructFieldList::new(
            struct_field_list
                .fields()
                .map(|field| self.lower_struct_field(field, generic_param_list))
                .collect(),
        )
    }

    fn lower_struct_field(
        &mut self,
        struct_field: ast::StructDeclField,
        generic_param_list: &GenericParamList,
    ) -> StructField {
        let name = self.lower_node(
            struct_field.name(),
            |this, _| {
                Spanned::new(
                    this.interner.get_or_intern_static(POISONED_STRING_VALUE),
                    this.span_node(&struct_field),
                )
            },
            |this, name| {
                Spanned::new(
                    name.ident().unwrap().text_key(),
                    this.span_node(&struct_field),
                )
            },
        );
        let ty = if let Some(ty) = struct_field.ty() {
            self.lower_type(ty, generic_param_list)
        } else {
            self.types
                .alloc(Spanned::new(Type::Tuple(tiny_vec!()), name.span))
        };
        StructField::new(name, ty)
    }
}
