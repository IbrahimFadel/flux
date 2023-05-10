use std::collections::HashSet;

use crate::{hir::WherePredicate, ApplyId};
use flux_span::FileSpanned;
use itertools::Itertools;
use ts::TraitApplication;

use super::*;

impl<'a> LowerCtx<'a> {
    pub(super) fn handle_apply(&mut self, a: ApplyId) {
        let a = &self.global_item_tree.unwrap()[a];
        let file_id = self.file_id();

        self.verify_where_predicates(&a.generic_params, &a.generic_params.span.in_file(file_id));

        if let Some(trt_path) = &a.trt {
            if let Some((trt, trait_id)) = self.get_trait_with_id(trt_path) {
                let methods = &trt.methods;
                let methods_in_file = methods.file_span_ref(trt.file_id, methods.span);
                self.verify_apply_methods_match_trait_methods(&methods_in_file, &a.methods);
                self.verify_apply_assoc_types_match_trait_assoc_types(&trt, a);

                self.add_trait_application_to_context(a, trait_id, trt_path);
            }
        }

        for f in &a.methods.inner {
            let f_generic_params = &self.global_item_tree.unwrap()[f.inner].generic_params;
            self.combine_generic_parameters(&a.generic_params, f_generic_params);
            self.handle_function(f.inner, Some(a));
        }
    }

    fn add_trait_application_to_context(
        &mut self,
        apply: &Apply,
        trait_id: TraitId,
        trait_path: &Spanned<Path>,
    ) {
        // TODO: how tf do i avoid all these allocations?
        let file_id = self.file_id();
        let impltr = self.insert_type_to_tenv(&apply.ty, file_id);
        let impltr_args = self
            .tchk
            .tenv
            .get_entry(impltr)
            .get_params()
            .map(|params| params.to_vec())
            .unwrap_or(vec![]);
        let impltr_filespan = self.tchk.tenv.get_type_filespan(impltr);
        let trait_params: Vec<_> = trait_path
            .generic_args
            .iter()
            .map(|ty| self.insert_type_to_tenv(ty, file_id))
            .collect();

        let impltr_args: Vec<_> = impltr_args
            .iter()
            .map(|kind| {
                let ty = ts::Type::new(kind.clone());
                self.tchk
                    .tenv
                    .insert(ty.file_span(impltr_filespan.file_id, impltr_filespan.inner))
            })
            .collect();

        tracing::debug!(
            "adding application of trait `{}{}` to `{}{}`",
            self.string_interner
                .resolve(&self.global_item_tree.unwrap()[trait_id].name),
            if trait_params.is_empty() {
                "".to_string()
            } else {
                format!(
                    "<{}>",
                    trait_params
                        .iter()
                        .map(|tid| self.tchk.tenv.fmt_ty_id(*tid))
                        .join(", ")
                )
            },
            self.tchk.tenv.fmt_ty_id(impltr),
            if impltr_args.is_empty() {
                "".to_string()
            } else {
                format!(
                    "<{}>",
                    impltr_args
                        .iter()
                        .map(|tid| self.tchk.tenv.fmt_ty_id(*tid))
                        .join(", ")
                )
            }
        );

        self.tchk.trait_applications.push_application(
            trait_id.into_raw().into(),
            TraitApplication::new(trait_params, impltr, impltr_args),
        );
    }

    fn verify_apply_assoc_types_match_trait_assoc_types(&mut self, trt: &InFile<Trait>, a: &Apply) {
        let trait_assoc_types = &trt.assoc_types;
        let apply_assoc_types = &a.assoc_types;

        let apply_trt_span = a
            .trt
            .as_ref()
            .unwrap_or_else(|| {
                ice("entered verify_apply_assoc_types_match_trait_assoc_types without trait in apply")
            })
            .span;

        // apply Foo to Bar {
        //       ^^^^^^^^^^
        let apply_span = Span::combine(apply_trt_span, self.types[a.ty.raw()].span);

        self.verify_assoc_types_defined_match_trait_assoc_type_list(
            &trt.name,
            apply_span,
            trait_assoc_types,
            apply_assoc_types,
        );
    }

