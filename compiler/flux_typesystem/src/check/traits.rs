use super::*;

impl TChecker {
    pub fn add_trait_to_context(&mut self, trait_name: Spur) {
        self.trait_implementation_table
            .table
            .insert(trait_name, vec![]);
    }

    fn get_trait_implementation_span(
        &mut self,
        trt: &Spur,
        implementation: &TraitImplementation,
    ) -> InFile<Span> {
        let impltrs = self
            .trait_implementation_table
            .spans
            .get(trt)
            .unwrap_or_else(|| flux_diagnostics::ice("span isn't stored for trait implementation"))
            .clone();

        for (implementation_b, span) in impltrs {
            if self.are_trait_implementations_equal(implementation, &implementation_b) {
                return span;
            }
        }
        flux_diagnostics::ice("span not stored for trait implementation");
    }

    pub fn add_trait_application_to_context(
        &mut self,
        trait_name: &FileSpanned<Spur>,
        trait_args: &[TypeId],
        impltor: TypeId,
    ) -> Result<(), Diagnostic> {
        println!(
            "adding trait application {}{} to {} to context",
            self.string_interner.resolve(&trait_name),
            if trait_args.is_empty() {
                "".to_string()
            } else {
                format!(
                    "<{}>",
                    trait_args
                        .iter()
                        .map(|arg| self.tenv.fmt_ty_id(*arg))
                        .join(", ")
                )
            },
            self.tenv.fmt_ty_id(impltor)
        );
        tracing::trace!(
            "adding trait application {}{} to {} to context",
            self.string_interner.resolve(&trait_name),
            if trait_args.is_empty() {
                "".to_string()
            } else {
                format!(
                    "<{}>",
                    trait_args
                        .iter()
                        .map(|arg| self.tenv.fmt_ty_id(*arg))
                        .join(", ")
                )
            },
            self.tenv.fmt_ty_id(impltor)
        );
        let trait_implementors = self
            .trait_implementation_table
            .table
            .get(trait_name)
            .ok_or_else(|| {
                TypeError::TraitDoesNotExist {
                    trait_name: trait_name
                        .map_inner_ref(|spur| self.tenv.string_interner.resolve(spur).to_string()),
                }
                .to_diagnostic()
            })?
            .clone();

        let impltor_name = self
            .tenv
            .string_interner
            .get_or_intern(&self.tenv.fmt_ty_id_constr(impltor));

        let implementations_for_trait_by_impltr: Vec<_> = trait_implementors
            .iter()
            .filter(|implementor| {
                self.unify(
                    implementor.get_impltor(),
                    impltor,
                    Span::poisoned().in_file(FileId::poisoned()),
                )
                .is_ok()
            })
            .collect();

        let impltor_filespan = self.tenv.get_type_filespan(impltor);
        let impltor_arg_typekinds = self
            .tenv
            .get_entry(impltor)
            .get_params()
            .map(|typekinds| typekinds.to_vec())
            .unwrap_or(vec![]);
        let impltor_args = impltor_arg_typekinds
            .iter()
            .map(|kind| {
                let ty = Type::new(kind.clone());
                self.tenv
                    .insert(ty.file_span(impltor_filespan.file_id, impltor_filespan.inner))
            })
            .collect();

        let trait_implementation =
            TraitImplementation::new(trait_args.to_vec(), impltor, impltor_args);

        for implementation in &implementations_for_trait_by_impltr {
            if self.are_trait_implementations_equal(implementation, &trait_implementation) {
                let implementation_a_span =
                    self.get_trait_implementation_span(trait_name, implementation);
                return Err(TypeError::ConflictingTraitImplementations {
                    trait_name: self
                        .tenv
                        .string_interner
                        .resolve(&trait_name.inner.inner)
                        .to_string(),
                    impltor: self.tenv.string_interner.resolve(&impltor_name).to_string(),
                    implementation_a: implementation_a_span,
                    implementation_b: trait_name.to_filespan(),
                }
                .to_diagnostic());
            }
        }

        self.trait_implementation_table.set_type_span(
            trait_name.inner.inner,
            trait_implementation.clone(),
            trait_name.to_filespan(),
        );
        let trait_implementors = self
            .trait_implementation_table
            .table
            .get_mut(trait_name)
            .unwrap();
        trait_implementors.push(trait_implementation);

        Ok(())
    }

    fn are_trait_implementations_equal(
        &mut self,
        impl_a: &TraitImplementation,
        impl_b: &TraitImplementation,
    ) -> bool {
        if impl_a.get_trait_params().len() != impl_b.get_trait_params().len() {
            return false;
        }
        if impl_a.get_impltor_params().len() != impl_b.get_impltor_params().len() {
            return false;
        }

        if self
            .unify(
                impl_a.get_impltor(),
                impl_b.get_impltor(),
                Span::poisoned().in_file(FileId::poisoned()),
            )
            .is_err()
        {
            return false;
        };

        let num_ok_unifications = impl_a
            .get_trait_params()
            .iter()
            .zip(impl_b.get_trait_params().iter())
            .filter_map(|(a, b)| {
                self.unify(*a, *b, Span::poisoned().in_file(FileId::poisoned()))
                    .ok()
            })
            .count();

        let trait_params_equal = num_ok_unifications == impl_a.get_trait_params().len();

        let num_ok_unifications = impl_a
            .get_impltor_params()
            .iter()
            .zip(impl_b.get_impltor_params().iter())
            .filter_map(|(a, b)| {
                self.unify(*a, *b, Span::poisoned().in_file(FileId::poisoned()))
                    .ok()
            })
            .count();

        let impltor_params_equal = num_ok_unifications == impl_a.get_impltor_params().len();

        trait_params_equal && impltor_params_equal
    }

