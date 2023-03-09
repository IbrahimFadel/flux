use flux_span::Span;
use hashbrown::HashSet;
use itertools::Itertools;
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
            self.handle_apply_trait(trt_path, &a.methods);
        }
        for f in &a.methods.inner {
            self.handle_function(*f, item_tree);
        }
    }

    fn handle_apply_trait(&mut self, trt_path: &Spanned<Path>, methods: &Spanned<Vec<FunctionId>>) {
        let trt = self
            .def_map
            .unwrap()
            .resolve_path(trt_path, self.cur_module_id);
        match trt {
            Ok(per_ns) => match per_ns.types {
                Some((def_id, m, vis)) => {
                    let item_tree = &self.def_map.unwrap().item_trees[m];
                    let file_id = self.def_map.unwrap().modules[m].file_id;
                    let trt = match def_id {
                        ModuleDefId::TraitId(trt_idx) => {
                            let trt = &item_tree[trt_idx];
                            if self.cur_module_id != m && vis == Visibility::Private {
                                self.diagnostics.push(
                                    LowerError::TriedApplyingPrivateTrait {
                                        trt: self
                                            .string_interner
                                            .resolve(&trt.name.inner)
                                            .to_string(),
                                        declared_as_private: (),
                                        declared_as_private_file_span: trt
                                            .visibility
                                            .span
                                            .in_file(file_id),
                                        application: (),
                                        application_file_span: trt_path
                                            .span
                                            .in_file(self.file_id()),
                                    }
                                    .to_diagnostic(),
                                );
                            }
                            trt
                        }
                        _ => unreachable!(),
                    };

                    self.check_trait_methods_with_apply_methods(&trt.methods, m, methods);
                }
                None => self.diagnostics.push(
                    LowerError::UnresolvedTrait {
                        trt: trt_path.inner.to_string(self.string_interner),
                        trt_file_span: trt_path.span.in_file(self.file_id()),
                    }
                    .to_diagnostic(),
                ),
            },
            Err(err) => self.diagnostics.push(
                err.to_lower_error(self.file_id(), self.string_interner)
                    .to_diagnostic(),
            ),
        }
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
                        trait_module_id,
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
        trait_module_id: ModuleId,
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
            trait_module_id,
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
                        .sorted()
                        .collect::<Vec<_>>(),
                    trait_methods_declared_file_span: trait_methods.span.in_file(trait_file_id),
                    unimplemented_methods: unimplemented_methods
                        .iter()
                        .map(|spur| self.string_interner.resolve(spur).to_string())
                        .sorted()
                        .collect::<Vec<_>>(),
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
                        .sorted()
                        .collect::<Vec<_>>(),
                    trait_methods_declared_file_span: trait_methods.span.in_file(trait_file_id),
                    methods_that_dont_belond: methods_that_dont_belong
                        .iter()
                        .map(|spur| self.string_interner.resolve(spur).to_string())
                        .sorted()
                        .collect::<Vec<_>>(),
                    methods_that_dont_belond_file_span: apply_methods.span.in_file(self.file_id()),
                }
                .to_diagnostic(),
            );
        }
    }

    fn check_trait_method_generic_params_with_apply_method_generic_params(
        &mut self,
        trait_generic_params: &InFile<Spanned<&GenericParams>>,
        trait_module_id: ModuleId,
        apply_generic_params: &Spanned<GenericParams>,
    ) {
        // self.check_where_predicates(
        //     &trait_generic_params,
        //     trait_generic_params.to_filespan(),
        //     trait_module_id,
        // );
        // self.check_where_predicates(
        //     &apply_generic_params,
        //     apply_generic_params.span.in_file(self.file_id()),
        //     self.cur_module_id,
        // );
        self.check_generic_param_lengths(trait_generic_params, apply_generic_params);

        let def_map = self.def_map.unwrap();

        for trait_where_predicate in &trait_generic_params.where_predicates.0 {
            let apply_where_predicate = apply_generic_params
                .where_predicates
                .0
                .iter()
                .find(|predicate| predicate.ty == trait_where_predicate.ty);

            // let trait_trait = self.get_trait(&trait_where_predicate.bound, trait_module_id);
            // let trait_trait = match trait_trait {
            //     Err(err) => {
            //         self.diagnostics.push(err);
            //         None
            //     }
            //     Ok(res) => res,
            // };
            // // let trait_trait = trait_trait.map(|(trt, _)| trt);
            let trait_trait = def_map.resolve_path(&trait_where_predicate.bound, trait_module_id);
            let trait_trait = match trait_trait {
                Err(_) => {
                    self.diagnostics.push(
                        LowerError::UnresolvedTrait {
                            trt: trait_where_predicate.bound.to_string(self.string_interner),
                            trt_file_span: trait_where_predicate.bound.span.in_file(self.file_id()),
                        }
                        .to_diagnostic(),
                    );
                    None
                }
                Ok(res) => res.types.map(|(def_id, mod_id, _)| {
                    let item_tree = &self.def_map.unwrap().item_trees[mod_id];
                    match def_id {
                        crate::ModuleDefId::TraitId(t) => &item_tree[t],
                        _ => todo!(),
                    }
                }),
            };

            match apply_where_predicate {
                Some(apply_where_predicate) => {
                    // let apply_trait = self.get_trait(&apply_where_predicate.bound, trait_module_id);
                    // let apply_trait = match apply_trait {
                    //     Err(err) => {
                    //         // self.diagnostics.push(err);
                    //         None
                    //     }
                    //     Ok(res) => res,
                    // };
                    // let apply_trait = apply_trait.map(|(trt, _)| trt);
                    // .unwrap_or_else(|err| {
                    //     self.diagnostics.push(err);
                    //     None
                    // })
                    // .map(|(trt, _)| trt);
                    let apply_trait = def_map
                        .resolve_path(&apply_where_predicate.bound, trait_module_id)
                        .map_or_else(
                            |_| {
                                todo!();
                            },
                            |res| {
                                res.types.map(|(def_id, mod_id, _)| {
                                    let item_tree = &self.def_map.unwrap().item_trees[mod_id];
                                    match def_id {
                                        crate::ModuleDefId::TraitId(t) => &item_tree[t],
                                        _ => todo!(),
                                    }
                                })
                            },
                        );

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

            let trt = self.get_trait(&where_predicate.bound, module_id);
            let trt = match trt {
                Err(err) => {
                    self.diagnostics.push(err);
                    None
                }
                Ok(res) => res,
            };
            if let Some((trt, file_id)) = trt {
                let trt_generic_params = trt.generic_params.clone();

                let generic_args_span =
                    Span::span_iter_of_spanned(where_predicate.bound.generic_args.iter())
                        .unwrap_or(where_predicate.bound.span);

                self.check_generic_args_with_parameters(
                    &where_predicate.bound.generic_args,
                    generic_args_span.in_file(generic_params_file_span.file_id),
                    &generic_params.where_predicates.0,
                    &trt_generic_params,
                    trt_generic_params.span.in_file(file_id),
                );

                self.check_where_predicates(
                    &trt_generic_params,
                    trt_generic_params.span.in_file(file_id),
                    module_id,
                );
            }
        }
    }

    fn check_generic_args_with_parameters(
        &mut self,
        generic_args: &[Spanned<TypeIdx>],
        generic_args_file_span: InFile<Span>,
        generic_args_where_predicates: &[WherePredicate],
        trait_def_generic_params: &GenericParams,
        trait_def_generic_params_file_span: InFile<Span>,
    ) {
        let args_len = generic_args.len();
        let params_len = trait_def_generic_params.types.len();
        println!("{args_len} {params_len}");
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
                    if let Type::Generic(name) = *self.type_interner.resolve(arg.inner) {
                        // predicate in args that matches the requirement
                        let predicate_matched = generic_args_where_predicates
                            .iter()
                            .filter(|predicate| predicate.name.inner == name)
                            .find(|predicate| {
                                predicate.bound.segments == required_predicate.bound.segments
                            });

                        if let Some(predicate_matched) = predicate_matched {
                            // todo!()
                        } else {
                            self.diagnostics.push(
                                LowerError::GenericArgDoesNotMatchRestriction {
                                    generic: self.string_interner.resolve(&name).to_string(),
                                    generic_file_span: arg
                                        .span
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
                        todo!()
                    }
                }
            });
    }

    fn get_trait(
        &self,
        path: &Spanned<Path>,
        module_id: ModuleId,
    ) -> Result<Option<(&Trait, FileId)>, Diagnostic> {
        let def_map = self.def_map.unwrap();
        let trt = def_map.resolve_path(&path, module_id);
        trt.map(|res| {
            res.types.map(|(def_id, mod_id, _)| {
                let item_tree = &self.def_map.unwrap().item_trees[mod_id];
                let file_id = self.def_map.unwrap()[mod_id].file_id;
                let trt = match def_id {
                    crate::ModuleDefId::TraitId(t) => &item_tree[t],
                    _ => todo!(),
                };
                (trt, file_id)
            })
        })
        .map_err(|_| {
            LowerError::UnresolvedTrait {
                trt: path.to_string(self.string_interner),
                trt_file_span: path.span.in_file(self.file_id()),
            }
            .to_diagnostic()
        })
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

    // pub(super) fn check_if_where_predicate_types_exist(
    //     &mut self,
    //     generic_params: &InFile<Spanned<&GenericParams>>,
    // ) {
    //     for where_predicate in &generic_params.where_predicates.0 {
    //         if where_predicate.ty == generic_params.invalid_idx() {
    //             self.diagnostics.push(
    //                 LowerError::UnknownGeneric {
    //                     generic: self
    //                         .string_interner
    //                         .resolve(&where_predicate.name)
    //                         .to_string(),
    //                     generic_file_span: where_predicate
    //                         .name
    //                         .span
    //                         .in_file(generic_params.file_id),
    //                     generic_params: generic_params
    //                         .types
    //                         .iter()
    //                         .map(|(_, param)| self.string_interner.resolve(param).to_string())
    //                         .collect(),
    //                     generic_params_file_span: generic_params
    //                         .inner
    //                         .span
    //                         .in_file(generic_params.file_id),
    //                 }
    //                 .to_diagnostic(),
    //             );
    //         }
    //     }
    // }

    fn check_if_optional_traits_are_equal(
        &mut self,
        trait_trait: Option<&Trait>,
        apply_trait: Option<&Trait>,
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
