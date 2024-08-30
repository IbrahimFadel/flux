use std::convert::identity;

use flux_diagnostics::{Diagnostic, ToDiagnostic};
use flux_id::id;
use flux_util::{InFile, Span, WithSpan};

use crate::{diagnostics::TypeError, int_paths, ConcreteKind, TEnv, Type, TypeKind};

impl<'a> TEnv<'a> {
    pub fn unify(
        &mut self,
        a: id::Ty,
        b: id::Ty,
        unification_span: InFile<Span>,
    ) -> Result<(), Diagnostic> {
        use TypeKind::*;
        // println!("{:?} {:?}", self.get(a).kind, self.get(b).kind);
        // println!("{} {}", self.fmt_tid(a), self.fmt_tid(b));
        let unifies = match (&self.get(a).inner.kind, &self.get(b).inner.kind) {
            (Ref(a_ref), _) => self.unify(*a_ref, b, unification_span),
            (_, Ref(b_ref)) => self.unify(a, *b_ref, unification_span),
            (Unknown, _) => {
                self.types
                    .set_with(a, |old_ty| old_ty.map(|ty| ty.set_kind(Ref(b))));
                Ok(())
            }
            (_, Unknown) => {
                self.types
                    .set_with(b, |old_ty| old_ty.map(|ty| ty.set_kind(Ref(a))));
                Ok(())
            }
            (ThisPath(this_path), _) => {
                let a_ty = self.get(a);
                let potential_this = self.resolve_this_path(this_path);
                if potential_this.len() == 1 {
                    let this_tid = self.insert(
                        Type::new(potential_this[0].clone(), a_ty.restrictions.clone())
                            .at(a_ty.span),
                    );
                    self.unify(this_tid, b, unification_span)
                } else {
                    Err(self.type_mismatch(a, b, unification_span))
                }
            }
            (_, ThisPath(this_path)) => {
                let b_ty = self.get(b);
                let potential_this = self.resolve_this_path(this_path);
                if potential_this.len() == 1 {
                    let this_tid = self.insert(
                        Type::new(potential_this[0].clone(), b_ty.restrictions.clone())
                            .at(b_ty.span),
                    );
                    self.unify(a, this_tid, unification_span)
                } else {
                    Err(self.type_mismatch(a, b, unification_span))
                }
            }
            (Concrete(a_concrete), Concrete(b_concrete)) => match (a_concrete, b_concrete) {
                (ConcreteKind::Path(a_path), ConcreteKind::Path(b_path)) => {
                    if a_path.segments != b_path.segments {
                        return Err(self.type_mismatch(a, b, unification_span));
                    } else {
                        let mut new_a_path = a_path.clone();
                        let mut new_b_path = b_path.clone();
                        a_path
                            .args
                            .clone()
                            .into_iter()
                            .zip(b_path.args.clone().into_iter())
                            .enumerate()
                            .map(|(i, (a_arg, b_arg))| {
                                let a_tid = self.insert(a_arg.at(Span::poisoned()));
                                let b_tid = self.insert(b_arg.at(Span::poisoned()));
                                let res = self.unify(a_tid, b_tid, unification_span);
                                new_a_path.args[i] = self.get(a_tid).inner.clone();
                                new_b_path.args[i] = self.get(b_tid).inner.clone();
                                res
                            })
                            .collect::<Result<Vec<_>, Diagnostic>>()?;

                        self.types.set_with(a, |old_ty| {
                            old_ty.map(|ty| {
                                ty.set_kind(TypeKind::Concrete(ConcreteKind::Path(new_a_path)))
                            })
                        });
                        self.types.set_with(b, |old_ty| {
                            old_ty.map(|ty| {
                                ty.set_kind(TypeKind::Concrete(ConcreteKind::Path(new_b_path)))
                            })
                        });
                        Ok(())
                    }
                }
                (ConcreteKind::Tuple(a_tup), ConcreteKind::Tuple(b_tup)) => {
                    let a_span = self.get_span(a);
                    let b_span = self.get_span(b);
                    a_tup
                        .clone()
                        .iter()
                        .zip(b_tup.clone().iter())
                        .map(|(a_inner, b_inner)| {
                            let a_inner = self.insert(a_inner.clone().at(a_span));
                            let b_inner = self.insert(b_inner.clone().at(b_span));
                            self.unify(a_inner, b_inner, unification_span.clone())
                        })
                        .collect::<Result<(), _>>()
                }
                (ConcreteKind::Ptr(a_ptr), ConcreteKind::Ptr(b_ptr)) => {
                    let a_span = self.get_span(a);
                    let b_span = self.get_span(b);
                    let a_ptr = (*a_ptr.clone()).at(a_span);
                    let b_ptr = (*b_ptr.clone()).at(b_span);
                    let a_ptr = self.insert(a_ptr);
                    let b_ptr = self.insert(b_ptr);
                    self.unify(a_ptr, b_ptr, unification_span)
                }
                _ => Err(self.type_mismatch(a, b, unification_span)),
            },
            (Int, Int) => Ok(()),
            (Concrete(ConcreteKind::Path(path)), Int) if path.is_in(int_paths(self.interner)) => {
                self.types
                    .set_with(b, |old_ty| old_ty.map(|ty| ty.set_kind(Ref(a))));
                Ok(())
            }
            (Int, Concrete(ConcreteKind::Path(path))) if path.is_in(int_paths(self.interner)) => {
                self.types
                    .set_with(a, |old_ty| old_ty.map(|ty| ty.set_kind(Ref(b))));
                Ok(())
            }
            (Generic(_, a_restrictions), Generic(_, b_restrictions)) => {
                let b_restrictions = b_restrictions.clone();
                for a_restriction in a_restrictions.clone() {
                    self.resolve_trait_restriction(b, &a_restriction)
                        .map_err(|_| self.type_mismatch(a, b, unification_span))?;
                }
                for b_restriction in b_restrictions {
                    self.resolve_trait_restriction(a, &b_restriction)
                        .map_err(|_| self.type_mismatch(a, b, unification_span))?;
                }
                Ok(())
            }
            (_, _) => Err(self.type_mismatch(a, b, unification_span)),
        };
        // println!("OK: {}", unifies.is_ok());
        unifies
    }

