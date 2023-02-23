use flux_diagnostics::{Diagnostic, ToDiagnostic};
use flux_span::{FileId, FileSpanned, InFile, Span, Spanned, WithSpan};
use itertools::Itertools;
use lasso::Spur;
use tracing::{info, trace};

use crate::{
    diagnostics::TypeError,
    env::TraitRestriction,
    trait_solver::{TraitImplementation, TraitSolver},
    ConcreteKind, Constraint, TEnv, Type, TypeId, TypeKind,
};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ExpectedPathType {
    Any,
    Local,
    Variable,
    Function,
}

#[derive(Debug)]
pub struct TChecker {
    pub tenv: TEnv,
    trait_solver: TraitSolver,
    constraints: Vec<Constraint>,
}

impl TChecker {
    pub fn new(tenv: TEnv) -> Self {
        Self {
            tenv,
            trait_solver: TraitSolver::new(),
            constraints: vec![],
        }
    }

    pub fn add_constraint(&mut self, constraint: Constraint) {
        trace!("adding constraint {}", self.fmt_constraint(&constraint));
        self.constraints.push(constraint);
    }

    fn fmt_constraint(&self, constraint: &Constraint) -> String {
        match constraint {
            Constraint::TypeEq(a, b) => format!("{a} == {b}"),
            Constraint::FieldAccess {
                struct_ty,
                field,
                field_ty,
            } => format!(
                "{struct_ty} has field with name {} of type {field_ty}",
                self.tenv.string_interner.resolve(field)
            ),
        }
    }

    pub fn add_trait_to_context(&mut self, trait_name: Spur) {
        self.trait_solver
            .implementation_table
            .table
            .insert(trait_name, vec![]);
    }

    fn get_trait_implementation_span(
        &mut self,
        trt: &Spur,
        implementation: &TraitImplementation,
    ) -> InFile<Span> {
        let impltrs = self
            .trait_solver
            .implementation_table
            .spans
            .get(trt)
            .expect("internal compiler error: span isn't stored for trait implementation")
            .clone();

        for (implementation_b, span) in impltrs {
            if self.are_trait_implementations_equal(implementation, &implementation_b) {
                return span;
            }
        }
        panic!("internal compiler error: span not stored for trait implementation")
    }

