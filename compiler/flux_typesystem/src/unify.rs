use flux_diagnostics::{Diagnostic, ToDiagnostic};
use flux_id::id;
use flux_util::{InFile, Span, WithSpan};

use crate::{diagnostics::TypeError, int_paths, ConcreteKind, TEnv, Type};

impl<'res> TEnv<'res> {
    pub fn unify(
        &mut self,
        a: id::Ty,
        b: id::Ty,
        unification_span: InFile<Span>,
    ) -> Result<(), Diagnostic> {
        use crate::r#type::Type::*;
        match (
            self.types.get(a).inner.inner.clone(),
            self.types.get(b).inner.inner.clone(),
        ) {
            (Unknown, _) => {
                self.types
                    .set_with(a, |old_ty| old_ty.map_inner(|_| Ref(b)));
                Ok(())
            }
            (_, Unknown) => {
                self.types
                    .set_with(b, |old_ty| old_ty.map_inner(|_| Ref(a)));
                Ok(())
            }
            (Concrete(a_concrete), Concrete(b_concrete)) => {
                self.unify_concretes(a, a_concrete, b, b_concrete, unification_span)
            }
            (Int(a_id), Int(b_id)) => match (a_id, b_id) {
                (None, None) => Ok(()),
                (None, Some(b_id)) => {
                    self.types
                        .set_with(a, |old_ty| old_ty.map_inner(|_| Int(Some(b_id))));
                    Ok(())
                }
                (Some(a_id), None) => {
                    self.types
                        .set_with(b, |old_ty| old_ty.map_inner(|_| Int(Some(a_id))));
                    Ok(())
                }
                (Some(a_id), Some(b_id)) => self.unify(a_id, b_id, unification_span),
            },
            (Int(int_id), Concrete(ConcreteKind::Path(path))) => match int_id {
                Some(id) => self.unify(id, b, unification_span),
                None => {
                    if path.is_in(int_paths(self.interner)) {
                        self.types
                            .set_with(a, |old_ty| old_ty.map_inner(|_| Int(Some(b))));
                        Ok(())
                    } else {
                        return Err(self.type_mismatch(a, b, unification_span));
                    }
                }
            },
            (Concrete(ConcreteKind::Path(path)), Int(int_id)) => match int_id {
                Some(id) => self.unify(a, id, unification_span),
                None => {
                    if path.is_in(int_paths(self.interner)) {
                        self.types
                            .set_with(b, |old_ty| old_ty.map_inner(|_| Type::Int(Some(a))));
                        Ok(())
                    } else {
                        return Err(self.type_mismatch(a, b, unification_span));
                    }
                }
            },
            (ThisPath(this_path), _) => {
                let a_filespan = self.get_filespan(a);
                let ty = self.resolve_this_path(&this_path);
                let tid = self.insert(ty.clone().file_span(a_filespan.file_id, a_filespan.inner));
                self.unify(tid, b, unification_span)
            }
            (_, ThisPath(this_path)) => {
                let b_filespan = self.get_filespan(b);
                let ty = self.resolve_this_path(&this_path);
                let tid = self.insert(ty.clone().file_span(b_filespan.file_id, b_filespan.inner));
                self.unify(a, tid, unification_span)
            }
            (_, _) => Err(self.type_mismatch(a, b, unification_span)),
        }
    }

    fn unify_concretes(
        &mut self,
        a: id::Ty,
        a_concrete: ConcreteKind,
        b: id::Ty,
        b_concrete: ConcreteKind,
        unification_span: InFile<Span>,
    ) -> Result<(), Diagnostic> {
        use ConcreteKind::*;
        let a_filespan = self.get_filespan(a);
        let b_filespan = self.get_filespan(b);
        match (a_concrete, b_concrete) {
            (Array(a_arr, n_a), Array(b_arr, n_b)) => {
                let (n_a, n_b) = (n_a, n_b);
                let a_arr = self.insert((*a_arr).file_span(a_filespan.file_id, a_filespan.inner));
                let b_arr = self.insert((*b_arr).file_span(b_filespan.file_id, b_filespan.inner));
                self.unify(a_arr, b_arr, unification_span.clone())?;
                if n_a != n_b {
                    Err(self.type_mismatch(a, b, unification_span))
                } else {
                    Ok(())
                }
            }
            (Path(a_path), Path(b_path)) => {
                if a_path.segments != b_path.segments {
                    return Err(self.type_mismatch(a, b, unification_span));
                }
                let a_generic_args = a_path.args.clone();
                let b_generic_args = b_path.args.clone();
                let a_args_len = a_generic_args.len();
                let b_args_len = b_generic_args.len();
                if a_args_len < b_args_len {
                    for i in 0..a_args_len {
                        let a_arg = self.insert(
                            a_generic_args[i]
                                .clone()
                                .file_span(a_filespan.file_id, a_filespan.inner),
                        );
                        let b_arg = self.insert(
                            b_generic_args[i]
                                .clone()
                                .file_span(b_filespan.file_id, b_filespan.inner),
                        );
                        self.unify(a_arg, b_arg, unification_span)?;
                    }
                    for i in a_args_len..b_args_len {
                        self.push_arg_to_path(a, b_generic_args[i].clone());
                    }
                } else if b_args_len < a_args_len {
                    for i in 0..b_args_len {
                        let a_arg = self.insert(
                            a_generic_args[i]
                                .clone()
                                .file_span(a_filespan.file_id, a_filespan.inner),
                        );
                        let b_arg = self.insert(
                            b_generic_args[i]
                                .clone()
                                .file_span(b_filespan.file_id, b_filespan.inner),
                        );
                        self.unify(a_arg, b_arg, unification_span)?;
                    }
                    for i in b_args_len..a_args_len {
                        self.push_arg_to_path(b, a_generic_args[i].clone());
                    }
                }
                Ok(())
            }
            (Ptr(a_ptr), Ptr(b_ptr)) => {
                let a_filespan = self.get_filespan(a);
                let b_filespan = self.get_filespan(b);
                let a_ptr = self.insert((*a_ptr).file_span(a_filespan.file_id, a_filespan.inner));
                let b_ptr = self.insert((*b_ptr).file_span(b_filespan.file_id, b_filespan.inner));
                self.unify(a_ptr, b_ptr, unification_span)
            }
            (Tuple(a_tup), Tuple(b_tup)) => {
                let a_filespan = self.get_filespan(a);
                let b_filespan = self.get_filespan(b);
                a_tup
                    .clone()
                    .iter()
                    .zip(b_tup.clone().iter())
                    .map(|(a_inner, b_inner)| {
                        let a_inner = self.insert(
                            a_inner
                                .clone()
                                .file_span(a_filespan.file_id, a_filespan.inner),
                        );
                        let b_inner = self.insert(
                            b_inner
                                .clone()
                                .file_span(b_filespan.file_id, b_filespan.inner),
                        );
                        self.unify(a_inner, b_inner, unification_span.clone())
                    })
                    .collect::<Result<(), _>>()
            }
            _ => Err(self.type_mismatch(a, b, unification_span)),
        }
    }

    fn type_mismatch(&self, a: id::Ty, b: id::Ty, unification_span: InFile<Span>) -> Diagnostic {
        let a_ty = self.types.get(a);
        let b_ty = self.types.get(b);
        TypeError::TypeMismatch {
            a: self.fmt_tid(a),
            a_file_span: a_ty.to_file_span(),
            b: self.fmt_tid(b),
            b_file_span: b_ty.to_file_span(),
            span: (),
            span_file_span: unification_span,
        }
        .to_diagnostic()
    }
}
