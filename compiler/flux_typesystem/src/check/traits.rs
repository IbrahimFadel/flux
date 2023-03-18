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
                    trait_name: self.string_interner.resolve(&trait_name).to_string(),
                    trait_name_file_span: trait_name.to_filespan(),
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
                    implementation_a: (),
                    implementation_a_file_span: implementation_a_span,
                    implementation_b: (),
                    implementation_b_file_span: trait_name.to_filespan(),
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
                    trait_name: self.string_interner.resolve(&trt).to_string(),
                    trait_name_file_span: trt.to_filespan(),
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

    pub fn check_if_type_implements_restrictions(
        &mut self,
        ty: TypeId,
        restrictions: &[TraitRestriction],
    ) -> Result<(), Diagnostic> {
        let impltr_filespan = self.tenv.get_type_filespan(ty);

        let mut unimplemented_restrictions = vec![];

        let kind = self.tenv.get_typekind_with_id(ty);
        match &kind.inner.inner {
            TypeKind::Generic(_, restrictions_of_type_being_checked) => {
                if restrictions_of_type_being_checked.len() != restrictions.len() {
                    return Err(TypeError::TraitRestrictionsNotMet {
                        ty: self.tenv.fmt_ty_id(ty),
                        ty_file_span: impltr_filespan,
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
            TypeKind::Int(depends_on) => {
                if let Some(depends_on) = depends_on {
                    return self.check_if_type_implements_restrictions(*depends_on, restrictions);
                } else {
                    self.check_if_int_implements_restrictions(kind.to_filespan(), restrictions)?;
                }
            }
            _ => {}
        }

        for restriction in restrictions {
            if !self.does_type_implements_restrictions(ty, restriction)?.0 {
                unimplemented_restrictions.push(
                    self.tenv
                        .fmt_trait_restriction(restriction)
                        .file_span(restriction.name.file_id, restriction.name.span),
                );
            }
        }

        if !unimplemented_restrictions.is_empty() {
            tracing::info!(
                "type `{}` does not implement restrictions: {}",
                self.tenv.fmt_ty_id(ty),
                unimplemented_restrictions
                    .iter()
                    .map(|restriction| format!("`{}`", restriction.inner.inner))
                    .join(", ")
            );
            Err(TypeError::TraitRestrictionsNotMet {
                ty: self.tenv.fmt_ty_id(ty),
                ty_file_span: impltr_filespan,
                unmet_restrictions: unimplemented_restrictions,
            }
            .to_diagnostic())
        } else {
            Ok(())
        }
    }

    fn does_type_implements_restrictions(
        &mut self,
        ty: TypeId,
        restriction: &TraitRestriction,
    ) -> Result<(bool, InFile<Span>), Diagnostic> {
        tracing::debug!(
            "checking if type `{}` implements restriction: {}",
            self.tenv.fmt_ty_id(ty),
            format!("`{}`", self.tenv.fmt_trait_restriction(restriction))
        );
        let implementations_for_ty = self.get_implementations(&restriction.name, ty)?.to_vec();
        let mut valid_impl = None;
        for implementation in implementations_for_ty {
            if implementation.get_trait_params().len() == restriction.args.len() {
                let num_ok_unifications = implementation
                    .get_trait_params()
                    .iter()
                    .zip(restriction.args.iter())
                    .filter_map(|(a, b)| {
                        self.unify(*a, *b, Span::poisoned().in_file(FileId::poisoned()))
                            .ok()
                    })
                    .count();
                if num_ok_unifications == implementation.get_trait_params().len() {
                    valid_impl = Some((
                        true,
                        self.tenv.get_type_filespan(implementation.get_impltor()),
                    ));
                }
            }
        }

        tracing::debug!(
            "type `{}`{} restrictions {}",
            self.tenv.fmt_ty_id(ty),
            if valid_impl.is_some() {
                " implements".to_string()
            } else {
                format!(" does not implement")
            },
            self.tenv.fmt_trait_restriction(restriction)
        );
        Ok(valid_impl.unwrap_or((false, Span::poisoned().in_file(FileId::poisoned()))))
    }

    fn check_if_int_implements_restrictions(
        &mut self,
        file_span: InFile<Span>,
        restrictions: &[TraitRestriction],
    ) -> Result<(), Diagnostic> {
        let s64 = self.tenv.insert(
            Type::new(TypeKind::Concrete(ConcreteKind::Path(
                self.string_interner.get_or_intern_static("s64"),
            )))
            .file_span(file_span.file_id, file_span.inner),
        );
        let s32 = self.tenv.insert(
            Type::new(TypeKind::Concrete(ConcreteKind::Path(
                self.string_interner.get_or_intern_static("s32"),
            )))
            .file_span(file_span.file_id, file_span.inner),
        );
        let s16 = self.tenv.insert(
            Type::new(TypeKind::Concrete(ConcreteKind::Path(
                self.string_interner.get_or_intern_static("s16"),
            )))
            .file_span(file_span.file_id, file_span.inner),
        );
        let s8 = self.tenv.insert(
            Type::new(TypeKind::Concrete(ConcreteKind::Path(
                self.string_interner.get_or_intern_static("s8"),
            )))
            .file_span(file_span.file_id, file_span.inner),
        );
        let u64 = self.tenv.insert(
            Type::new(TypeKind::Concrete(ConcreteKind::Path(
                self.string_interner.get_or_intern_static("u64"),
            )))
            .file_span(file_span.file_id, file_span.inner),
        );
        let u32 = self.tenv.insert(
            Type::new(TypeKind::Concrete(ConcreteKind::Path(
                self.string_interner.get_or_intern_static("u32"),
            )))
            .file_span(file_span.file_id, file_span.inner),
        );
        let u16 = self.tenv.insert(
            Type::new(TypeKind::Concrete(ConcreteKind::Path(
                self.string_interner.get_or_intern_static("u16"),
            )))
            .file_span(file_span.file_id, file_span.inner),
        );
        let u8 = self.tenv.insert(
            Type::new(TypeKind::Concrete(ConcreteKind::Path(
                self.string_interner.get_or_intern_static("u8"),
            )))
            .file_span(file_span.file_id, file_span.inner),
        );
        let mut unmet_restrictions = vec![];
        for restriction in restrictions {
            let does_s64_impl = self.does_type_implements_restrictions(s64, restriction)?;
            let does_s32_impl = self.does_type_implements_restrictions(s32, restriction)?;
            let does_s16_impl = self.does_type_implements_restrictions(s16, restriction)?;
            let does_s8_impl = self.does_type_implements_restrictions(s8, restriction)?;
            let does_u64_impl = self.does_type_implements_restrictions(u64, restriction)?;
            let does_u32_impl = self.does_type_implements_restrictions(u32, restriction)?;
            let does_u16_impl = self.does_type_implements_restrictions(u16, restriction)?;
            let does_u8_impl = self.does_type_implements_restrictions(u8, restriction)?;
            let impltors: Vec<_> = [
                (does_s64_impl, s64),
                (does_s32_impl, s32),
                (does_s16_impl, s16),
                (does_s8_impl, s8),
                (does_u64_impl, u64),
                (does_u32_impl, u32),
                (does_u16_impl, u16),
                (does_u8_impl, u8),
            ]
            .into_iter()
            .filter(|((b, _), _)| *b)
            .map(|((_, file_span), id)| {
                self.tenv
                    .fmt_ty_id(id)
                    .file_span(file_span.file_id, file_span.inner)
            })
            .collect();

            if impltors.len() > 1 {
                return Err(TypeError::MultiplePossibleIntSpecializations {
                    int_types: impltors.iter().map(|s| s.inner.inner.clone()).collect(),
                    int_types_file_span: file_span,
                    trt: self.string_interner.resolve(&restriction.name).to_string(),
                    trt_file_span: restriction.name.to_filespan(),
                    applications: impltors,
                }
                .to_diagnostic());
            } else if impltors.is_empty() {
                unmet_restrictions.push(
                    self.tenv
                        .fmt_trait_restriction(restriction)
                        .file_span(restriction.name.file_id, restriction.name.span),
                );
            }
        }
        if !unmet_restrictions.is_empty() {
            Err(TypeError::TraitRestrictionsNotMet {
                ty: "int".to_string(),
                ty_file_span: file_span,
                unmet_restrictions,
            }
            .to_diagnostic())
        } else {
            Ok(())
        }
    }
}
