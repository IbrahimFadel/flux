use flux_diagnostics::{ice, Diagnostic, ToDiagnostic};
use flux_span::{FileId, Interner, Span, Spanned, ToSpan, WithSpan};
use flux_syntax::ast::{self, AstNode, Root};
use flux_typesystem::{Insert, TEnv, TypeId};
use la_arena::Idx;

use crate::{
    body::LowerCtx,
    diagnostics::LowerError,
    hir::{
        ApplyDecl, AssociatedTypeDecl, AssociatedTypeDefinition, FnDecl, GenericParams, ModDecl,
        Param, ParamList, Path, TraitDecl, Type, TypeBound, TypeBoundList, Visibility,
        WherePredicate,
    },
    item::{ItemId, ItemTreeIdx},
    module::ModuleId,
};

use super::ItemTree;

pub struct Ctx<'a> {
    pub(crate) body_ctx: crate::body::LowerCtx<'a>,
    diagnostics: Vec<Diagnostic>,
    pub(crate) file_id: FileId,
    item_tree: &'a mut ItemTree,
    module_id: ModuleId,
}

impl<'a> Ctx<'a> {
    pub(crate) fn new(
        item_tree: &'a mut ItemTree,
        module_id: ModuleId,
        interner: &'static Interner,
        file_id: FileId,
        tenv: &'a mut TEnv,
    ) -> Self {
        Self {
            body_ctx: LowerCtx::new(interner, tenv),
            diagnostics: vec![],
            item_tree,
            module_id,
            file_id,
        }
    }

    pub(super) fn lower_module_items(mut self, root: &Root) -> Vec<ItemId> {
        root.items().map(|item| self.lower_item(&item)).collect()
    }