    fn verify_assoc_types_defined_match_trait_assoc_type_list(
        &mut self,
        trait_name: &Spur,
        apply_span: Span,
        trait_assoc_types: &[(Name, Vec<Spanned<Path>>)],
        apply_assoc_types: &[(Name, TypeIdx)],
    ) {
        let trait_assoc_type_names: HashSet<Name> = trait_assoc_types
            .iter()
            .map(|(name, _)| name.clone())
            .collect();
        let apply_assoc_type_names: HashSet<Name> = apply_assoc_types
            .iter()
            .map(|(name, _)| name.clone())
            .collect();

        apply_assoc_type_names
            .difference(&trait_assoc_type_names)
            .for_each(|doest_belong| {
                self.diagnostics.push(
                    LowerError::AssocTypeDoesntBelong {
                        ty: self.string_interner.resolve(doest_belong).to_string(),
                        ty_file_span: doest_belong.span.in_file(self.file_id()),
                        trait_name: self.string_interner.resolve(&trait_name).to_string(),
                    }
                    .to_diagnostic(),
                );
            });
        let unassigned_assoc_types: Vec<_> = trait_assoc_type_names
            .difference(&apply_assoc_type_names)
            .collect();
        if !unassigned_assoc_types.is_empty() {
            self.diagnostics.push(
                LowerError::UnassignedAssocTypes {
                    types: unassigned_assoc_types
                        .iter()
                        .map(|spur| self.string_interner.resolve(spur).to_string())
                        .collect(),
                    apply: (),
                    apply_file_span: apply_span.in_file(self.file_id()),
                    trait_name: self.string_interner.resolve(trait_name).to_string(),
                }
                .to_diagnostic(),
            );
        }

        for (assoc_type_name, assoc_type_tyidx) in apply_assoc_types {
            let trait_assoc_type = trait_assoc_types
                .iter()
                .find(|(name, _)| name.inner == assoc_type_name.inner);

            if let Some((_, trait_assoc_type_restrictions)) = trait_assoc_type {
                let tid = self.insert_type_to_tenv(assoc_type_tyidx, self.file_id());
                let type_restrictions: Vec<_> = trait_assoc_type_restrictions
                    .iter()
                    .map(|restriction| self.path_to_trait_restriction(restriction))
                    .collect();
                self.tchk
                    .does_type_implement_restrictions(tid, &type_restrictions)
                    .unwrap_or_else(|err| {
                        self.diagnostics.push(err);
                    });
            }
        }
    }

    fn verify_apply_methods_match_trait_methods(
        &mut self,
        trait_methods: &InFile<Spanned<&Vec<Spanned<FunctionId>>>>,
        apply_methods: &Spanned<Vec<Spanned<FunctionId>>>,
    ) {
        self.verify_methods_defined_match_method_list(trait_methods, apply_methods);

        for apply_method in &apply_methods.inner {
            for trait_method in trait_methods.inner.inner {
                let apply_method = &self.global_item_tree.unwrap()[apply_method.inner];
                let trait_method = &self.global_item_tree.unwrap()[trait_method.inner];

                if apply_method.name == trait_method.name {
                    self.combine_generic_parameters(
                        &apply_method.generic_params,
                        &trait_method.generic_params,
                    );

                    let trait_method_generics = &trait_method.generic_params;
                    let trait_method_generics_file_spanned = trait_method_generics
                        .file_span_ref(trait_methods.file_id, trait_method_generics.span);
                    self.verify_apply_generic_params_match_trait_method_generic_params(
                        &trait_method_generics_file_spanned,
                        &apply_method.generic_params,
                    );
                }
            }
        }
    }

    fn verify_methods_defined_match_method_list(
        &mut self,
        trait_methods: &FileSpanned<&Vec<Spanned<FunctionId>>>,
        apply_methods: &Spanned<Vec<Spanned<FunctionId>>>,
    ) {
        let apply_method_names_filespanned: Vec<FileSpanned<String>> = apply_methods
            .iter()
            .map(|method_id| {
                self.global_item_tree.unwrap()[method_id.inner]
                    .name
                    .map_ref(|name| self.string_interner.resolve(name).to_string())
                    .in_file(self.file_id())
            })
            .collect();
        let apply_method_names: HashSet<Spur> = apply_methods
            .iter()
            .map(|method_id| self.global_item_tree.unwrap()[method_id.inner].name.inner)
            .collect();
        let trait_method_names_filespanned: Vec<FileSpanned<String>> = trait_methods
            .iter()
            .map(|method_id| {
                self.global_item_tree.unwrap()[method_id.inner]
                    .name
                    .map_ref(|name| self.string_interner.resolve(name).to_string())
                    .in_file(trait_methods.file_id)
            })
            .collect();
        let trait_method_names: HashSet<Spur> = trait_methods
            .iter()
            .map(|method_id| self.global_item_tree.unwrap()[method_id.inner].name.inner)
            .collect();
        let methods_that_dont_belong: Vec<_> =
            apply_method_names.difference(&trait_method_names).collect();
        let unimplemented_methods: Vec<_> =
            trait_method_names.difference(&apply_method_names).collect();

        if unimplemented_methods.len() > 0 {
            self.diagnostics.push(
                LowerError::UnimplementedTraitMethods {
                    trait_methods_declared: trait_method_names_filespanned,
                    unimplemented_methods: unimplemented_methods
                        .iter()
                        .map(|spur| self.string_interner.resolve(spur).to_string())
                        .collect(),
                    unimplemented_methods_file_span: apply_methods.span.in_file(self.file_id()),
                }
                .to_diagnostic(),
            );
        }

        if methods_that_dont_belong.len() > 0 {
            self.diagnostics.push(
                LowerError::MethodsDontBelongInApply {
                    trait_methods_declared: trait_method_names
                        .iter()
                        .map(|spur| self.string_interner.resolve(spur).to_string())
                        .collect(),
                    trait_methods_declared_file_span: trait_methods
                        .span
                        .in_file(trait_methods.file_id),
                    methods_that_dont_belond: apply_method_names_filespanned, // methods_that_dont_belond_file_span: apply_methods.span.in_file(self.file_id()),
                    apply_location: (),
                    apply_location_file_span: apply_methods.span.in_file(self.file_id()),
                }
                .to_diagnostic(),
            );
        }
    }

