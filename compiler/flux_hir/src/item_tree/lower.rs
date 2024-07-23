use flux_diagnostics::{ice, Diagnostic, ToDiagnostic};
use flux_span::{FileId, Interner, Span, Spanned, ToSpan, WithSpan, Word};
use flux_syntax::ast::{self, AstNode, Root};
use flux_typesystem::{self as ts, TEnv, TypeId, TypeKind};
use la_arena::Idx;
use ts::{FnSignature, ThisCtx, TraitId};

use crate::{
    body::LowerCtx,
    diagnostics::LowerError,
    hir::{
        ApplyDecl, AssociatedTypeDecl, AssociatedTypeDefinition, EnumDecl, EnumDeclVariant,
        EnumDeclVariantList, FnDecl, GenericParams, ModDecl, Param, ParamList, StructDecl,
        StructFieldDecl, StructFieldDeclList, TraitDecl, TypeBound, TypeBoundList, UseDecl,
        Visibility, WherePredicate,
    },
    item::{ItemId, ItemTreeIdx},
    module::ModuleId,
    pkg::PackageBodies,
};

use super::ItemTree;

pub struct Ctx<'a, 'pkgs> {
    pub(crate) body_ctx: crate::body::LowerCtx<'a, 'pkgs>,
    diagnostics: Vec<Diagnostic>,
    item_tree: &'a mut ItemTree,
}

impl<'a, 'pkgs> Ctx<'a, 'pkgs> {
    pub(crate) fn new(
        item_tree: &'a mut ItemTree,
        module_id: ModuleId,
        package_bodies: &'a mut PackageBodies,
        tenv: &'a mut TEnv,
        interner: &'static Interner,
        file_id: FileId,
    ) -> Self {
        Self {
            body_ctx: LowerCtx::new(
                None,
                None,
                None,
                module_id,
                &[],
                package_bodies,
                tenv,
                interner,
                file_id,
            ),
            diagnostics: vec![],
            item_tree,
        }
    }

    pub(super) fn lower_module_items(mut self, root: &Root) -> (Vec<ItemId>, Vec<Diagnostic>) {
        (
            root.items().map(|item| self.lower_item(&item)).collect(),
            self.diagnostics,
        )
    }

    fn lower_item(&mut self, item: &ast::Item) -> ItemId {
        let item_id = match item {
            ast::Item::ApplyDecl(apply_decl) => self.lower_apply_decl(apply_decl),
            ast::Item::EnumDecl(enum_decl) => self.lower_enum_decl(enum_decl),
            ast::Item::FnDecl(fn_decl) => self.lower_fn_decl(fn_decl),
            ast::Item::ModDecl(mod_decl) => self.lower_mod_decl(mod_decl),
            ast::Item::StructDecl(struct_decl) => self.lower_struct_decl(struct_decl),
            ast::Item::TraitDecl(trait_decl) => self.lower_trait_decl(trait_decl),
            ast::Item::UseDecl(use_decl) => self.lower_use_decl(use_decl),
        };

        self.item_tree.top_level.push(item_id.clone());
        item_id
    }

    fn lower_apply_decl(&mut self, apply_decl: &ast::ApplyDecl) -> ItemId {
        let visibility = self.lower_visibility(apply_decl.visibility());
        let generic_params =
            self.lower_generic_param_list(apply_decl.generic_param_list(), visibility.span);
        let trt = apply_decl
            .trt()
            .map(|trt| self.body_ctx.lower_path(trt.path(), &generic_params));
        let to_ty = self.lower_apply_to_ty(apply_decl.to_ty(), &generic_params);
        let assoc_types =
            self.lower_associated_type_definitions(apply_decl.associated_types(), &generic_params);

        let methods = self.lower_apply_methods(apply_decl.methods(), &generic_params);

        let apply = ApplyDecl::new(visibility, generic_params, trt, to_ty, assoc_types, methods);
        let apply_id = self.item_tree.applies.alloc(apply);
        ItemId::new(self.body_ctx.module_id, ItemTreeIdx::Apply(apply_id))
    }

