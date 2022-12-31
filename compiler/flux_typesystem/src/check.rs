use flux_diagnostics::{Diagnostic, ToDiagnostic};
use flux_span::{FileSpanned, InFile, Span, Spanned};
use tracing::trace;

use crate::{diagnostics::TypeError, ConcreteKind, Constraint, TEnv, TypeId, TypeKind};

#[derive(Debug)]
pub struct TChecker {
    pub tenv: TEnv,
    constraints: Vec<Constraint>,
}

impl TChecker {
    pub fn new(tenv: TEnv) -> Self {
        Self {
            tenv,
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
                self.tenv.set_type(a, b_kind.inner.inner);
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
                // let constraints = self.tenv.get_entry(a).constraints;
                todo!()
            }
            (Concrete(ConcreteKind::Path(path)), Int(int_id)) => match int_id {
                Some(int_id) => self.unify(a, *int_id, unification_span),
                None => {
                    if self.tenv.int_paths.get(&path).is_some() {
                        self.tenv.set_type(b, TypeKind::Int(Some(a)));
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
                        self.tenv.set_type(a, TypeKind::Int(Some(b)));
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
                        self.tenv.set_type(b, TypeKind::Float(Some(a)));
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
                        self.tenv.set_type(a, TypeKind::Float(Some(b)));
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
        }
    }
}