    fn verify_apply_generic_params_match_trait_method_generic_params(
        &mut self,
        trait_generic_params: &InFile<Spanned<&GenericParams>>,
        apply_generic_params: &Spanned<GenericParams>,
    ) {
        self.verify_generic_param_lengths_equal(trait_generic_params, apply_generic_params);

        for trait_where_predicate in &trait_generic_params.where_predicates.0 {
            let apply_where_predicate = apply_generic_params
                .where_predicates
                .0
                .iter()
                .find(|predicate| predicate.ty == trait_where_predicate.ty);

            let trait_trait = self.get_trait(&trait_where_predicate.bound);

            match apply_where_predicate {
                Some(apply_where_predicate) => {
                    let apply_trait = self.get_trait(&apply_where_predicate.bound);
                    self.verify_option_trait_names_equal(
                        trait_trait,
                        apply_trait,
                        trait_where_predicate,
                        apply_where_predicate,
                        trait_generic_params,
                    );
                }
                None => {
                    if trait_where_predicate.ty != trait_generic_params.invalid_idx() {
                        let apply_decl_generic_param_name =
                            &apply_generic_params.types[trait_where_predicate.ty];
                        self.diagnostics.push(
                            LowerError::MissingWherePredicateInApplyMethod {
                                trait_decl_where_predicate: trait_where_predicate
                                    .bound
                                    .to_string(self.string_interner),
                                trait_decl_where_predicate_file_span: trait_where_predicate
                                    .bound
                                    .span
                                    .in_file(trait_generic_params.file_id),
                                apply_decl_generic_missing_where_predicate: self
                                    .string_interner
                                    .resolve(&apply_decl_generic_param_name)
                                    .to_string(),
                                apply_decl_generic_missing_where_predicate_file_span:
                                    apply_decl_generic_param_name.span.in_file(self.file_id()),
                            }
                            .to_diagnostic(),
                        );
                    }
                }
            }
        }
    }

    fn verify_generic_param_lengths_equal(
        &mut self,
        trait_generic_params: &InFile<Spanned<&GenericParams>>,
        apply_generic_params: &Spanned<GenericParams>,
    ) {
        let trait_params_len = trait_generic_params.types.len();
        let apply_params_len = apply_generic_params.types.len();
        if trait_params_len != apply_params_len {
            self.diagnostics.push(
                LowerError::IncorrectNumGenericParamsInApplyMethod {
                    got_num: apply_params_len,
                    got_num_file_span: apply_generic_params.span.in_file(self.file_id()),
                    expected_num: trait_params_len,
                    expected_num_file_span: trait_generic_params
                        .span
                        .in_file(trait_generic_params.file_id),
                }
                .to_diagnostic(),
            );
        }
    }

    fn verify_option_trait_names_equal(
        &mut self,
        trait_trait: Option<InFile<Trait>>,
        apply_trait: Option<InFile<Trait>>,
        trait_where_predicate: &WherePredicate,
        apply_where_predicate: &WherePredicate,
        trait_generic_params: &InFile<Spanned<&GenericParams>>,
    ) {
        if !matches!((trait_trait, apply_trait), (Some(trait_trait), Some(apply_trait)) if trait_trait.name.inner == apply_trait.name.inner)
        {
            self.diagnostics.push(
                LowerError::WherePredicatesDontMatchInApplyMethod {
                    trait_decl_where_predicate: trait_where_predicate
                        .bound
                        .to_string(self.string_interner),
                    trait_decl_where_predicate_file_span: trait_where_predicate
                        .bound
                        .span
                        .in_file(trait_generic_params.file_id),
                    apply_decl_where_predicate: apply_where_predicate
                        .bound
                        .to_string(self.string_interner),
                    apply_decl_where_predicate_file_span: apply_where_predicate
                        .bound
                        .span
                        .in_file(self.file_id()),
                    trait_generic_param: self
                        .string_interner
                        .resolve(&trait_where_predicate.name)
                        .to_string(),
                    trait_generic_param_file_span: trait_where_predicate
                        .name
                        .span
                        .in_file(trait_generic_params.file_id),
                    apply_generic_param: self
                        .string_interner
                        .resolve(&apply_where_predicate.name)
                        .to_string(),
                    apply_generic_param_file_span: apply_where_predicate
                        .name
                        .span
                        .in_file(self.file_id()),
                }
                .to_diagnostic(),
            );
        }
    }
}