    pub fn add_trait_application_to_context(
        &mut self,
        trait_name: &FileSpanned<Spur>,
        trait_args: &[TypeId],
        impltor: TypeId,
    ) -> Result<(), Diagnostic> {
        let trait_implementors = self
            .trait_solver
            .implementation_table
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
        let impltor_arg_keys = self
            .tenv
            .get_entry(impltor)
            .get_params()
            .map(|keys| keys.to_vec())
            .unwrap_or(vec![]);
        let impltor_args = impltor_arg_keys
            .iter()
            .map(|key| {
                let kind = self.tenv.type_interner.resolve(*key);
                let ty = Type::new(kind.clone(), &mut self.tenv.type_interner);
                self.tenv
                    .insert(ty.in_file(impltor_filespan.file_id, impltor_filespan.inner))
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

        self.trait_solver.implementation_table.set_type_span(
            trait_name.inner.inner,
            trait_implementation.clone(),
            trait_name.to_filespan(),
        );
        let trait_implementors = self
            .trait_solver
            .implementation_table
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

    fn get_implementations(
        &mut self,
        trt: &FileSpanned<Spur>,
        impltor: TypeId,
    ) -> Result<Vec<TraitImplementation>, Diagnostic> {
        let impls = self
            .trait_solver
            .implementation_table
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

    fn check_if_type_implements_restrictions(
        &mut self,
        ty: TypeId,
        restrictions: &[TraitRestriction],
    ) -> Result<(), Diagnostic> {
        info!(
            "checking if type `{}` implements restrictions: {}",
            self.tenv.fmt_ty_id(ty),
            restrictions
                .iter()
                .map(|restriction| format!("`{}`", self.tenv.fmt_trait_restriction(restriction)))
                .join(", ")
        );
        let impltr_filespan = self.tenv.get_type_filespan(ty);

        let mut unimplemented_restrictions = vec![];

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
                    .in_file(impltr_filespan.file_id, impltr_filespan.inner),
                unmet_restrictions: unimplemented_restrictions,
            }
            .to_diagnostic())
        } else {
            Ok(())
        }
    }

    pub fn unify(
        &mut self,
        a: TypeId,
        b: TypeId,
        unification_span: InFile<Span>,
    ) -> Result<(), Diagnostic> {
        use TypeKind::*;
        let a_kind = self.tenv.get_typekind_with_id(a);
        let b_kind = self.tenv.get_typekind_with_id(b);
        match (&a_kind.inner.inner, &b_kind.inner.inner) {
            (Unknown, _) => {
                let b_entry = &self.tenv.get_entry(b).inner.inner.clone();
                if let Some(b_params) = b_entry.get_params() {
                    let ty = Type::with_params_as_keys(
                        b_kind.inner.inner,
                        b_params.iter().cloned(),
                        &mut self.tenv.type_interner,
                    );
                    self.tenv.set_type(a, ty);
                } else {
                    let ty = Type::new(b_kind.inner.inner, &mut self.tenv.type_interner);
                    self.tenv.set_type(a, ty);
                }
                Ok(())
            }
            (Generic(_), _) => {
                /*
                fn foo<T>(x T) {
                }
                foo(0); // Ok

                fn foo<T: Foo>(x T) {
                }
                foo(0); // Check
                Does int implement Foo?
                Subtyping:
                -   Does sN implement Foo?
                -   Does uN implement Foo?
                */
                let restrictions = self.tenv.get_entry(a).restrictions.clone();
                self.check_if_type_implements_restrictions(b, &restrictions)?;
                // todo!()
                Ok(())
            }
            (Concrete(ConcreteKind::Path(path)), Int(int_id)) => match int_id {
                Some(int_id) => self.unify(a, *int_id, unification_span),
                None => {
                    if self.tenv.int_paths.get(path).is_some() {
                        let ty = Type::new(TypeKind::Int(Some(a)), &mut self.tenv.type_interner);
                        self.tenv.set_type(b, ty);
                        Ok(())
                    } else {
                        Err(self.type_mismatch(a, b, unification_span).to_diagnostic())
                    }
                }
            },
            (Int(int_id), Concrete(ConcreteKind::Path(path))) => match int_id {
                Some(int_id) => self.unify(*int_id, a, unification_span),
                None => {
                    if self.tenv.int_paths.get(path).is_some() {
                        let ty = Type::new(TypeKind::Int(Some(b)), &mut self.tenv.type_interner);
                        self.tenv.set_type(a, ty);
                        Ok(())
                    } else {
                        Err(self.type_mismatch(a, b, unification_span).to_diagnostic())
                    }
                }
            },
            (Concrete(ConcreteKind::Path(path)), Float(float_id)) => match float_id {
                Some(float_id) => self.unify(a, *float_id, unification_span),
                None => {
                    if self.tenv.float_paths.get(path).is_some() {
                        let ty = Type::new(TypeKind::Float(Some(a)), &mut self.tenv.type_interner);
                        self.tenv.set_type(b, ty);
                        Ok(())
                    } else {
                        Err(self.type_mismatch(a, b, unification_span).to_diagnostic())
                    }
                }
            },
            (Float(float_id), Concrete(ConcreteKind::Path(path))) => match float_id {
                Some(float_id) => self.unify(*float_id, a, unification_span),
                None => {
                    if self.tenv.float_paths.get(path).is_some() {
                        let ty = Type::new(TypeKind::Float(Some(b)), &mut self.tenv.type_interner);
                        self.tenv.set_type(a, ty);
                        Ok(())
                    } else {
                        Err(self.type_mismatch(a, b, unification_span).to_diagnostic())
                    }
                }
            },
            (Concrete(concrete_a), Concrete(concrete_b)) => {
                if concrete_a == concrete_b {
                    Ok(())
                } else {
                    Err(self.type_mismatch(a, b, unification_span).to_diagnostic())
                }
            }
            (_, _) => Err(self.type_mismatch(a, b, unification_span).to_diagnostic()),
        }
    }

    fn type_mismatch(&self, a: TypeId, b: TypeId, unification_span: InFile<Span>) -> TypeError {
        let a_span = self.tenv.get_type_filespan(a);
        let b_span = self.tenv.get_type_filespan(b);

        TypeError::TypeMismatch {
            a: FileSpanned::new(
                Spanned::new(self.tenv.fmt_ty_id(a), a_span.inner),
                a_span.file_id,
            ),
            b: FileSpanned::new(
                Spanned::new(self.tenv.fmt_ty_id(b), b_span.inner),
                b_span.file_id,
            ),
            span: unification_span,
            // a_got_from_list,
            // b_got_from_list,
        }
    }
}