    fn are_trait_restrictions_equal(&mut self, a: &TraitRestriction, b: &TraitRestriction) -> bool {
        if a.name.inner.inner != b.name.inner.inner {
            return false;
        }

        if a.args.len() != b.args.len() {
            return false;
        }

        if a.args
            .iter()
            .zip(b.args.iter())
            .filter(|(a, b)| {
                self.unify(**a, **b, Span::poisoned().in_file(FileId::poisoned()))
                    .is_ok()
            })
            .count()
            != a.args.len()
        {
            return false;
        }

        true
    }

    fn get_implementations(
        &mut self,
        trt: &FileSpanned<Spur>,
        impltor: TypeId,
    ) -> Result<Vec<TraitImplementation>, Diagnostic> {
        let impls = self
            .trait_implementation_table
            .table
            .get(trt)
            .ok_or_else(|| {
                TypeError::TraitDoesNotExist {
                    trait_name: trt
                        .map_inner_ref(|spur| self.tenv.string_interner.resolve(spur).to_string()),
                }
                .to_diagnostic()
            })?
            .clone();
        let impls = impls
            .iter()
            .filter(|implementation| {
                self.unify(
                    impltor,
                    implementation.get_impltor(),
                    Span::poisoned().in_file(FileId::poisoned()),
                )
                .is_ok()
            })
            .cloned()
            .collect();
        Ok(impls)
    }

    pub(crate) fn check_if_type_implements_restrictions(
        &mut self,
        ty: TypeId,
        restrictions: &[TraitRestriction],
    ) -> Result<(), Diagnostic> {
        println!(
            "HII {}",
            format!(
                "checking if type `{}` implements restrictions: {}",
                self.tenv.fmt_ty_id(ty),
                restrictions
                    .iter()
                    .map(|restriction| format!(
                        "`{}`",
                        self.tenv.fmt_trait_restriction(restriction)
                    ))
                    .join(", ")
            )
        );
        tracing::info!(
            "checking if type `{}` implements restrictions: {}",
            self.tenv.fmt_ty_id(ty),
            restrictions
                .iter()
                .map(|restriction| format!("`{}`", self.tenv.fmt_trait_restriction(restriction)))
                .join(", ")
        );
        let impltr_filespan = self.tenv.get_type_filespan(ty);

        let mut unimplemented_restrictions = vec![];

        if let TypeKind::Generic(_, restrictions_of_type_being_checked) =
            self.tenv.get_typekind_with_id(ty).inner.inner
        {
            if restrictions_of_type_being_checked.len() != restrictions.len() {
                return Err(TypeError::TraitRestrictionsNotMet {
                    ty: self
                        .tenv
                        .fmt_ty_id(ty)
                        .file_span(impltr_filespan.file_id, impltr_filespan.inner),
                    unmet_restrictions: unimplemented_restrictions,
                }
                .to_diagnostic());
            }
            if restrictions_of_type_being_checked
                .iter()
                .zip(restrictions.iter())
                .filter(|(a, b)| self.are_trait_restrictions_equal(a, b))
                .count()
                == restrictions.len()
            {
                return Ok(());
            }
        }

        for restriction in restrictions {
            let implementations_for_ty = self.get_implementations(&restriction.name, ty)?.to_vec();
            let mut found_valid_impl = false;
            for implementation in implementations_for_ty {
                if implementation.get_trait_params().len() == restriction.args.len() {
                    let num_ok_unifications = implementation
                        .get_trait_params()
                        .iter()
                        .zip(restriction.args.iter())
                        .filter_map(|(a, b)| {
                            println!(
                                "unifying: {} {}",
                                self.tenv.fmt_ty_id(*a),
                                self.tenv.fmt_ty_id(*b)
                            );
                            self.unify(*a, *b, Span::poisoned().in_file(FileId::poisoned()))
                                .ok()
                        })
                        .count();
                    if num_ok_unifications == implementation.get_trait_params().len() {
                        found_valid_impl = true;
                    }
                }
            }

            if !found_valid_impl {
                unimplemented_restrictions.push(self.tenv.fmt_trait_restriction(restriction));
            }
        }

        if !unimplemented_restrictions.is_empty() {
            Err(TypeError::TraitRestrictionsNotMet {
                ty: self
                    .tenv
                    .fmt_ty_id(ty)
                    .file_span(impltr_filespan.file_id, impltr_filespan.inner),
                unmet_restrictions: unimplemented_restrictions,
            }
            .to_diagnostic())
        } else {
            tracing::debug!(
                "type `{}` implements restrictions {}",
                self.tenv.fmt_ty_id(ty),
                restrictions
                    .iter()
                    .map(|restriction| format!(
                        "`{}`",
                        self.tenv.fmt_trait_restriction(restriction)
                    ))
                    .join(", ")
            );
            Ok(())
        }
    }
}