    pub fn types_unify(&self, a: &TypeKind, b: &TypeKind) -> bool {
        use TypeKind::*;
        match (a, b) {
            (_, Unknown) | (Unknown, _) => false,
            (Ref(a), _) => self.types_unify(&self.get(*a).kind, b),
            (_, Ref(b)) => self.types_unify(a, &self.get(*b).kind),
            (Concrete(a_concrete), Concrete(b_concrete)) => {
                self.concretes_unify(a_concrete, b_concrete)
            }
            (Concrete(ConcreteKind::Path(path)), Int)
            | (Int, Concrete(ConcreteKind::Path(path)))
                if path.is_in(int_paths(self.interner)) =>
            {
                true
            }
            (Generic(_, a_restrictions), Generic(_, b_restrictions)) => {
                let b_restrictions = b_restrictions.clone();
                for a_restriction in a_restrictions.clone() {
                    if !self.does_type_implement_trait(b, &a_restriction) {
                        return false;
                    }
                }
                for b_restriction in b_restrictions {
                    if !self.does_type_implement_trait(a, &b_restriction) {
                        return false;
                    }
                }
                true
            }
            // ...
            _ => false,
        }
    }

    pub fn assoc_types_unify(&self, a: &TypeKind, b: &TypeKind) {}

    fn concretes_unify(&self, a_concrete: &ConcreteKind, b_concrete: &ConcreteKind) -> bool {
        use ConcreteKind::*;
        match (a_concrete, b_concrete) {
            (Array(a_arr, n_a), Array(b_arr, n_b)) => {
                let (n_a, n_b) = (n_a, n_b);
                if !self.types_unify(&a_arr.kind, &b_arr.kind) {
                    return false;
                }
                n_a == n_b
            }
            (Path(a_path), Path(b_path)) => {
                if a_path.segments != b_path.segments {
                    return false;
                }
                let a_args_len = a_path.args.len();
                let b_args_len = b_path.args.len();
                if a_args_len < b_args_len {
                    for i in 0..a_args_len {
                        if !self.types_unify(&a_path.args[i].kind, &b_path.args[i].kind) {
                            return false;
                        }
                    }
                } else if b_args_len < a_args_len {
                    for i in 0..b_args_len {
                        if !self.types_unify(&a_path.args[i].kind, &b_path.args[i].kind) {
                            return false;
                        }
                    }
                }
                true
            }
            (Ptr(a_ptr), Ptr(b_ptr)) => self.types_unify(&a_ptr.kind, &b_ptr.kind),
            (Tuple(a_tup), Tuple(b_tup)) => a_tup
                .iter()
                .zip(b_tup.iter())
                .map(|(a_inner, b_inner)| self.types_unify(&a_inner.kind, &b_inner.kind))
                .any(identity),
            _ => false,
        }
    }

    pub(super) fn type_mismatch(
        &self,
        a: id::Ty,
        b: id::Ty,
        unification_span: InFile<Span>,
    ) -> Diagnostic {
        TypeError::TypeMismatch {
            a: self.fmt_tid(a),
            a_file_span: self.get_span(a).in_file(unification_span.file_id),
            b: self.fmt_tid(b),
            b_file_span: self.get_span(b).in_file(unification_span.file_id),
            span: (),
            span_file_span: unification_span,
        }
        .to_diagnostic()
    }
}

