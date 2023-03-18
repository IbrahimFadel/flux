use flux_span::Span;
use hashbrown::HashSet;
use lasso::Spur;

use super::*;

impl<'a> LowerCtx<'a> {
    pub(super) fn handle_apply(&mut self, a: Idx<Apply>, item_tree: &ItemTree) {
        let a = &item_tree[a];
        self.check_where_predicates(
            &a.generic_params,
            a.generic_params.span.in_file(self.file_id()),
            self.cur_module_id,
        );
        if let Some(trt_path) = &a.trt {
            let trt = self.get_trait(trt_path);
            match trt {
                Some((trt, trt_mod_id)) => {
                    self.check_trait_methods_with_apply_methods(
                        &trt.methods,
                        trt_mod_id,
                        &a.methods,
                    );
                    self.check_trait_assoc_types_with_apply_assoc_types(&trt, a);

                    let file_id = self.file_id();

                    let trt_args = &trt_path
                        .generic_args
                        .iter()
                        .map(|ty| self.insert_type_to_tenv(ty, file_id))
                        .collect::<Vec<_>>();

                    let impltr = self.insert_type_to_tenv(&a.ty, file_id);
                    self.tchk
                        .add_trait_application_to_context(
                            &trt_path
                                .map_ref(|path| path.to_spur(self.string_interner))
                                .in_file(file_id),
                            trt_args,
                            impltr,
                        )
                        .unwrap_or_else(|err| {
                            self.diagnostics.push(err);
                        });
                }
                None => todo!(),
            }
        }
        for f in &a.methods.inner {
            let f_generic_params = &item_tree[*f].generic_params;
            self.combine_generic_parameters(&a.generic_params, f_generic_params);
            self.handle_function(*f, item_tree);
        }
    }

    fn check_trait_assoc_types_with_apply_assoc_types(&mut self, trt: &InFile<Trait>, a: &Apply) {
        let trait_assoc_types = &trt.assoc_types;
        let apply_assoc_types = &a.assoc_types;

        let apply_trt_span = a
            .trt
            .as_ref()
            .unwrap_or_else(|| {
                ice("entered check_trait_assoc_types_with_apply_assoc_types without trait in apply")
            })
            .span;

        // apply Foo to Bar {
        //       ^^^^^^^^^^
        let apply_span = Span::combine(apply_trt_span, self.types[a.ty.raw()].span);

        self.check_assoc_types_differences(
            &trt.name,
            apply_span,
            trait_assoc_types,
            apply_assoc_types,
        );
    }