    fn lower_enum_decl(&mut self, enum_decl: &ast::EnumDecl) -> ItemId {
        let visibility = self.lower_visibility(enum_decl.visibility());
        let name = self.body_ctx.lower_name(enum_decl.name());
        let mut generic_params =
            self.lower_generic_param_list(enum_decl.generic_param_list(), name.span);
        self.update_generic_params_with_where_clause(&mut generic_params, enum_decl.where_clause());
        let variants = self.lower_enum_decl_variants(&name, enum_decl.variants(), &generic_params);
        let enum_decl = EnumDecl::new(visibility, name, generic_params, variants);
        let enum_decl_id = self.item_tree.enums.alloc(enum_decl);
        ItemId::new(self.body_ctx.module_id, ItemTreeIdx::Enum(enum_decl_id))
    }

    fn lower_fn_decl(&mut self, function: &ast::FnDecl) -> ItemId {
        let visibility = self.lower_visibility(function.visibility());
        let name = self.body_ctx.lower_name(function.name());
        let mut generic_param_list =
            self.lower_generic_param_list(function.generic_param_list(), name.span);
        self.update_generic_params_with_where_clause(
            &mut generic_param_list,
            function.where_clause(),
        );
        let param_list = self.lower_param_list(function.param_list(), &generic_param_list);
        let ret_ty = self.lower_function_return_type(
            function.return_type(),
            &generic_param_list,
            param_list.span.end_span(),
        );
        let function = FnDecl::new(
            name,
            visibility,
            generic_param_list,
            param_list,
            ret_ty,
            Some(function.clone()),
        );

        let fn_id = self.item_tree.functions.alloc(function);
        ItemId::new(self.body_ctx.module_id, ItemTreeIdx::Function(fn_id))
    }

    fn lower_mod_decl(&mut self, mod_decl: &ast::ModDecl) -> ItemId {
        let visibility = self.lower_visibility(mod_decl.visibility());
        let name = self.body_ctx.lower_name(mod_decl.name());
        let mod_decl = ModDecl::new(visibility, name);
        let mod_decl_id = self.item_tree.mods.alloc(mod_decl);
        ItemId::new(self.body_ctx.module_id, ItemTreeIdx::Module(mod_decl_id))
    }

    fn lower_struct_decl(&mut self, struct_decl: &ast::StructDecl) -> ItemId {
        let visibility = self.lower_visibility(struct_decl.visibility());
        let name = self.body_ctx.lower_name(struct_decl.name());
        let mut generic_params =
            self.lower_generic_param_list(struct_decl.generic_param_list(), name.span);
        self.update_generic_params_with_where_clause(
            &mut generic_params,
            struct_decl.where_clause(),
        );
        let fields =
            self.lower_struct_field_decl_list(&name, struct_decl.field_list(), &generic_params);
        let struct_decl = StructDecl::new(visibility, name, generic_params, fields);
        let struct_decl_id = self.item_tree.structs.alloc(struct_decl);
        ItemId::new(self.body_ctx.module_id, ItemTreeIdx::Struct(struct_decl_id))
    }

    fn lower_trait_decl(&mut self, trait_decl: &ast::TraitDecl) -> ItemId {
        let visibility = self.lower_visibility(trait_decl.visibility());
        let name = self.body_ctx.lower_name(trait_decl.name());
        let mut generic_params =
            self.lower_generic_param_list(trait_decl.generic_param_list(), name.span);
        self.update_generic_params_with_where_clause(
            &mut generic_params,
            trait_decl.where_clause(),
        );
        let associated_types =
            self.lower_associated_type_decls(trait_decl.associated_types(), &generic_params);
        let methods = self.lower_trait_method_decls(trait_decl.method_decls(), &generic_params);
        let trait_decl =
            TraitDecl::new(visibility, name, generic_params, associated_types, methods);
        let trait_id = self.item_tree.traits.alloc(trait_decl);
        ItemId::new(self.body_ctx.module_id, ItemTreeIdx::Trait(trait_id))
    }

    fn lower_use_decl(&mut self, use_decl: &ast::UseDecl) -> ItemId {
        let path = self
            .body_ctx
            .lower_path(use_decl.path(), &GenericParams::empty());
        let alias = use_decl
            .alias()
            .map(|alias| self.body_ctx.lower_name(Some(alias)));
        let use_decl = UseDecl::new(path, alias);
        let use_decl_id = self.item_tree.uses.alloc(use_decl);
        ItemId::new(self.body_ctx.module_id, ItemTreeIdx::Use(use_decl_id))
    }