//     pub fn unify(
//         &mut self,
//         a: id::Ty,
//         b: id::Ty,
//         unification_span: InFile<Span>,
//     ) -> Result<(), Diagnostic> {
//         use crate::r#type::Type::*;
//         match (
//             self.types.get(a).inner.inner.clone(),
//             self.types.get(b).inner.inner.clone(),
//         ) {
//             (Unknown, _) => {
//                 self.types
//                     .set_with(a, |old_ty| old_ty.map_inner(|_| Ref(b)));
//                 Ok(())
//             }
//             (_, Unknown) => {
//                 self.types
//                     .set_with(b, |old_ty| old_ty.map_inner(|_| Ref(a)));
//                 Ok(())
//             }
//             (Concrete(a_concrete), Concrete(b_concrete)) => {
//                 self.unify_concretes(a, a_concrete, b, b_concrete, unification_span)
//             }
//             (Int(a_id), Int(b_id)) => match (a_id, b_id) {
//                 (None, None) => Ok(()),
//                 (None, Some(b_id)) => {
//                     self.types
//                         .set_with(a, |old_ty| old_ty.map_inner(|_| Int(Some(b_id))));
//                     Ok(())
//                 }
//                 (Some(a_id), None) => {
//                     self.types
//                         .set_with(b, |old_ty| old_ty.map_inner(|_| Int(Some(a_id))));
//                     Ok(())
//                 }
//                 (Some(a_id), Some(b_id)) => self.unify(a_id, b_id, unification_span),
//             },
//             (Int(int_id), Concrete(ConcreteKind::Path(path))) => match int_id {
//                 Some(id) => {
//                     println!("oh {}", self.fmt_tid(id));

//                     self.unify(id, b, unification_span)
//                 }
//                 None => {
//                     if path.is_in(int_paths(self.interner)) {
//                         println!("WAH?");
//                         self.types
//                             .set_with(a, |old_ty| old_ty.map_inner(|_| Int(Some(b))));
//                         Ok(())
//                     } else {
//                         return Err(self.type_mismatch(a, b, unification_span));
//                     }
//                 }
//             },
//             (Concrete(ConcreteKind::Path(path)), Int(int_id)) => match int_id {
//                 Some(id) => self.unify(a, id, unification_span),
//                 None => {
//                     if path.is_in(int_paths(self.interner)) {
//                         self.types
//                             .set_with(b, |old_ty| old_ty.map_inner(|_| Type::Int(Some(a))));
//                         Ok(())
//                     } else {
//                         return Err(self.type_mismatch(a, b, unification_span));
//                     }
//                 }
//             },
//             (ThisPath(this_path), _) => {
//                 let a_filespan = self.get_filespan(a);
//                 let ty = self.resolve_this_path(&this_path);
//                 let tid = self.insert(ty.clone().file_span(a_filespan.file_id, a_filespan.inner));
//                 self.unify(tid, b, unification_span)
//             }
//             (_, ThisPath(this_path)) => {
//                 let b_filespan = self.get_filespan(b);
//                 let ty = self.resolve_this_path(&this_path);
//                 let tid = self.insert(ty.clone().file_span(b_filespan.file_id, b_filespan.inner));
//                 self.unify(a, tid, unification_span)
//             }
//             (_, _) => Err(self.type_mismatch(a, b, unification_span)),
//         }
//     }

//     fn do_concretes_unify(&self, a_concrete: &ConcreteKind, b_concrete: &ConcreteKind) -> bool {
//         use ConcreteKind::*;
//         match (a_concrete, b_concrete) {
//             (Array(a_arr, n_a), Array(b_arr, n_b)) => {
//                 let (n_a, n_b) = (n_a, n_b);
//                 if !self.unifies(a_arr, b_arr) {
//                     return false;
//                 }
//                 n_a == n_b
//             }
//             (Path(a_path), Path(b_path)) => {
//                 if a_path.segments != b_path.segments {
//                     return false;
//                 }
//                 let a_args_len = a_path.args.len();
//                 let b_args_len = b_path.args.len();
//                 if a_args_len < b_args_len {
//                     for i in 0..a_args_len {
//                         if !self.unifies(&a_path.args[i], &b_path.args[i]) {
//                             return false;
//                         }
//                     }
//                 } else if b_args_len < a_args_len {
//                     for i in 0..b_args_len {
//                         if !self.unifies(&a_path.args[i], &b_path.args[i]) {
//                             return false;
//                         }
//                     }
//                 }
//                 true
//             }
//             (Ptr(a_ptr), Ptr(b_ptr)) => self.unifies(a_ptr, b_ptr),
//             (Tuple(a_tup), Tuple(b_tup)) => a_tup
//                 .iter()
//                 .zip(b_tup.iter())
//                 .map(|(a_inner, b_inner)| self.unifies(a_inner, b_inner))
//                 .any(identity),
//             _ => false,
//         }
//     }

