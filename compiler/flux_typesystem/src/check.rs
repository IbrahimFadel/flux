use flux_diagnostics::{Diagnostic, ToDiagnostic};
use flux_span::{FileSpanned, InFile, Span, Spanned, WithSpan};
use hashbrown::HashMap;
use itertools::Itertools;
use lasso::Spur;
use tracing::{info, trace};

use crate::{
    diagnostics::TypeError,
    env::TraitRestriction,
    trait_solver::{TraitImplementation, TraitSolver},
    ConcreteKind, Constraint, TEnv, Type, TypeId, TypeKind,
};

#[derive(Debug)]
pub struct TChecker {
    pub tenv: TEnv,
    trait_solver: TraitSolver,
    constraints: Vec<Constraint>,
}

/*

Trait Name ->
    Root type of implementor ->
        [Implementations]


From ->
    Foo ->
        { trait_params: [i32], impltor_params: [T] },
        { trait_params: [i32], impltor_params: [i32] }
    Bar ->
        { trait_params: [T], impltor_params: [T] }


apply<T> From<i32> to Foo<T> {}
apply From<i32> to Foo<i32> {}
apply<T> From<T> to Bar<T> {}

*/

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

    fn add_new_implementation(
        &mut self,
        trt: Spur,
        trait_params: Vec<TypeId>,
        impltr: TypeId,
        span: InFile<Span>,
    ) -> Result<(), Diagnostic> {
        let entry = self
            .trait_solver
            .implementation_table
            .table
            .entry(trt)
            .or_insert_with(|| HashMap::new());
        let tykind = self.tenv.get_entry(impltr);
        // let name = tykind.inner.inner.

        // let entry = self.trait_solver.implementation_table.table
        //     .entry(trt)
        //     .or_insert_with(|| HashMap::new())
        //     .entry(impltr)
        //     .or_insert_with(|| vec![]);

        // let new_trait_impl = TraitImplementation::new(trait_params, impltr_params);
        // entry
        //     .iter()
        //     .find(|trait_impl| **trait_impl == new_trait_impl)
        //     .cloned()
        //     .and_then(|conflicting_trait_impl| {
        //         //
        //         let span = self.trait_solver.implementation_table.get_type_span(&trt, &impltr, &conflicting_trait_impl);
        //         let diagnostic = TypeError::ConflictingTraitImplementations { implementation_a_file_id: span.file_id, implementation_b_file_id: span.file_id, impl_a_trt: format!("{}{}", self.tenv.string_interner.resolve(&trt), if trait_params.is_empty() {
        //             format!("")
        //         } else {
        //             format!("<{}>", trait_params.iter().map(|id| self.tenv.fmt_ty_id(*id)).join(", "))
        //         }), impl_a_ty: , impl_b_trt: (), impl_b_ty: () }
        //         Some(())
        //     });

        Ok(())
    }

    fn check_if_type_implements_restrictions(
        &self,
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
        for restriction in restrictions {
            let trait_name = restriction.name.inner.inner;
            self.trait_solver
                .implementation_table
                .table
                .get(&trait_name)
                .ok_or_else(|| {
                    TypeError::TraitInTraitRestrictionDoesNotExist {
                        trait_name: self
                            .tenv
                            .string_interner
                            .resolve(&trait_name)
                            .to_string()
                            .in_file(restriction.name.file_id, restriction.name.span),
                    }
                    .to_diagnostic()
                })?;
        }
        Ok(())
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
            (Generic, _) => {
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
                let entry = &self.tenv.get_entry(a);
                self.check_if_type_implements_restrictions(b, &entry.restrictions)?;
                // todo!()
                Ok(())
            }
            (Concrete(ConcreteKind::Path(path)), Int(int_id)) => match int_id {
                Some(int_id) => self.unify(a, *int_id, unification_span),
                None => {
                    if self.tenv.int_paths.get(&path).is_some() {
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
                    if self.tenv.int_paths.get(&path).is_some() {
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
                    if self.tenv.float_paths.get(&path).is_some() {
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
                    if self.tenv.float_paths.get(&path).is_some() {
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

        // let mut a_got_from_list = vec![];
        // let mut a_inner = a;
        // while let TypeKind::Ref(id) = &self.tenv.get_typekind_with_id(a_inner).inner.inner {
        //     a_got_from_list.push(self.tenv.get_type_filespan(*id));
        //     a_inner = *id;
        // }
        // println!("{:?}", self.tenv.get_typekind_with_id(b).inner.inner);
        // let mut b_got_from_list = vec![];
        // let mut b_inner = b;
        // while let TypeKind::Ref(id) = &self.tenv.get_typekind_with_id(b_inner).inner.inner {
        //     b_got_from_list.push(self.tenv.get_type_filespan(*id));
        //     b_inner = *id;
        // }

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