    fn lower_visibility(&mut self, visibility: Option<ast::Visibility>) -> Spanned<Visibility> {
        self.body_ctx.lower_node(
            visibility,
            |_, visibility| Visibility::Private.at(visibility.range().to_span()),
            |_, visibility| {
                visibility
                    .public()
                    .map_or(Visibility::Private, |_| Visibility::Public)
                    .at(visibility.range().to_span())
            },
        )
    }

    fn lower_generic_param_list(
        &mut self,
        generic_param_list: Option<ast::GenericParamList>,
        fallback_span: Span,
    ) -> Spanned<GenericParams> {
        self.body_ctx.lower_optional_node(
            generic_param_list,
            |_| GenericParams::default().at(fallback_span),
            |this, generic_params| {
                let mut hir_generic_param_list = GenericParams::default();
                generic_params.type_params().for_each(|param| {
                    let name = this.lower_name(param.name());
                    hir_generic_param_list.types.alloc(name);
                });
                hir_generic_param_list.at(generic_params.range().to_span())
            },
        )
    }

    fn update_generic_params_with_where_clause(
        &mut self,
        generic_params: &mut Spanned<GenericParams>,
        where_clause: Option<ast::WhereClause>,
    ) {
        self.body_ctx.lower_optional_node(
            where_clause,
            |_| (),
            |this, where_clause| {
                where_clause.predicates().for_each(|predicate| {
                    let name = this.lower_name(predicate.name());
                    let idx = generic_params
                        .types
                        .iter()
                        .find_map(|(idx, ty_name)| {
                            if ty_name.inner == name.inner {
                                Some(idx)
                            } else {
                                None
                            }
                        })
                        .unwrap_or_else(|| {
                            self.diagnostics.push(
                                LowerError::UnknownGeneric {
                                    name: this.interner.resolve(&name.inner).to_string(),
                                    name_file_span: name.span.in_file(this.file_id),
                                }
                                .to_diagnostic(),
                            );
                            generic_params.invalid_idx()
                        });

                    if let Some(type_bound_list) = predicate.type_bound_list() {
                        type_bound_list.type_bounds().for_each(|type_bound| {
                            let bound = this.lower_path(type_bound.trait_path(), &generic_params);
                            generic_params.inner.where_predicates.push(WherePredicate {
                                ty: idx,
                                name: name.inner,
                                bound: bound,
                            })
                        });
                    }
                });
            },
        );
    }

    fn lower_param_list(
        &mut self,
        param_list: Option<ast::ParamList>,
        generic_params: &GenericParams,
    ) -> Spanned<ParamList> {
        self.body_ctx.lower_node(
            param_list,
            |_, param_list| ParamList::poisoned().at(param_list.range().to_span()),
            |this, param_list| {
                let params = param_list
                    .params()
                    .map(|param| {
                        let name = this.lower_name(param.name());
                        let ty = this.lower_type(param.ty(), generic_params);
                        Param { name, ty }
                    })
                    .collect();

                ParamList::new(params).at(param_list.range().to_span())
            },
        )
    }

    fn lower_function_return_type(
        &mut self,
        return_type: Option<ast::FnReturnType>,
        generic_params: &GenericParams,
        fallback_span: Span,
    ) -> Spanned<TypeId> {
        self.body_ctx.lower_node(
            return_type,
            |this, _| this.insert_unit(fallback_span).at(fallback_span),
            |this, ret_ty| {
                let span = ret_ty.range().to_span();
                match ret_ty.ty() {
                    Some(ty) => this.lower_type(Some(ty), generic_params),
                    None => this.insert_unit(span).at(span),
                }
            },
        )
    }

    fn lower_associated_type_decls(
        &mut self,
        associated_type_decls: impl Iterator<Item = ast::TraitAssocTypeDecl>,
        generic_params: &GenericParams,
    ) -> Vec<AssociatedTypeDecl> {
        associated_type_decls
            .map(|associated_type_decl| {
                let name = self.body_ctx.lower_name(associated_type_decl.name());
                let type_bound_list = self
                    .lower_type_bound_list(associated_type_decl.type_bound_list(), generic_params);
                AssociatedTypeDecl::new(name, type_bound_list)
            })
            .collect()
    }