//     fn unify_concretes(
//         &mut self,
//         a: id::Ty,
//         a_concrete: ConcreteKind,
//         b: id::Ty,
//         b_concrete: ConcreteKind,
//         unification_span: InFile<Span>,
//     ) -> Result<(), Diagnostic> {
//         use ConcreteKind::*;
//         let a_filespan = self.get_filespan(a);
//         let b_filespan = self.get_filespan(b);
//         match (a_concrete, b_concrete) {
//             (Array(a_arr, n_a), Array(b_arr, n_b)) => {
//                 let (n_a, n_b) = (n_a, n_b);
//                 let a_arr = self.insert((*a_arr).file_span(a_filespan.file_id, a_filespan.inner));
//                 let b_arr = self.insert((*b_arr).file_span(b_filespan.file_id, b_filespan.inner));
//                 self.unify(a_arr, b_arr, unification_span.clone())?;
//                 if n_a != n_b {
//                     Err(self.type_mismatch(a, b, unification_span))
//                 } else {
//                     Ok(())
//                 }
//             }
//             (Path(a_path), Path(b_path)) => {
//                 if a_path.segments != b_path.segments {
//                     return Err(self.type_mismatch(a, b, unification_span));
//                 }
//                 let a_generic_args = a_path.args.clone();
//                 let b_generic_args = b_path.args.clone();
//                 let a_args_len = a_generic_args.len();
//                 let b_args_len = b_generic_args.len();
//                 if a_args_len < b_args_len {
//                     for i in 0..a_args_len {
//                         let a_arg = self.insert(
//                             a_generic_args[i]
//                                 .clone()
//                                 .file_span(a_filespan.file_id, a_filespan.inner),
//                         );
//                         let b_arg = self.insert(
//                             b_generic_args[i]
//                                 .clone()
//                                 .file_span(b_filespan.file_id, b_filespan.inner),
//                         );
//                         self.unify(a_arg, b_arg, unification_span)?;
//                     }
//                     for i in a_args_len..b_args_len {
//                         self.push_arg_to_path(a, b_generic_args[i].clone());
//                     }
//                 } else if b_args_len < a_args_len {
//                     for i in 0..b_args_len {
//                         let a_arg = self.insert(
//                             a_generic_args[i]
//                                 .clone()
//                                 .file_span(a_filespan.file_id, a_filespan.inner),
//                         );
//                         let b_arg = self.insert(
//                             b_generic_args[i]
//                                 .clone()
//                                 .file_span(b_filespan.file_id, b_filespan.inner),
//                         );
//                         self.unify(a_arg, b_arg, unification_span)?;
//                     }
//                     for i in b_args_len..a_args_len {
//                         self.push_arg_to_path(b, a_generic_args[i].clone());
//                     }
//                 }
//                 Ok(())
//             }
//             (Ptr(a_ptr), Ptr(b_ptr)) => {
//                 let a_filespan = self.get_filespan(a);
//                 let b_filespan = self.get_filespan(b);
//                 let a_ptr = self.insert((*a_ptr).file_span(a_filespan.file_id, a_filespan.inner));
//                 let b_ptr = self.insert((*b_ptr).file_span(b_filespan.file_id, b_filespan.inner));
//                 self.unify(a_ptr, b_ptr, unification_span)
//             }
//             (Tuple(a_tup), Tuple(b_tup)) => {
//                 let a_filespan = self.get_filespan(a);
//                 let b_filespan = self.get_filespan(b);
//                 a_tup
//                     .clone()
//                     .iter()
//                     .zip(b_tup.clone().iter())
//                     .map(|(a_inner, b_inner)| {
//                         let a_inner = self.insert(
//                             a_inner
//                                 .clone()
//                                 .file_span(a_filespan.file_id, a_filespan.inner),
//                         );
//                         let b_inner = self.insert(
//                             b_inner
//                                 .clone()
//                                 .file_span(b_filespan.file_id, b_filespan.inner),
//                         );
//                         self.unify(a_inner, b_inner, unification_span.clone())
//                     })
//                     .collect::<Result<(), _>>()
//             }
//             _ => Err(self.type_mismatch(a, b, unification_span)),
//         }
//     }

//     fn type_mismatch(&self, a: id::Ty, b: id::Ty, unification_span: InFile<Span>) -> Diagnostic {
//         let a_ty = self.types.get(a);
//         let b_ty = self.types.get(b);
//         TypeError::TypeMismatch {
//             a: self.fmt_tid(a),
//             a_file_span: a_ty.to_file_span(),
//             b: self.fmt_tid(b),
//             b_file_span: b_ty.to_file_span(),
//             span: (),
//             span_file_span: unification_span,
//         }
//         .to_diagnostic()
//     }
// }
