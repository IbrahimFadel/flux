use super::*;

impl TChecker {
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
                    let ty = Type::with_params(b_kind.inner.inner, b_params.iter().cloned());
                    self.tenv.set_type(a, ty);
                } else {
                    let ty = Type::new(b_kind.inner.inner);
                    self.tenv.set_type(a, ty);
                }
                Ok(())
            }
            (Generic(_, restrictions), _) => {
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
                // let restrictions = self.tenv.get_entry(a).restrictions.clone();
                // self.check_if_type_implements_restrictions(b, &restrictions)?;
                Ok(())
            }
            (Concrete(ConcreteKind::Path(path)), Int(int_id)) => match int_id {
                Some(int_id) => self.unify(a, *int_id, unification_span),
                None => {
                    if self.tenv.int_paths.get(path).is_some() {
                        let ty = Type::new(TypeKind::Int(Some(a)));
                        self.tenv.set_type(b, ty);
                        Ok(())
                    } else {
                        Err(self.type_mismatch(a, b, unification_span))
                    }
                }
            },
            (Int(int_id), Concrete(ConcreteKind::Path(path))) => match int_id {
                Some(int_id) => self.unify(*int_id, a, unification_span),
                None => {
                    if self.tenv.int_paths.get(path).is_some() {
                        let ty = Type::new(TypeKind::Int(Some(b)));
                        self.tenv.set_type(a, ty);
                        Ok(())
                    } else {
                        Err(self.type_mismatch(a, b, unification_span))
                    }
                }
            },
            (Concrete(ConcreteKind::Path(path)), Float(float_id)) => match float_id {
                Some(float_id) => self.unify(a, *float_id, unification_span),
                None => {
                    if self.tenv.float_paths.get(path).is_some() {
                        let ty = Type::new(TypeKind::Float(Some(a)));
                        self.tenv.set_type(b, ty);
                        Ok(())
                    } else {
                        Err(self.type_mismatch(a, b, unification_span))
                    }
                }
            },
            (Float(float_id), Concrete(ConcreteKind::Path(path))) => match float_id {
                Some(float_id) => self.unify(*float_id, a, unification_span),
                None => {
                    if self.tenv.float_paths.get(path).is_some() {
                        let ty = Type::new(TypeKind::Float(Some(b)));
                        self.tenv.set_type(a, ty);
                        Ok(())
                    } else {
                        Err(self.type_mismatch(a, b, unification_span))
                    }
                }
            },
            (Concrete(concrete_a), Concrete(concrete_b)) => {
                if concrete_a == concrete_b {
                    Ok(())
                } else {
                    Err(self.type_mismatch(a, b, unification_span))
                }
            }
            (_, _) => Err(self.type_mismatch(a, b, unification_span)),
        }
    }

    fn type_mismatch(&self, a: TypeId, b: TypeId, unification_span: InFile<Span>) -> Diagnostic {
        let a_file_span = self.tenv.get_type_filespan(a);
        let b_file_span = self.tenv.get_type_filespan(b);

        TypeError::TypeMismatch {
            a: self.tenv.fmt_ty_id(a),
            a_file_span,
            b: self.tenv.fmt_ty_id(b),
            b_file_span,
            span: (),
            span_file_span: unification_span,
        }
        .to_diagnostic()
    }
}