    fn lower_type_bound_list(
        &mut self,
        type_bound_list: Option<ast::TypeBoundList>,
        generic_params: &GenericParams,
    ) -> TypeBoundList {
        self.body_ctx.lower_optional_node(
            type_bound_list,
            |_| TypeBoundList::new(vec![]),
            |this, type_bound_list| {
                TypeBoundList::new(
                    type_bound_list
                        .type_bounds()
                        .map(|bound| {
                            this.lower_path(bound.trait_path(), &generic_params)
                                .map(|path| TypeBound::new(path))
                        })
                        .collect(),
                )
            },
        )
    }

    fn lower_trait_method_decls(
        &mut self,
        trait_method_decls: impl Iterator<Item = ast::TraitMethodDecl>,
        trait_generic_params: &Spanned<GenericParams>,
    ) -> Vec<Idx<FnDecl>> {
        trait_method_decls
            .map(|method_decl| {
                let name = self.body_ctx.lower_name(method_decl.name());
                let visibility = Visibility::Public.at(method_decl
                    .fn_kw()
                    .map_or(name.span, |fn_kw| fn_kw.text_range().to_span()));

                let method_generic_params =
                    self.lower_generic_param_list(method_decl.generic_param_list(), name.span);

                let span = method_generic_params.span;
                let mut generic_params = method_generic_params.map(|method_generic_params| {
                    method_generic_params
                        .union(trait_generic_params)
                        .unwrap_or_else(|(fallback_generic_params, duplicates)| {
                            let diagnostic = LowerError::DuplicateGenericParams {
                                generics_that_were_chilling: (),
                                generics_that_were_chilling_file_span: trait_generic_params
                                    .span
                                    .in_file(self.body_ctx.file_id),
                                generics_that_caused_duplication: duplicates
                                    .iter()
                                    .map(|key| self.body_ctx.interner.resolve(key).to_string())
                                    .collect(),
                                generics_that_caused_duplication_file_span: span
                                    .in_file(self.body_ctx.file_id),
                            }
                            .to_diagnostic();
                            self.diagnostics.push(diagnostic);
                            fallback_generic_params
                        })
                });
                self.update_generic_params_with_where_clause(
                    &mut generic_params,
                    method_decl.where_clause(),
                );

                let param_list = self.lower_param_list(method_decl.param_list(), &generic_params);
                let ret_ty = self.lower_function_return_type(
                    method_decl.return_ty(),
                    &generic_params,
                    param_list.span,
                );

                let fn_decl =
                    FnDecl::new(name, visibility, generic_params, param_list, ret_ty, None);
                self.item_tree.functions.alloc(fn_decl)
            })
            .collect()
    }

    fn lower_apply_to_ty(
        &mut self,
        to_ty: Option<ast::ApplyDeclType>,
        generic_params: &GenericParams,
    ) -> Spanned<TypeId> {
        self.body_ctx.lower_node(
            to_ty,
            |this, to_ty| {
                let span = to_ty.range().to_span();
                this.insert_unknown(span).at(span)
            },
            |this, to_ty| this.lower_type(to_ty.ty(), generic_params),
        )
    }

    fn lower_associated_type_definitions(
        &mut self,
        assoc_types: impl Iterator<Item = ast::ApplyDeclAssocType>,
        generic_params: &GenericParams,
    ) -> Vec<AssociatedTypeDefinition> {
        assoc_types
            .map(|assoc_type| {
                let name = self.body_ctx.lower_name(assoc_type.name());
                let ty = self.body_ctx.lower_type(assoc_type.ty(), generic_params);
                AssociatedTypeDefinition::new(name, ty)
            })
            .collect()
    }

