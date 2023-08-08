use crate::hir::WherePredicate;

use super::*;

impl<'a> LowerCtx<'a> {
    /// Check for unknown generics, and verify the restrictions
    /// Restrictions are confirmed to exist, have the correct amounts of generic arguments supplied to them, and those arguments are verified (that they implement all the restrictions on those arguments as defined in the where clause of the trait restriction)
    pub(super) fn verify_where_predicates(
        &mut self,
        generic_params: &GenericParams,
        generic_params_file_span: &InFile<Span>,
    ) {
        for where_predicate in generic_params.where_predicates.iter() {
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

            let trt = if let Some(trt) = self.get_trait(&where_predicate.bound) {
                trt
            } else {
                continue;
            };

            let generic_args_span = Span::span_iter_of_span(
                where_predicate
                    .bound
                    .generic_args
                    .iter()
                    .map(|arg| self.tchk.tenv.get_type_filespan(*arg).inner),
            )
            .unwrap_or(where_predicate.bound.span);
            // let generic_args_span = Span::span_iter_of_span(
            //     where_predicate
            //         .bound
            //         .generic_args
            //         .iter()
            //         .map(|arg| self.tchk.tenv.get_type_filespan(*arg).inner),
            // )
            // .unwrap_or(where_predicate.bound.span);

            self.verify_generic_args_match_trait_definition(
                &where_predicate.bound.generic_args,
                generic_args_span.in_file(generic_params_file_span.file_id),
                &generic_params.where_predicates.0,
                &trt.generic_params,
                trt.generic_params.span.in_file(trt.file_id),
            );

            self.verify_where_predicates(&trt.generic_params, generic_params_file_span)
        }
    }

    /// Verify that the number of supplied generic arguments match the number required for that trait
    /// and that they implement the restrictions on them
    fn verify_generic_args_match_trait_definition(
        &mut self,
        generic_args: &[TypeId],
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
                    let arg_typekind = self.tchk.tenv.get_typekind_with_id(*arg);
                    // arg_typekind.
                    if let TypeKind::Generic(name, _) = &arg_typekind.inner.inner {
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
                            let arg_span = arg_typekind.span;
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
                        todo!("{:#?} {:#?}", arg, arg_typekind);
                    }
                }
            });
    }

    pub(crate) fn path_to_trait_restriction(
        &mut self,
        path: &Spanned<Path>,
    ) -> ts::TraitRestriction {
        let trait_name = path
            .map_ref(|path| path.to_spur(self.string_interner))
            .in_file(self.file_id());
        let (_, trait_id) = self.get_trait_with_id(path).unwrap();
        let trait_id_raw = trait_id.into_raw();
        ts::TraitRestriction {
            trait_id: trait_id_raw.into(),
            trait_name,
            args: path.generic_args.clone(),
        }
    }
}