    fn lower_item(&mut self, item: &ast::Item) -> ItemId {
        let item_id = match item {
            ast::Item::ApplyDecl(apply_decl) => self.lower_apply_decl(apply_decl),
            ast::Item::EnumDecl(_) => todo!(),
            ast::Item::FnDecl(fn_decl) => self.lower_fn_decl(fn_decl, None),
            ast::Item::ModDecl(mod_decl) => self.lower_mod_decl(mod_decl),
            ast::Item::StructDecl(_) => todo!(),
            ast::Item::TraitDecl(trait_decl) => self.lower_trait_decl(trait_decl),
            ast::Item::UseDecl(_) => todo!(),
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
        let methods =
            self.lower_apply_methods(apply_decl.methods(), trt.as_ref().map(|path| &path.inner));

        let apply = ApplyDecl::new(visibility, generic_params, trt, to_ty, assoc_types, methods);
        let apply_id = self.item_tree.applies.alloc(apply);
        ItemId::new(self.module_id, ItemTreeIdx::Apply(apply_id))
    }

    fn lower_fn_decl(&mut self, function: &ast::FnDecl, this_trait: Option<&Path>) -> ItemId {
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
            this_trait,
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
        ItemId::new(self.module_id, ItemTreeIdx::Function(fn_id))
    }

    fn lower_mod_decl(&mut self, mod_decl: &ast::ModDecl) -> ItemId {
        let visibility = self.lower_visibility(mod_decl.visibility());
        let name = self.body_ctx.lower_name(mod_decl.name());
        let mod_decl = ModDecl::new(visibility, name);
        let mod_decl_id = self.item_tree.mods.alloc(mod_decl);
        ItemId::new(self.module_id, ItemTreeIdx::Module(mod_decl_id))
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

        let path_to_trait = Path::new(vec![name.inner], vec![]);
        let methods = self.lower_trait_method_decls(
            trait_decl.method_decls(),
            &generic_params,
            Some(&path_to_trait),
        );
        let trait_decl =
            TraitDecl::new(visibility, name, generic_params, associated_types, methods);
        let trait_id = self.item_tree.traits.alloc(trait_decl);
        ItemId::new(self.module_id, ItemTreeIdx::Trait(trait_id))
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
                hir_generic_param_list.at(fallback_span)
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
                                    name: self.body_ctx.interner.resolve(&name.inner).to_string(),
                                    name_file_span: name.span.in_file(self.file_id),
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
        this_trait: Option<&Path>,
    ) -> Spanned<TypeId> {
        self.body_ctx.lower_node(
            return_type,
            |this, _| this.tenv.insert(Type::unit()).at(fallback_span),
            |this, ret_ty| match ret_ty.ty() {
                Some(ty) => match this_trait {
                    Some(this_trait) => {
                        this.lower_trait_method_type(Some(ty), generic_params, this_trait)
                    }
                    None => this.lower_type(Some(ty), generic_params),
                },
                None => this.tenv.insert(Type::unit()).at(ret_ty.range().to_span()),
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
        this_trait: Option<&Path>,
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
                        .union(trait_generic_params, span, self)
                        .unwrap_or_else(|(fallback_generic_params, duplicates)| {
                            let diagnostic = LowerError::DuplicateGenericParams {
                                generics_that_were_chilling: (),
                                generics_that_were_chilling_file_span: trait_generic_params
                                    .span
                                    .in_file(self.file_id),
                                generics_that_caused_duplication: duplicates
                                    .iter()
                                    .map(|key| self.body_ctx.interner.resolve(key).to_string())
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
                    this_trait,
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
            |this, to_ty| this.tenv.insert(Type::Unknown).at(to_ty.range().to_span()),
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
        this_trait: Option<&Path>,
    ) -> Vec<Idx<FnDecl>> {
        methods
            .map(|method| {
                self.lower_fn_decl(&method, this_trait)
                    .idx
                    .try_into()
                    .unwrap_or_else(|_| {
                        ice("lower_fn_decl returned ItemId not associated with function")
                    })
            })
            .collect()
    }
}

// use std::marker::PhantomData;

// use flux_diagnostics::{ice, Diagnostic, ToDiagnostic};
// use flux_span::{InputFile, Span, Spanned, ToSpan, WithSpan};
// use flux_syntax::ast::{self, AstNode, Root};
// use la_arena::{Arena, Idx};
// use text_size::TextRange;

// use crate::{hir::Visibility, module::ModuleId};

// use super::{Item, ItemTree};

// // fn id<N: ItemTreeNode>(index: Idx<N>) -> FileItemTreeId<N> {
// //     FileItemTreeId {
// //         index,
// //         _p: PhantomData,
// //     }
// // }

// pub(super) struct Ctx<'a> {
//     // body_ctx: crate::body::LowerCtx<'a>,
//     diagnostics: Vec<Diagnostic>,
//     file: InputFile,
//     item_tree: &'a mut ItemTree,
//     module_id: ModuleId,
// }

// impl<'a> Ctx<'a> {
//     pub fn new(file: InputFile, item_tree: &'a mut ItemTree, module_id: ModuleId) -> Self {
//         Self {
//             diagnostics: vec![],
//             file,
//             item_tree,
//             module_id,
//         }
//     }

//     pub(super) fn lower_module_items(mut self, root: &Root) -> Vec<Item> {
//         root.items().map(|item| self.lower_item(&item)).collect()
//     }

//     fn lower_item(&mut self, item: &ast::Item) -> Item {
//         let mod_item: Item = match item {
//             // ast::Item::ApplyDecl(a) => self.lower_apply(a).into(),
//             // ast::Item::EnumDecl(e) => self.lower_enum(e).into(),
//             ast::Item::FnDecl(function) => self.lower_function(function, &None).into(),
//             // ast::Item::ModDecl(m) => self.lower_mod(m).into(),
//             // ast::Item::StructDecl(s) => self.lower_struct(s).into(),
//             // ast::Item::TraitDecl(t) => self.lower_trait(t).into(),
//             // ast::Item::UseDecl(u) => self.lower_use(u).into(),
//             _ => todo!(),
//         };
//         self.item_tree.top_level.push((mod_item, self.module_id));
//         mod_item
//     }

//     // fn lower_apply(&mut self, apply: &ast::ApplyDecl) -> Item {
//     //     let visibility = self.lower_visibility(apply.visibility());
//     //     let generic_params = self.lower_generic_params(
//     //         apply.generic_param_list(),
//     //         apply.where_clause(),
//     //         visibility.span,
//     //     );
//     //     let trt = apply.trt().map(|trt| {
//     //         self.body_ctx
//     //             .lower_path(trt.path(), &generic_params, self.file_id)
//     //     });
//     //     // let ty = self.body_ctx.lower_node(
//     //     //     apply.to_ty(),
//     //     //     |this, ty| {
//     //     //         this.types
//     //     //             .alloc(Type::Unknown.at(ty.range().to_span()))
//     //     //             .into()
//     //     //     },
//     //     //     |this, ty| this.lower_type(ty.ty(), &generic_params),
//     //     // );
//     //     let ty = self.body_ctx.lower_node(
//     //         apply.to_ty(),
//     //         |this, ty| {
//     //             this.tchk
//     //                 .tenv
//     //                 .insert_unknown(ty.range().to_span().in_file(self.file_id))
//     //         },
//     //         |this, ty| this.lower_type(ty.ty(), &generic_params, self.file_id),
//     //     );
//     //     // self.tenv.insert(ts::Type::new(TypeKin))
//     //     let assoc_types = self.lower_apply_assoc_types(apply.associated_types(), &generic_params);
//     //     let (methods, methods_end) = self.lower_apply_methods(apply.methods(), &trt.clone());

//     //     let (l, r) = match (apply.lbrace(), apply.rbrace()) {
//     //         (Some(lbrace), Some(rbrace)) => {
//     //             (lbrace.text_range().start(), rbrace.text_range().end())
//     //         }
//     //         (Some(lbrace), None) => match methods_end {
//     //             Some(methods_end) => (lbrace.text_range().end(), methods_end),
//     //             None => (lbrace.text_range().end(), lbrace.text_range().end()),
//     //         },
//     //         (None, Some(rbrace)) => (generic_params.span.range.end(), rbrace.text_range().end()),
//     //         (None, None) => match methods_end {
//     //             Some(methods_end) => (generic_params.span.range.end(), methods_end),
//     //             None => (
//     //                 generic_params.span.range.end(),
//     //                 generic_params.span.range.end(),
//     //             ),
//     //         },
//     //     };
//     //     let apply_methods_span = TextRange::new(l, r).to_span();
//     //     let methods = methods.at(apply_methods_span);
//     //     let res = Apply {
//     //         visibility,
//     //         generic_params,
//     //         trt,
//     //         ty,
//     //         assoc_types,
//     //         methods,
//     //     };
//     //     id(self.item_tree.applies.alloc(res))
//     // }

//     // fn lower_enum(&mut self, eenum: &ast::EnumDecl) -> Item {
//     //     let visibility = self.lower_visibility(eenum.visibility());
//     //     let name = self.body_ctx.lower_name(eenum.name());
//     //     let generic_params =
//     //         self.lower_generic_params(eenum.generic_param_list(), eenum.where_clause(), name.span);
//     //     let variants = self.lower_enum_variants(eenum.variants(), &generic_params);
//     //     let res = Enum {
//     //         visibility,
//     //         name,
//     //         generic_params,
//     //         variants,
//     //     };
//     //     id(self.item_tree.enums.alloc(res))
//     // }

//     fn lower_function(
//         &mut self,
//         function: &ast::FnDecl,
//         this_trait: &Option<Spanned<Path>>,
//     ) -> Item {
//         let visibility = self.lower_visibility(function.visibility());
//         let name = self.body_ctx.lower_name(function.name());
//         let generic_params = self.lower_generic_params(
//             function.generic_param_list(),
//             function.where_clause(),
//             name.span,
//         );
//         let params = self.lower_params(function.param_list(), &generic_params);
//         let ret_ty = self.lower_return_type(function.return_type(), &generic_params, this_trait);
//         let res = Function {
//             visibility,
//             name,
//             generic_params,
//             params,
//             ret_ty,
//             ast: Some(function.clone()),
//         };
//         id(self.item_tree.functions.alloc(res))
//     }

//     // fn lower_mod(&mut self, m: &ast::ModDecl) -> FileItemTreeId<Mod> {
//     //     let visibility = self.lower_visibility(m.visibility()).inner;
//     //     let name = self.body_ctx.lower_name(m.name());
//     //     let res = Mod { visibility, name };
//     //     id(self.item_tree.mods.alloc(res))
//     // }

//     // fn lower_struct(&mut self, s: &ast::StructDecl) -> FileItemTreeId<Struct> {
//     //     let visibility = self.lower_visibility(s.visibility());
//     //     let name = self.body_ctx.lower_name(s.name());
//     //     let generic_params =
//     //         self.lower_generic_params(s.generic_param_list(), s.where_clause(), name.span);
//     //     let fields = self.lower_struct_fields(s.field_list(), &generic_params);
//     //     let res = Struct {
//     //         visibility,
//     //         name,
//     //         generic_params,
//     //         fields,
//     //     };
//     //     id(self.item_tree.structs.alloc(res))
//     // }

//     // fn lower_enum_variants(
//     //     &mut self,
//     //     variants: impl Iterator<Item = ast::EnumDeclVariant>,
//     //     generic_params: &GenericParams,
//     // ) -> Vec<EnumVariant> {
//     //     variants
//     //         .map(|variant| {
//     //             let name = self.body_ctx.lower_name(variant.name());
//     //             let ty = variant.ty().map(|ty| {
//     //                 self.body_ctx
//     //                     .lower_type(Some(ty), generic_params, self.file_id)
//     //             });
//     //             EnumVariant { name, ty }
//     //         })
//     //         .collect()
//     // }

//     // fn lower_struct_fields(
//     //     &mut self,
//     //     field_list: Option<ast::StructDeclFieldList>,
//     //     generic_params: &GenericParams,
//     // ) -> StructFields {
//     //     self.body_ctx.lower_node(
//     //         field_list,
//     //         |_, _| StructFields::poisoned(),
//     //         |this, field_list| {
//     //             let fields = field_list
//     //                 .fields()
//     //                 .map(|field| {
//     //                     let name = this.lower_name(field.name());
//     //                     let ty = this.lower_type(field.ty(), &generic_params, self.file_id);
//     //                     StructField { name, ty }
//     //                 })
//     //                 .collect();
//     //             StructFields { fields }
//     //         },
//     //     )
//     // }

//     // fn lower_trait(&mut self, t: &ast::TraitDecl) -> FileItemTreeId<Trait> {
//     //     let visibility = self.lower_visibility(t.visibility());
//     //     let name = self.body_ctx.lower_name(t.name());
//     //     let generic_params =
//     //         self.lower_generic_params(t.generic_param_list(), t.where_clause(), name.span);
//     //     let assoc_types = self.lower_trait_assoc_types(t.associated_types(), &generic_params);
//     //     let (methods, methods_end) = self.lower_trait_methods(
//     //         t.method_decls(),
//     //         &Some(Path::from_spur(name.inner, self.string_interner).at(name.span)),
//     //     );

//     //     let (l, r) = match (t.lbrace(), t.rbrace()) {
//     //         (Some(lbrace), Some(rbrace)) => {
//     //             (lbrace.text_range().start(), rbrace.text_range().end())
//     //         }
//     //         (Some(lbrace), None) => match methods_end {
//     //             Some(methods_end) => (lbrace.text_range().end(), methods_end),
//     //             None => (lbrace.text_range().end(), lbrace.text_range().end()),
//     //         },
//     //         (None, Some(rbrace)) => (generic_params.span.range.end(), rbrace.text_range().end()),
//     //         (None, None) => match methods_end {
//     //             Some(methods_end) => (generic_params.span.range.end(), methods_end),
//     //             None => (
//     //                 generic_params.span.range.end(),
//     //                 generic_params.span.range.end(),
//     //             ),
//     //         },
//     //     };
//     //     let trait_methods_span = TextRange::new(l, r).to_span();
//     //     let methods = methods.at(trait_methods_span);
//     //     let res = Trait {
//     //         visibility,
//     //         name,
//     //         generic_params,
//     //         assoc_types,
//     //         methods,
//     //     };
//     //     id(self.item_tree.traits.alloc(res))
//     // }

//     // fn lower_use(&mut self, u: &ast::UseDecl) -> FileItemTreeId<Use> {
//     //     let visibility = self.lower_visibility(u.visibility()).inner;
//     //     let path = self
//     //         .body_ctx
//     //         .lower_path(u.path(), &GenericParams::poisoned(), self.file_id);
//     //     let alias = u.alias().map(|alias| self.body_ctx.lower_name(Some(alias)));
//     //     let res = Use {
//     //         visibility,
//     //         path,
//     //         alias,
//     //     };
//     //     id(self.item_tree.uses.alloc(res))
//     // }

//     fn lower_visibility(&mut self, visibility: Option<ast::Visibility>) -> Spanned<Visibility> {
//         self.body_ctx.lower_node(
//             visibility,
//             |_, visibility| Visibility::Private.at(visibility.range().to_span()),
//             |_, visibility| {
//                 visibility
//                     .public()
//                     .map_or(Visibility::Private, |_| Visibility::Public)
//                     .at(visibility.range().to_span())
//             },
//         )
//     }

//     fn lower_generic_params(
//         &mut self,
//         generic_params: Option<ast::GenericParamList>,
//         where_clause: Option<ast::WhereClause>,
//         fallback_span: Span,
//     ) -> Spanned<GenericParams> {
//         let mut generic_params = match generic_params {
//             Some(ast_generic_params) => {
//                 let mut generic_params = GenericParams::default();
//                 ast_generic_params.type_params().for_each(|param| {
//                     let name = self.body_ctx.lower_name(param.name());
//                     generic_params.types.alloc(name);
//                 });
//                 generic_params.at(ast_generic_params.range().to_span())
//             }
//             None => GenericParams::poisoned().at(fallback_span),
//         };

//         if let Some(where_clause) = where_clause {
//             where_clause.predicates().for_each(|predicate| {
//                 let name = self.body_ctx.lower_name(predicate.name());
//                 let idx = generic_params
//                     .types
//                     .iter()
//                     .find(|(_, n)| **n == name)
//                     .map(|(idx, _)| idx)
//                     .unwrap_or_else(|| {
//                         self.diagnostics.push(
//                             LowerError::UnknownGeneric {
//                                 generic: self.string_interner.resolve(&name.inner).to_string(),
//                                 generic_file_span: name.span.in_file(self.file_id),
//                                 generic_params: generic_params
//                                     .types
//                                     .iter()
//                                     .map(|(_, param)| {
//                                         self.string_interner.resolve(param).to_string()
//                                     })
//                                     .collect(),
//                                 generic_params_file_span: generic_params.span.in_file(self.file_id),
//                             }
//                             .to_diagnostic(),
//                         );
//                         generic_params.invalid_idx()
//                     });
//                 self.body_ctx.lower_node(
//                     predicate.type_bound_list(),
//                     |_, _| todo!(),
//                     |this, type_bound_list| {
//                         type_bound_list.type_bounds().for_each(|bound| {
//                             let path =
//                                 this.lower_path(bound.trait_path(), &generic_params, self.file_id);
//                             generic_params
//                                 .inner
//                                 .where_predicates
//                                 .0
//                                 .push(WherePredicate {
//                                     ty: idx,
//                                     name: name.clone(),
//                                     bound: path,
//                                 })
//                         })
//                     },
//                 );
//             });
//         }
//         generic_params
//     }

//     fn lower_trait_assoc_types(
//         &mut self,
//         assoc_types: impl Iterator<Item = ast::TraitAssocTypeDecl>,
//         generic_params: &GenericParams,
//     ) -> Vec<(Name, Vec<Spanned<Path>>)> {
//         assoc_types
//             .map(|ty| {
//                 let name = self.body_ctx.lower_name(ty.name());
//                 let type_bound_list = ty.type_bound_list().map_or(vec![], |type_bound_list| {
//                     self.lower_type_bound_list(Some(type_bound_list), generic_params)
//                 });
//                 (name, type_bound_list)
//             })
//             .collect()
//     }

//     fn lower_type_bound_list(
//         &mut self,
//         type_bound_list: Option<ast::TypeBoundList>,
//         generic_params: &GenericParams,
//     ) -> Vec<Spanned<Path>> {
//         self.body_ctx.lower_node(
//             type_bound_list,
//             |_, _| todo!(),
//             |this, type_bound_list| {
//                 type_bound_list
//                     .type_bounds()
//                     .map(|bound| this.lower_path(bound.trait_path(), &generic_params, self.file_id))
//                     .collect()
//             },
//         )
//     }

//     fn lower_apply_assoc_types(
//         &mut self,
//         assoc_types: impl Iterator<Item = ast::ApplyDeclAssocType>,
//         generic_params: &GenericParams,
//     ) -> Vec<(Name, TypeId)> {
//         assoc_types
//             .map(|ty| {
//                 let name = self.body_ctx.lower_name(ty.name());
//                 let ty = self
//                     .body_ctx
//                     .lower_type(ty.ty(), &generic_params, self.file_id);
//                 (name, ty)
//             })
//             .collect()
//     }

//     fn lower_apply_methods(
//         &mut self,
//         methods: impl Iterator<Item = ast::FnDecl>,
//         this_trait: &Option<Spanned<Path>>,
//     ) -> (Vec<Spanned<FunctionId>>, Option<TextSize>) {
//         let mut end = None;
//         let methods = methods
//             .map(|method| {
//                 let f = self.lower_function(&method, this_trait);
//                 end = Some(
//                     self.item_tree[f]
//                         .ast
//                         .as_ref()
//                         .unwrap_or_else(|| ice("apply method should always have ast"))
//                         .range()
//                         .end(),
//                 );
//                 Spanned::new(f.index, self.item_tree[f].name.span)
//             })
//             .collect();

//         (methods, end)
//     }

//     fn lower_trait_methods(
//         &mut self,
//         methods: impl Iterator<Item = ast::TraitMethodDecl>,
//         this_trait: &Option<Spanned<Path>>,
//     ) -> (Vec<Spanned<FunctionId>>, Option<TextSize>) {
//         let mut end = None;
//         let methods = methods
//             .map(|method| {
//                 let name = self.body_ctx.lower_name(method.name());
//                 let generic_params = self.lower_generic_params(
//                     method.generic_param_list(),
//                     method.where_clause(),
//                     name.span,
//                 );
//                 let params = self.lower_params(method.param_list(), &generic_params);
//                 let ret_ty =
//                     self.lower_return_type(method.return_ty(), &generic_params, this_trait);
//                 // let ret_ty_span = self.body_ctx.types[ret_ty.raw()].span;
//                 let ret_ty_span = self.body_ctx.tchk.tenv.get_type_filespan(ret_ty).inner;
//                 end = Some(ret_ty_span.range.end());
//                 let name_span = name.span;
//                 let f = Function {
//                     visibility: Visibility::Public.at(name_span), // Arbitrary really... since theres no real concept of visibility for trait methods, but i guess they are public
//                     name,
//                     generic_params,
//                     params,
//                     ret_ty,
//                     ast: None,
//                 };
//                 Spanned::new(self.item_tree.functions.alloc(f), name_span)
//             })
//             .collect();
//         (methods, end)
//     }

//     fn lower_params(
//         &mut self,
//         params: Option<ast::ParamList>,
//         generic_params: &GenericParams,
//     ) -> Spanned<Params> {
//         self.body_ctx.lower_node(
//             params,
//             |_, param_list| Params::new(vec![]).at(param_list.range().to_span()),
//             |this, param_list| {
//                 Params::new(
//                     param_list
//                         .params()
//                         .map(|param| {
//                             let name = this.lower_name(param.name());
//                             let ty = this.lower_type(param.ty(), &generic_params, self.file_id);
//                             Param { name, ty }
//                         })
//                         .collect(),
//                 )
//                 .at(param_list.range().to_span())
//             },
//         )
//     }

//     fn lower_return_type(
//         &mut self,
//         ty: Option<ast::FnReturnType>,
//         generic_params: &GenericParams,
//         this_trait: &Option<Spanned<Path>>,
//     ) -> TypeId {
//         self.body_ctx.lower_node(
//             ty,
//             |this, ty| {
//                 this.tchk
//                     .tenv
//                     .insert_unit(ty.range().to_span().in_file(self.file_id))
//                 // this.types
//                 //     .alloc(Type::Tuple(vec![]).at(ty.range().to_span()))
//                 //     .into()
//             },
//             |this, ty| match ty.ty() {
//                 Some(ty) => {
//                     if let Some(this_trait) = this_trait {
//                         this.lower_apply_method_type(
//                             Some(ty),
//                             generic_params,
//                             this_trait.clone(),
//                             self.file_id,
//                         )
//                     } else {
//                         this.lower_type(Some(ty), &generic_params, self.file_id)
//                     }
//                 }
//                 None => this
//                     .tchk
//                     .tenv
//                     .insert_unit(ty.range().to_span().in_file(self.file_id)),
//             },
//         )
//     }
// }
