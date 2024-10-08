use std::collections::HashSet;

use flux_diagnostics::{ice, Diagnostic, ToDiagnostic};
use flux_id::{
    id::{self, WithMod},
    Map,
};
use flux_parser::ast::{self, AstNode};
use flux_typesystem::{ThisCtx, Type};
use flux_util::{FileId, Interner, Span, Spanned, ToSpan, WithSpan, Word};

use crate::{
    def::{
        item::{ApplyDecl, EnumDecl, FnDecl, ModDecl, StructDecl, TraitDecl, UseDecl, Visibility},
        AssociatedTypeDecl, AssociatedTypeDefinition, EnumDeclVariant, EnumDeclVariantList,
        GenericParams, Param, ParamList, StructFieldDecl, StructFieldDeclList, TypeBound,
        TypeBoundList, WherePredicate,
    },
    diagnostics::LowerError,
    item::{ItemId, ItemTreeIdx},
};

use super::{lower_node, lower_optional_node_mut, r#type};

#[derive(Debug, Default, Clone)]
pub(crate) struct ItemTree {
    pub top_level: Vec<ItemId>,
    pub applies: Map<id::ApplyDecl, ApplyDecl>,
    pub enums: Map<id::EnumDecl, EnumDecl>,
    pub functions: Map<id::FnDecl, FnDecl>,
    pub mods: Map<id::ModDecl, ModDecl>,
    pub structs: Map<id::StructDecl, StructDecl>,
    pub traits: Map<id::TraitDecl, TraitDecl>,
    pub uses: Map<id::UseDecl, UseDecl>,
}

impl ItemTree {
    pub fn new() -> Self {
        Self {
            top_level: vec![],
            applies: Map::new(),
            enums: Map::new(),
            functions: Map::new(),
            mods: Map::new(),
            structs: Map::new(),
            traits: Map::new(),
            uses: Map::new(),
        }
    }
}

pub(super) struct LoweringCtx<'a> {
    item_tree: &'a mut ItemTree,
    type_lowerer: r#type::LoweringCtx,
    file_id: FileId,
    module_id: id::Mod,
    interner: &'static Interner,
    diagnostics: &'a mut Vec<Diagnostic>,
}

impl<'a> LoweringCtx<'a> {
    pub(super) fn new(
        file_id: FileId,
        module_id: id::Mod,
        item_tree: &'a mut ItemTree,
        interner: &'static Interner,
        diagnostics: &'a mut Vec<Diagnostic>,
    ) -> Self {
        Self {
            item_tree,
            type_lowerer: r#type::LoweringCtx::new(ThisCtx::Function, interner),
            file_id,
            module_id,
            interner,
            diagnostics,
        }
    }

    pub(super) fn lower_module_items(mut self, root: &ast::Root) -> Vec<ItemId> {
        root.items().map(|item| self.lower_item(&item)).collect()
    }