    fn check_assoc_types_differences(
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
                    .check_if_type_implements_restrictions(tid, &type_restrictions)
                    .unwrap_or_else(|err| {
                        self.diagnostics.push(err);
                    });
            }
        }
    }

    fn path_to_trait_restriction(&mut self, path: &Spanned<Path>) -> ts::TraitRestriction {
        let name = path
            .map_ref(|path| path.to_spur(self.string_interner))
            .in_file(self.file_id());
        let args = path
            .generic_args
            .iter()
            .map(|idx| self.insert_type_to_tenv(idx, self.file_id()))
            .collect();
        ts::TraitRestriction { name, args }
    }

    fn check_trait_methods_with_apply_methods(
        &mut self,
        trait_methods: &Spanned<Vec<FunctionId>>,
        trait_module_id: ModuleId,
        apply_methods: &Spanned<Vec<FunctionId>>,
    ) {
        self.check_trait_methods_match_apply_methods(trait_methods, trait_module_id, apply_methods);

        let def_map = self.def_map.unwrap();

        for apply_method in &apply_methods.inner {
            // GenericParams::combine(t, b);
            for trait_method in &trait_methods.inner {
                let apply_method = &def_map.item_trees[self.cur_module_id][*apply_method];
                let trait_method = &def_map.item_trees[trait_module_id][*trait_method];

                if apply_method.name == trait_method.name {
                    let trait_file_id = def_map.modules[trait_module_id].file_id;
                    self.check_trait_method_with_apply_method(
                        trait_method,
                        trait_file_id,
                        apply_method,
                    );
                }
            }
        }
    }

    fn check_trait_method_with_apply_method(
        &mut self,
        trait_method: &Function,
        trait_file_id: FileId,
        apply_method: &Function,
    ) {
        self.check_trait_method_generic_params_with_apply_method_generic_params(
            &trait_method
                .generic_params
                .file_span_ref(trait_file_id, trait_method.generic_params.span),
            // InFile::new(
            //     Spanned::new(
            //         &trait_method.generic_params,
            //         trait_method.generic_params.span,
            //     ),
            //     trait_file_id,
            // ),
            &apply_method.generic_params,
        );
    }

    fn check_trait_methods_match_apply_methods(
        &mut self,
        trait_methods: &Spanned<Vec<FunctionId>>,
        trait_module_id: ModuleId,
        apply_methods: &Spanned<Vec<FunctionId>>,
    ) {
        let def_map = self.def_map.unwrap();
        let apply_method_names: HashSet<Spur> = apply_methods
            .iter()
            .map(|method_id| {
                def_map.item_trees[self.cur_module_id][*method_id]
                    .name
                    .inner
            })
            .collect();
        let trait_method_names: HashSet<Spur> = trait_methods
            .iter()
            .map(|method_id| def_map.item_trees[trait_module_id][*method_id].name.inner)
            .collect();
        let methods_that_dont_belong: Vec<_> =
            apply_method_names.difference(&trait_method_names).collect();
        let unimplemented_methods: Vec<_> =
            trait_method_names.difference(&apply_method_names).collect();

        if unimplemented_methods.len() > 0 {
            let trait_file_id = def_map[trait_module_id].file_id;
            self.diagnostics.push(
                LowerError::UnimplementedTraitMethods {
                    trait_methods_declared: trait_method_names
                        .iter()
                        .map(|spur| self.string_interner.resolve(spur).to_string())
                        .collect(),
                    trait_methods_declared_file_span: trait_methods.span.in_file(trait_file_id),
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
            let trait_file_id = def_map[trait_module_id].file_id;
            self.diagnostics.push(
                LowerError::MethodsDontBelongInApply {
                    trait_methods_declared: trait_method_names
                        .iter()
                        .map(|spur| self.string_interner.resolve(spur).to_string())
                        .collect(),
                    trait_methods_declared_file_span: trait_methods.span.in_file(trait_file_id),
                    methods_that_dont_belond: methods_that_dont_belong
                        .iter()
                        .map(|spur| self.string_interner.resolve(spur).to_string())
                        .collect(),
                    methods_that_dont_belond_file_span: apply_methods.span.in_file(self.file_id()),
                }
                .to_diagnostic(),
            );
        }
    }

    fn check_trait_method_generic_params_with_apply_method_generic_params(
        &mut self,
        trait_generic_params: &InFile<Spanned<&GenericParams>>,
        apply_generic_params: &Spanned<GenericParams>,
    ) {
        self.check_generic_param_lengths(trait_generic_params, apply_generic_params);

        for trait_where_predicate in &trait_generic_params.where_predicates.0 {
            let apply_where_predicate = apply_generic_params
                .where_predicates
                .0
                .iter()
                .find(|predicate| predicate.ty == trait_where_predicate.ty);

            let trait_trait = self
                .get_trait(&trait_where_predicate.bound)
                .map(|(trt, _)| trt.inner);

            match apply_where_predicate {
                Some(apply_where_predicate) => {
                    let apply_trait = self
                        .get_trait(&apply_where_predicate.bound)
                        .map(|(trt, _)| trt.inner);
                    self.check_if_optional_traits_are_equal(
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

    pub(super) fn check_where_predicates(
        &mut self,
        generic_params: &GenericParams,
        generic_params_file_span: InFile<Span>,
        module_id: ModuleId,
    ) {
        for where_predicate in &generic_params.where_predicates.0 {
            if where_predicate.ty == generic_params.invalid_idx() {
                self.diagnostics.push(
                    LowerError::UnknownGeneric {
                        generic: self
                            .string_interner
                            .resolve(&where_predicate.name)
                            .to_string(),
                        generic_file_span: where_predicate
                            .name
                            .span
                            .in_file(generic_params_file_span.file_id),
                        generic_params: generic_params
                            .types
                            .iter()
                            .map(|(_, param)| self.string_interner.resolve(param).to_string())
                            .collect(),
                        generic_params_file_span: generic_params_file_span.clone(),
                    }
                    .to_diagnostic(),
                );
                continue;
            }

            let trt = self.get_trait(&where_predicate.bound);
            if let Some((trt, _trt_mod_id)) = trt {
                let trt_generic_params = trt.generic_params.clone();

                let generic_args_span = Span::span_iter_of_span(
                    where_predicate
                        .bound
                        .generic_args
                        .iter()
                        .map(|arg| self.types[arg.raw()].span),
                )
                .unwrap_or(where_predicate.bound.span);

                self.check_generic_args_with_parameters(
                    &where_predicate.bound.generic_args,
                    generic_args_span.in_file(generic_params_file_span.file_id),
                    &generic_params.where_predicates.0,
                    &trt_generic_params,
                    trt_generic_params.span.in_file(trt.file_id),
                );

                self.check_where_predicates(
                    &trt_generic_params,
                    trt_generic_params.span.in_file(trt.file_id),
                    module_id,
                );
            }
        }
    }

    fn check_generic_args_with_parameters(
        &mut self,
        generic_args: &[TypeIdx],
        generic_args_file_span: InFile<Span>,
        generic_args_where_predicates: &[WherePredicate],
        trait_def_generic_params: &GenericParams,
        trait_def_generic_params_file_span: InFile<Span>,
    ) {
        let args_len = generic_args.len();
        let params_len = trait_def_generic_params.types.len();
        if args_len != params_len {
            self.diagnostics.push(
                LowerError::IncorrectNumGenericArgsInWherePredicate {
                    expected_num: params_len,
                    expected_num_file_span: trait_def_generic_params_file_span.clone(),
                    got_num: args_len,
                    got_num_file_span: generic_args_file_span.clone(),
                }
                .to_diagnostic(),
            );
        }

        trait_def_generic_params
            .types
            .iter()
            .zip(generic_args.iter())
            .for_each(|((_, param_name), arg)| {
                // for every generic parameter in trait def and argument in trait bound

                // the predicates in the trait def that restrict the current trait def param
                let required_predicates = trait_def_generic_params
                    .where_predicates
                    .0
                    .iter()
                    .filter(|predicate| predicate.name.inner == param_name.inner);

                // for the every predicate on the current trait def param
                for required_predicate in required_predicates {
                    if let Type::Generic(name, _) = &self.types[arg.raw()].inner {
                        // predicate in args that matches the requirement
                        let predicate_matched = generic_args_where_predicates
                            .iter()
                            .filter(|predicate| predicate.name.inner == *name)
                            .find(|predicate| {
                                predicate.bound.segments == required_predicate.bound.segments
                            });

                        if let Some(_predicate_matched) = predicate_matched {
                            // todo!()
                        } else {
                            let arg_span = self.types[arg.raw()].span;
                            self.diagnostics.push(
                                LowerError::GenericArgDoesNotMatchRestriction {
                                    generic: self.string_interner.resolve(&name).to_string(),
                                    generic_file_span: arg_span
                                        .in_file(generic_args_file_span.file_id),
                                    restriction: required_predicate
                                        .bound
                                        .to_string(self.string_interner),
                                    restriction_file_span: required_predicate
                                        .bound
                                        .span
                                        .in_file(trait_def_generic_params_file_span.file_id),
                                }
                                .to_diagnostic(),
                            );
                        }
                    } else {
                        todo!("{:#?} {:#?}", arg, self.types[arg.raw()]);
                    }
                }
            });
    }

    fn check_generic_param_lengths(
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

    fn check_if_optional_traits_are_equal(
        &mut self,
        trait_trait: Option<Trait>,
        apply_trait: Option<Trait>,
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
