use flux_diagnostics::Diagnostic;
use flux_syntax::ast::{self, Root};

use crate::{
    body::LowerCtx,
    hir::{Function, Path},
    item::ItemId,
    module::ModuleId,
};

use super::ItemTree;

pub(super) struct Ctx<'a> {
    body_ctx: crate::body::LowerCtx,
    diagnostics: Vec<Diagnostic>,
    // file: InputFile,
    item_tree: &'a mut ItemTree,
    module_id: ModuleId,
}

impl<'a> Ctx<'a> {
    pub fn new(item_tree: &'a mut ItemTree, module_id: ModuleId) -> Self {
        Self {
            body_ctx: LowerCtx,
            diagnostics: vec![],
            item_tree,
            module_id,
        }
    }

    pub(super) fn lower_module_items(mut self, root: &Root) -> Vec<ItemId> {
        root.items().map(|item| self.lower_item(&item)).collect()
    }

    fn lower_item(&mut self, item: &ast::Item) -> ItemId {
        let item_id = match item {
            ast::Item::FnDecl(function) => self.lower_function(function, &None),
            _ => todo!(),
        };

        self.item_tree.top_level.push(item_id.clone());
        item_id
    }

    fn lower_function(&mut self, functoin: &ast::FnDecl, this_trait: &Option<Path>) -> ItemId {
        todo!()
        // let res = Function::new(db, name, visibility, generic_params);
        // id(self.item_tree.functions.alloc(res))
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