    fn lower_item(&mut self, item: &ast::Item) -> ItemId {
        let item_id = match item {
            ast::Item::ApplyDecl(apply_decl) => self.lower_apply_decl(apply_decl),
            ast::Item::EnumDecl(enum_decl) => self.lower_enum_decl(enum_decl),
            ast::Item::FnDecl(fn_decl) => self.lower_fn_decl(fn_decl, None),
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

        let to_ty = self.lower_apply_to_ty(apply_decl.to_ty(), &generic_params);
        let trt = apply_decl
            .trt()
            .map(|trt| self.type_lowerer.lower_path(trt.path(), &generic_params));

        let this_ctx = match trt {
            Some(_) => ThisCtx::TraitApplication(Box::new(to_ty.kind.clone()), vec![]),
            None => ThisCtx::TypeApplication(Box::new(to_ty.kind.clone())),
        };
        self.type_lowerer.set_this_ctx(this_ctx);

        let assoc_types =
            self.lower_associated_type_definitions(apply_decl.associated_types(), &generic_params);

        match trt {
            Some(_) => self.type_lowerer.set_associated_types(
                assoc_types
                    .iter()
                    .map(|assoc_ty| (assoc_ty.name.inner, assoc_ty.ty.kind.clone()))
                    .collect(),
            ),
            None => {}
        };

        let methods = self.lower_apply_methods(apply_decl.methods(), &generic_params);

        let apply = ApplyDecl::new(visibility, generic_params, trt, to_ty, assoc_types, methods);
        let apply_id = self.item_tree.applies.insert(apply);
        ItemTreeIdx::Apply(apply_id).in_mod(self.module_id).into()
    }

    fn lower_enum_decl(&mut self, enum_decl: &ast::EnumDecl) -> ItemId {
        let visibility = self.lower_visibility(enum_decl.visibility());
        let name = self.type_lowerer.lower_name(enum_decl.name());
        let mut generic_params =
            self.lower_generic_param_list(enum_decl.generic_param_list(), name.span);
        self.update_generic_params_with_where_clause(&mut generic_params, enum_decl.where_clause());
        let variants = self.lower_enum_decl_variants(&name, enum_decl.variants(), &generic_params);
        let enum_decl = EnumDecl::new(visibility, name, generic_params, variants);
        let enum_decl_id = self.item_tree.enums.insert(enum_decl);
        ItemTreeIdx::Enum(enum_decl_id)
            .in_mod(self.module_id)
            .into()
    }

    fn lower_fn_decl(
        &mut self,
        function: &ast::FnDecl,
        apply_generic_params: Option<&Spanned<GenericParams>>,
    ) -> ItemId {
        let visibility = self.lower_visibility(function.visibility());
        let name = self.type_lowerer.lower_name(function.name());
        let mut generic_param_list =
            self.lower_generic_param_list(function.generic_param_list(), name.span);
        self.update_generic_params_with_where_clause(
            &mut generic_param_list,
            function.where_clause(),
        );
        let generic_params_span = generic_param_list.span;
        if let Some(apply_generic_params) = apply_generic_params {
            generic_param_list = generic_param_list.map(|generic_param_list| {
                generic_param_list
                    .union(apply_generic_params)
                    .unwrap_or_else(|(fallback_generic_params, duplicates)| {
                        let diagnostic = LowerError::DuplicateGenericParams {
                            generics_that_were_chilling: (),
                            generics_that_were_chilling_file_span: apply_generic_params
                                .span
                                .in_file(self.file_id),
                            generics_that_caused_duplication: duplicates
                                .iter()
                                .map(|key| self.interner.resolve(key).to_string())
                                .collect(),
                            generics_that_caused_duplication_file_span: generic_params_span
                                .in_file(self.file_id),
                        }
                        .to_diagnostic();
                        self.diagnostics.push(diagnostic);
                        fallback_generic_params
                    })
            });
        }

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

        let fn_id = self.item_tree.functions.insert(function);
        ItemTreeIdx::Function(fn_id).in_mod(self.module_id).into()
    }

    fn lower_mod_decl(&mut self, mod_decl: &ast::ModDecl) -> ItemId {
        let visibility = self.lower_visibility(mod_decl.visibility());
        let name = self.type_lowerer.lower_name(mod_decl.name());
        let mod_decl = ModDecl::new(visibility, name);
        let mod_decl_id = self.item_tree.mods.insert(mod_decl);
        ItemTreeIdx::Module(mod_decl_id)
            .in_mod(self.module_id)
            .into()
    }

    fn lower_struct_decl(&mut self, struct_decl: &ast::StructDecl) -> ItemId {
        let visibility = self.lower_visibility(struct_decl.visibility());
        let name = self.type_lowerer.lower_name(struct_decl.name());
        let mut generic_params =
            self.lower_generic_param_list(struct_decl.generic_param_list(), name.span);
        self.update_generic_params_with_where_clause(
            &mut generic_params,
            struct_decl.where_clause(),
        );
        let fields =
            self.lower_struct_field_decl_list(&name, struct_decl.field_list(), &generic_params);
        let struct_decl = StructDecl::new(visibility, name, generic_params, fields);
        let struct_decl_id = self.item_tree.structs.insert(struct_decl);
        ItemTreeIdx::Struct(struct_decl_id)
            .in_mod(self.module_id)
            .into()
    }

    fn lower_trait_decl(&mut self, trait_decl: &ast::TraitDecl) -> ItemId {
        let visibility = self.lower_visibility(trait_decl.visibility());
        let name = self.type_lowerer.lower_name(trait_decl.name());
        let mut generic_params =
            self.lower_generic_param_list(trait_decl.generic_param_list(), name.span);
        self.update_generic_params_with_where_clause(
            &mut generic_params,
            trait_decl.where_clause(),
        );

        self.type_lowerer.set_this_ctx(ThisCtx::TraitDecl);

        let associated_types =
            self.lower_associated_type_decls(trait_decl.associated_types(), &generic_params);
        let methods = self.lower_trait_method_decls(trait_decl.method_decls(), &generic_params);
        let trait_decl =
            TraitDecl::new(visibility, name, generic_params, associated_types, methods);
        let trait_id = self.item_tree.traits.insert(trait_decl);
        ItemTreeIdx::Trait(trait_id).in_mod(self.module_id).into()
    }

    fn lower_use_decl(&mut self, use_decl: &ast::UseDecl) -> ItemId {
        let path = self
            .type_lowerer
            .lower_path(use_decl.path(), &GenericParams::empty())
            .map(|path| path.discard_args());
        let alias = use_decl
            .alias()
            .map(|alias| self.type_lowerer.lower_name(Some(alias)));
        let use_decl = UseDecl::new(path, alias, false);
        let use_decl_id = self.item_tree.uses.insert(use_decl);
        ItemTreeIdx::Use(use_decl_id).in_mod(self.module_id).into()
    }

    fn lower_visibility(&mut self, visibility: Option<ast::Visibility>) -> Spanned<Visibility> {
        lower_node(
            self,
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
        lower_optional_node_mut(
            self,
            generic_param_list,
            |_| GenericParams::default().at(fallback_span),
            |this, generic_params| {
                let mut hir_generic_param_list = GenericParams::default();
                generic_params.type_params().for_each(|param| {
                    let name = this.type_lowerer.lower_name(param.name());
                    hir_generic_param_list.types.insert(name);
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
        lower_optional_node_mut(
            self,
            where_clause,
            |_| (),
            |this, where_clause| {
                where_clause.predicates().for_each(|predicate| {
                    let name = this.type_lowerer.lower_name(predicate.name());
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
                            this.diagnostics.push(
                                LowerError::UnknownGeneric {
                                    name: this.interner.resolve(&name.inner).to_string(),
                                    name_file_span: name.span.in_file(this.file_id),
                                }
                                .to_diagnostic(),
                            );
                            GenericParams::INVALID_ID
                        });

                    if let Some(type_bound_list) = predicate.type_bound_list() {
                        type_bound_list.type_bounds().for_each(|type_bound| {
                            let bound = this
                                .type_lowerer
                                .lower_path(type_bound.trait_path(), &generic_params);
                            generic_params
                                .where_predicates
                                .push(WherePredicate::new(idx, name.inner, bound))
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
        lower_node(
            self,
            param_list,
            |_, param_list| ParamList::empty().at(param_list.range().to_span()),
            |this, param_list| {
                let params = param_list
                    .params()
                    .map(|param| {
                        let name = this.type_lowerer.lower_name(param.name());
                        let ty = this.type_lowerer.lower_type(param.ty(), generic_params);
                        Param::new(name, ty)
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
    ) -> Spanned<Type> {
        lower_node(
            self,
            return_type,
            |_, _| Type::unit().at(fallback_span),
            |this, ret_ty| {
                let span = ret_ty.range().to_span();
                match ret_ty.ty() {
                    Some(ty) => this.type_lowerer.lower_type(Some(ty), generic_params),
                    None => Type::unit().at(span),
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
                let name = self.type_lowerer.lower_name(associated_type_decl.name());
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
        lower_optional_node_mut(
            self,
            type_bound_list,
            |_| TypeBoundList::new(vec![]),
            |this, type_bound_list| {
                TypeBoundList::new(
                    type_bound_list
                        .type_bounds()
                        .map(|bound| {
                            this.type_lowerer
                                .lower_path(bound.trait_path(), &generic_params)
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
    ) -> Vec<id::FnDecl> {
        trait_method_decls
            .map(|method_decl| {
                let name = self.type_lowerer.lower_name(method_decl.name());
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
                                    .in_file(self.file_id),
                                generics_that_caused_duplication: duplicates
                                    .iter()
                                    .map(|key| self.interner.resolve(key).to_string())
                                    .collect(),
                                generics_that_caused_duplication_file_span: span
                                    .in_file(self.file_id),
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
                self.item_tree.functions.insert(fn_decl)
            })
            .collect()
    }

    fn lower_apply_to_ty(
        &mut self,
        to_ty: Option<ast::ApplyDeclType>,
        generic_params: &GenericParams,
    ) -> Spanned<Type> {
        lower_node(
            self,
            to_ty,
            |_, to_ty| Type::unknown().at(to_ty.range().to_span()),
            |this, to_ty: ast::ApplyDeclType| {
                this.type_lowerer.lower_type(to_ty.ty(), generic_params)
            },
        )
    }

    fn lower_associated_type_definitions(
        &mut self,
        assoc_types: impl Iterator<Item = ast::ApplyDeclAssocType>,
        generic_params: &GenericParams,
    ) -> Vec<AssociatedTypeDefinition> {
        assoc_types
            .map(|assoc_type| {
                let name = self.type_lowerer.lower_name(assoc_type.name());
                let ty = self
                    .type_lowerer
                    .lower_type(assoc_type.ty(), generic_params);
                AssociatedTypeDefinition::new(name, ty)
            })
            .collect()
    }

    fn lower_apply_methods(
        &mut self,
        methods: impl Iterator<Item = ast::FnDecl>,
        apply_generic_params: &Spanned<GenericParams>,
    ) -> Vec<id::FnDecl> {
        methods
            .map(|method| {
                self.lower_fn_decl(&method, Some(&apply_generic_params))
                    .inner
                    .clone()
                    .try_into()
                    .unwrap_or_else(|_| {
                        ice("`lower_fn_decl` returned `ItemId` not associated with function")
                    })
            })
            .collect()
    }

    fn lower_struct_field_decl_list(
        &mut self,
        struct_name: &Spanned<Word>,
        field_list: Option<ast::StructDeclFieldList>,
        generic_params: &Spanned<GenericParams>,
    ) -> StructFieldDeclList {
        let mut generic_params_used = HashSet::with_capacity(generic_params.types.len());
        let field_decl_list = lower_node(
            self,
            field_list,
            |_, _| StructFieldDeclList::empty(),
            |this, field_list| {
                StructFieldDeclList::new(
                    field_list
                        .fields()
                        .map(|field| {
                            let name = this.type_lowerer.lower_name(field.name());
                            let ty = this.type_lowerer.lower_type(field.ty(), generic_params);
                            ty.generics_used(&mut generic_params_used);
                            StructFieldDecl::new(name, ty)
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
        let mut generic_params_used = HashSet::with_capacity(generic_params.types.len());
        let variants = EnumDeclVariantList::new(
            variants
                .map(|variant| {
                    let name = self.type_lowerer.lower_name(variant.name());
                    let ty = variant.ty().map(|ty| {
                        let ty = self.type_lowerer.lower_type(Some(ty), generic_params);
                        ty.generics_used(&mut generic_params_used);
                        ty
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
        generics_used: &HashSet<Word>,
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
                    Some(self.interner.resolve(&item).to_string())
                } else {
                    None
                }
            })
            .collect();

        if !unused_generics.is_empty() {
            self.diagnostics.push(
                LowerError::UnusedGenerics {
                    item_name: self.interner.resolve(&item_name).to_string(),
                    item_name_file_span: item_name.span.in_file(self.file_id),
                    unused_generics: unused_generics,
                    unused_generics_file_span: generic_params.span.in_file(self.file_id),
                }
                .to_diagnostic(),
            );
        }
    }
}

mod diagnostics {}
