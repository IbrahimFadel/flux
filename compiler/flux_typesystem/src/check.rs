use flux_diagnostics::{ice, Diagnostic, ToDiagnostic};
use flux_span::{FileId, InFile, Span, WithSpan};

use crate::{diagnostics::TypeError, env::TEnv, ConcreteKind, TraitId, TypeId, TypeKind};

#[derive(Debug)]
pub struct TChecker<'tenv> {
    pub tenv: &'tenv mut TEnv,
}

impl<'tenv> TChecker<'tenv> {
    pub fn new(tenv: &'tenv mut TEnv) -> Self {
        Self { tenv }
    }
}

impl<'tenv> TChecker<'tenv> {
    pub fn type_implements_trait(&mut self, tid: TypeId, trid: &TraitId) -> bool {
        let poisoned_filespan = Span::poisoned().in_file(unsafe { FileId::poisoned() });
        match self.tenv.trait_applications.get(trid) {
            Some(applications) => {
                let valid_impls: Vec<_> = applications
                    .clone()
                    .iter()
                    .map(|application| {
                        println!(
                            "{} and {}",
                            self.tenv.fmt_tid(&tid),
                            self.tenv.fmt_tid(&application.tid)
                        );
                        self.unify(tid, application.tid, poisoned_filespan.clone())
                            .ok()
                            .map(|_| application.tid)
                    })
                    .flatten()
                    .collect();
                match valid_impls.len() {
                    0 => false,
                    1 => true,
                    2.. => ice("too lazy to do this"),
                }
            }
            None => false,
        }
    }

    fn is_int_supertype(&self, tid: TypeId) -> bool {
        match &self.tenv.get(&tid).inner.inner {
            TypeKind::Ref(tid) => self.is_int_supertype(*tid),
            TypeKind::Int(_) => true,
            _ => false,
        }
    }

    fn is_int_subtype(&self, tid: TypeId) -> bool {
        match &self.tenv.get(&tid).inner.inner {
            TypeKind::Ref(tid) => self.is_int_subtype(*tid),
            TypeKind::Concrete(ConcreteKind::Path(path)) => path.is_int(self.tenv.interner),
            _ => false,
        }
    }