    fn lower_apply_methods(
        &mut self,
        methods: impl Iterator<Item = ast::FnDecl>,
        apply_generic_params: &Spanned<GenericParams>,
    ) -> Vec<Idx<FnDecl>> {
        methods
            .map(|method| {
                let f: Idx<FnDecl> =
                    self.lower_fn_decl(&method)
                        .idx
                        .try_into()
                        .unwrap_or_else(|_| {
                            ice("lower_fn_decl returned ItemId not associated with function")
                        });
                let f_decl = &mut self.item_tree.functions[f];
                let span = f_decl.generic_params.span;
                let f_params = f_decl.generic_params.inner.clone();
                f_decl.generic_params = f_params
                    .union(apply_generic_params)
                    .map(|params| params.at(span))
                    .unwrap_or_else(|(fallback_generic_params, duplicates)| {
                        let diagnostic = LowerError::DuplicateGenericParams {
                            generics_that_were_chilling: (),
                            generics_that_were_chilling_file_span: apply_generic_params
                                .span
                                .in_file(self.body_ctx.file_id),
                            generics_that_caused_duplication: duplicates
                                .iter()
                                .map(|key| self.body_ctx.interner.resolve(key).to_string())
                                .collect(),
                            generics_that_caused_duplication_file_span: span
                                .in_file(self.body_ctx.file_id),
                        }
                        .to_diagnostic();
                        self.diagnostics.push(diagnostic);
                        fallback_generic_params.at(span)
                    });
                f
            })
            .collect()
    }

    fn lower_struct_field_decl_list(
        &mut self,
        struct_name: &Spanned<Word>,
        field_list: Option<ast::StructDeclFieldList>,
        generic_params: &Spanned<GenericParams>,
    ) -> StructFieldDeclList {
        let mut generic_params_used = Vec::with_capacity(generic_params.types.len());
        let field_decl_list = self.body_ctx.lower_node(
            field_list,
            |_, _| StructFieldDeclList::poisoned(),
            |this, field_list| {
                StructFieldDeclList::new(
                    field_list
                        .fields()
                        .map(|field| {
                            let name = this.lower_name(field.name());
                            let tid = this.lower_type(field.ty(), generic_params);
                            if let Some(generic_name) = this.tckh.tenv.generic_used(&tid) {
                                generic_params_used.push(generic_name);
                            }
                            StructFieldDecl::new(name, tid)
                        })
                        .collect(),
                )
            },
        );
        self.check_unused_generics(struct_name, generic_params, &generic_params_used);
        field_decl_list
    }

    fn lower_enum_decl_variants(
        &mut self,
        enum_name: &Spanned<Word>,
        variants: impl Iterator<Item = ast::EnumDeclVariant>,
        generic_params: &Spanned<GenericParams>,
    ) -> EnumDeclVariantList {
        let mut generic_params_used = Vec::with_capacity(generic_params.types.len());
        let variants = EnumDeclVariantList::new(
            variants
                .map(|variant| {
                    let name = self.body_ctx.lower_name(variant.name());
                    let ty = variant.ty().map(|ty| {
                        let tid = self.body_ctx.lower_type(Some(ty), generic_params);
                        if let Some(generic_name) = self.body_ctx.tckh.tenv.generic_used(&tid) {
                            generic_params_used.push(generic_name);
                        }
                        tid
                    });

                    EnumDeclVariant::new(name, ty)
                })
                .collect(),
        );
        self.check_unused_generics(enum_name, generic_params, &generic_params_used);
        variants
    }

    fn check_unused_generics(
        &mut self,
        item_name: &Spanned<Word>,
        generic_params: &Spanned<GenericParams>,
        generics_used: &[Word],
    ) {
        let unused_generics: Vec<_> = generic_params
            .types
            .values()
            .filter_map(|item| {
                if generics_used
                    .iter()
                    .find(|name| **name == item.inner)
                    .is_none()
                {
                    Some(self.body_ctx.interner.resolve(&item).to_string())
                } else {
                    None
                }
            })
            .collect();

        if !unused_generics.is_empty() {
            self.diagnostics.push(
                LowerError::UnusedGenerics {
                    item_name: self.body_ctx.interner.resolve(&item_name).to_string(),
                    item_name_file_span: item_name.span.in_file(self.body_ctx.file_id),
                    unused_generics: unused_generics,
                    unused_generics_file_span: generic_params.span.in_file(self.body_ctx.file_id),
                }
                .to_diagnostic(),
            );
        }
    }
}
