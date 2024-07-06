// mod traits;
// mod unify;

use flux_diagnostics::{Diagnostic, ToDiagnostic};
use flux_span::{InFile, Span};

use crate::{diagnostics::TypeError, env::TEnv, ConcreteKind, TypeId, TypeKind};

#[derive(Debug)]
pub struct TChecker<'tenv> {
    pub tenv: &'tenv mut TEnv,
    // pub trait_applications: TraitApplicationTable,
}

impl<'tenv> TChecker<'tenv> {
    pub fn new(tenv: &'tenv mut TEnv) -> Self {
        Self { tenv }
    }
}

impl<'tenv> TChecker<'tenv> {
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
            (ThisPath(segments, Some(aid)), _) => {
                let app = self.tenv.get_application(aid);
                let name = &segments[0];
                let assoc_type =
                    app.assoc_types
                        .iter()
                        .find_map(|(aname, tid)| if aname == name { Some(tid) } else { None });
                match assoc_type {
                    Some(assoc_tid) => self.unify(*assoc_tid, b, unification_span)?,
                    None => return Err(self.type_mismatch(&a, &b, unification_span)),
                }
            }
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