    pub fn unify(
        &mut self,
        a: TypeId,
        b: TypeId,
        unification_span: InFile<Span>, // What area caused the unification
    ) -> Result<(), Diagnostic> {
        use crate::r#type::TypeKind::*;
        let a_kind = self.tenv.get(&a);
        let b_kind = self.tenv.get(&b);
        match (&a_kind.inner.inner, &b_kind.inner.inner) {
            (Ref(a), _) => self.unify(*a, b, unification_span)?,
            (_, Ref(b)) => self.unify(a, *b, unification_span)?,
            (Unknown, _) => {
                self.tenv.set(a, b_kind.inner.inner.clone());
            }
            (_, Unknown) => {
                self.tenv.set(b, a_kind.inner.inner.clone());
            }
            (Concrete(a_concrete), Concrete(b_concrete)) => {
                use ConcreteKind::*;
                match (a_concrete, b_concrete) {
                    (Array(a_arr, n_a), Array(b_arr, n_b)) => {
                        let (n_a, n_b) = (*n_a, *n_b);
                        self.unify(*a_arr, *b_arr, unification_span.clone())?;
                        if n_a != n_b {
                            return Err(self.type_mismatch(&a, &b, unification_span));
                        }
                    }
                    (Path(a_path), Path(b_path)) => {
                        if a_path.segments != b_path.segments {
                            return Err(self.type_mismatch(&a, &b, unification_span));
                        }
                        if a_path.generic_args.len() != b_path.generic_args.len() {
                            return Err(self.type_mismatch(&a, &b, unification_span));
                        }
                        a_path
                            .generic_args
                            .clone()
                            .iter()
                            .zip(b_path.generic_args.clone().iter())
                            .map(|(a_inner, b_inner)| {
                                self.unify(*a_inner, *b_inner, unification_span.clone())
                            })
                            .collect::<Result<(), _>>()?
                    }
                    (Ptr(a_ptr), Ptr(b_ptr)) => self.unify(*a_ptr, *b_ptr, unification_span)?,
                    (Tuple(a_tup), Tuple(b_tup)) => a_tup
                        .clone()
                        .iter()
                        .zip(b_tup.clone().iter())
                        .map(|(a_inner, b_inner)| {
                            self.unify(*a_inner, *b_inner, unification_span.clone())
                        })
                        .collect::<Result<(), _>>()?,
                    _ => return Err(self.type_mismatch(&a, &b, unification_span)),
                }
            }
            (Int(a_id), Int(b_id)) => match (a_id, b_id) {
                (None, None) => {}
                (None, Some(b_id)) => {
                    self.tenv.set(a, TypeKind::Int(Some(*b_id)));
                }
                (Some(a_id), None) => {
                    self.tenv.set(b, TypeKind::Int(Some(*a_id)));
                }
                (Some(a_id), Some(b_id)) => self.unify(*a_id, *b_id, unification_span)?,
            },
            (Int(int_id), Generic(generic)) => {
                // Check all int types if they fit the description, choose default int type if there's more than one
            }
            (Int(int_id), Concrete(ConcreteKind::Path(path))) => match int_id {
                Some(id) => self.unify(*id, b, unification_span)?,
                None => {
                    if path.is_int(self.tenv.interner) {
                        self.tenv.set(a, TypeKind::Int(Some(b)));
                    } else {
                        return Err(self.type_mismatch(&a, &b, unification_span));
                    }
                }
            },
            (Concrete(ConcreteKind::Path(path)), Int(int_id)) => match int_id {
                Some(id) => self.unify(a, *id, unification_span)?,
                None => {
                    if path.is_int(self.tenv.interner) {
                        self.tenv.set(b, TypeKind::Int(Some(a)));
                    } else {
                        return Err(self.type_mismatch(&a, &b, unification_span));
                    }
                }
            },
            (Float(a_id), Float(b_id)) => match (a_id, b_id) {
                (None, None) => {}
                (None, Some(b_id)) => {
                    self.tenv.set(a, TypeKind::Float(Some(*b_id)));
                }
                (Some(a_id), None) => {
                    self.tenv.set(b, TypeKind::Float(Some(*a_id)));
                }
                (Some(a_id), Some(b_id)) => self.unify(*a_id, *b_id, unification_span)?,
            },
            (Float(float_id), Concrete(ConcreteKind::Path(path))) => match float_id {
                Some(id) => self.unify(*id, b, unification_span)?,
                None => {
                    if path.is_float(self.tenv.interner) {
                        self.tenv.set(a, TypeKind::Float(Some(b)));
                    } else {
                        return Err(self.type_mismatch(&a, &b, unification_span));
                    }
                }
            },
            (Concrete(ConcreteKind::Path(path)), Float(float_id)) => match float_id {
                Some(id) => self.unify(a, *id, unification_span)?,
                None => {
                    if path.is_int(self.tenv.interner) {
                        self.tenv.set(b, TypeKind::Float(Some(a)));
                    } else {
                        return Err(self.type_mismatch(&a, &b, unification_span));
                    }
                }
            },
            // (ThisPath(crate::r#type::ThisPath { segments }), _) => {
            // let app = self.tenv.get_application(trid, aid);
            // let name = &segments[0];
            // let assoc_type =
            //     app.assoc_types
            //         .iter()
            //         .find_map(|(aname, tid)| if aname == name { Some(tid) } else { None });
            // match assoc_type {
            //     Some(assoc_tid) => self.unify(*assoc_tid, b, unification_span)?,
            //     None => return Err(self.type_mismatch(&a, &b, unification_span)),
            // }
            // }
            (_, _) => return Err(self.type_mismatch(&a, &b, unification_span)),
        }
        Ok(())
    }

    fn type_mismatch(&self, a: &TypeId, b: &TypeId, unification_span: InFile<Span>) -> Diagnostic {
        let a_tkind = self.tenv.get(&a);
        let b_tkind = self.tenv.get(&b);
        TypeError::TypeMismatch {
            a: self.tenv.fmt_tid(a),
            a_file_span: a_tkind.to_file_span(),
            b: self.tenv.fmt_tid(b),
            b_file_span: b_tkind.to_file_span(),
            span: (),
            span_file_span: unification_span,
        }
        .to_diagnostic()
        // todo!()
        // let a_file_span = self.tenv.get_type_filespan(a);
        // let b_file_span = self.tenv.get_type_filespan(b);

        // TypeError::TypeMismatch {
        //     a: self.tenv.fmt_ty_id(a),
        //     a_file_span,
        //     b: self.tenv.fmt_ty_id(b),
        //     b_file_span,
        //     span: (),
        //     span_file_span: unification_span,
        // }
        // .to_diagnostic()
    }
}
