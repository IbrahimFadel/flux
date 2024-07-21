use flux_diagnostics::{ice, Diagnostic, ToDiagnostic};
use flux_span::{FileId, InFile, Span, WithSpan};

use crate::{
    diagnostics::TypeError, env::TEnv, Application, ApplicationId, ConcreteKind, FnSignature,
    Generic, ThisPath, TraitId, TypeId, TypeKind,
};

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
    pub fn insert_trait_application(
        &mut self,
        trid: TraitId,
        application: Application,
    ) -> ApplicationId {
        let applications = self
            .tenv
            .trait_applications
            .get_mut(trid.raw() - 1)
            .unwrap_or_else(|| ice(format!("invalid `TraitID` {:?}", trid)));
        let idx = applications.len() + 1;
        applications.push(application);
        debug_assert_ne!(idx, 0);
        ApplicationId::new(idx)
    }

    pub fn insert_application(
        &mut self,
        tid: TypeId,
        mut signatures: Vec<FnSignature>,
    ) -> ApplicationId {
        let poisoned_filespan = Span::poisoned().in_file(unsafe { FileId::poisoned() });

        for (idx, (ty, _)) in self.tenv.applications.clone().iter().enumerate() {
            if self.unify(tid, *ty, poisoned_filespan).is_ok() {
                self.tenv.applications[idx].1.append(&mut signatures);
                return ApplicationId::new(idx + 1);
            }
        }

        let idx = self.tenv.applications.len() + 1;
        self.tenv.applications.push((tid, signatures));
        debug_assert_ne!(idx, 0);
        ApplicationId::new(idx)
    }

    pub fn get_trait_application(&mut self, trid: &TraitId, aid: &ApplicationId) -> &Application {
        &self
            .tenv
            .trait_applications
            .get(trid.raw() - 1)
            .unwrap_or_else(|| ice(format!("invalid `TraitID` {:?}", trid)))
            .get(aid.raw() - 1)
            .unwrap_or_else(|| ice(format!("invalid `ApplicationId` {:?}", aid)))
    }

    pub fn get_application(&mut self, aid: &ApplicationId) -> &(TypeId, Vec<FnSignature>) {
        &self.tenv.applications[aid.raw() - 1]
    }

    pub fn valid_applications(&mut self, tid: TypeId, trid: &TraitId) -> Option<ApplicationId> {
        let poisoned_filespan = Span::poisoned().in_file(unsafe { FileId::poisoned() });
        self.tenv
            .trait_applications
            .get(trid.raw() - 1)
            .cloned()
            .map(|applications| {
                let valid_impls: Vec<_> = applications
                    .clone()
                    .iter()
                    .enumerate()
                    .map(|(idx, application)| {
                        self.unify(tid, application.tid, poisoned_filespan.clone())
                            .ok()
                            .map(|_| ApplicationId::new(idx + 1))
                    })
                    .flatten()
                    .collect();
                match valid_impls.len() {
                    0 => None,
                    1 => Some(valid_impls[0]),
                    2.. => ice("too lazy to do this"),
                }
            })
            .flatten()
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

    fn push_generic_arg_to_path(&mut self, tid: TypeId, generic_arg: TypeId) {
        if let TypeKind::Concrete(ConcreteKind::Path(path)) =
            &mut self.tenv.get_mut(&tid).inner.inner
        {
            path.generic_args.push(generic_arg);
        } else {
            ice("cannot push generic arg to non path type");
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
                        let a_generic_args = a_path.generic_args.clone();
                        let b_generic_args = b_path.generic_args.clone();
                        let a_args_len = a_generic_args.len();
                        let b_args_len = b_generic_args.len();
                        if a_args_len < b_args_len {
                            for i in 0..a_args_len {
                                let a_arg = a_generic_args[i];
                                let b_arg = b_generic_args[i];
                                self.unify(a_arg, b_arg, unification_span)?;
                            }
                            for i in a_args_len..b_args_len {
                                self.push_generic_arg_to_path(a, b_generic_args[i]);
                            }
                        } else if b_args_len < a_args_len {
                            for i in 0..b_args_len {
                                let a_arg = a_generic_args[i];
                                let b_arg = b_generic_args[i];
                                self.unify(a_arg, b_arg, unification_span)?;
                            }
                            for i in b_args_len..a_args_len {
                                self.push_generic_arg_to_path(b, a_generic_args[i]);
                            }
                        }
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
            // (Int(int_id), Generic(generic)) => {
            // Check all int types if they fit the description, choose default int type if there's more than one
            // }
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
            (ThisPath(this_path), _) => {
                self.check_this_path(&this_path.clone(), a, b, unification_span)?
            }
            (_, ThisPath(this_path)) => {
                self.check_this_path(&this_path.clone(), b, a, unification_span)?
            }
            (Generic(a_gen), Generic(b_gen)) => {
                if !self.generics_equal(&a_gen.clone(), &b_gen.clone()) {
                    return Err(self.type_mismatch(&a, &b, unification_span));
                }
            }
            (_, _) => return Err(self.type_mismatch(&a, &b, unification_span)),
        }
        Ok(())
    }

    fn generics_equal(&mut self, a_gen: &Generic, b_gen: &Generic) -> bool {
        if a_gen.restrictions.len() != b_gen.restrictions.len() {
            return false;
        }
        let poisoned_filespan = Span::poisoned().in_file(unsafe { FileId::poisoned() });
        for a_restriction in &a_gen.restrictions {
            for b_restriciton in &b_gen.restrictions {
                if a_restriction.absolute_path != b_restriciton.absolute_path {
                    return false;
                }
                if a_restriction.args.len() != b_restriciton.args.len() {
                    return false;
                }
                let successful_unifications = a_restriction
                    .args
                    .iter()
                    .zip(b_restriciton.args.iter())
                    .map(|(a, b)| self.unify(*a, *b, poisoned_filespan.clone()))
                    .flatten()
                    .count();
                if successful_unifications != a_restriction.args.len() {
                    return false;
                }
            }
        }

        true
    }

    fn check_this_path(
        &mut self,
        this_path: &ThisPath,
        this_tid: TypeId,
        b: TypeId,
        span: InFile<Span>,
    ) -> Result<(), Diagnostic> {
        let ThisPath { segments, this_ctx } = this_path;
        match (this_ctx.trait_id, this_ctx.application_id) {
            (Some(trid), Some(aid)) => {
                let app = self.get_trait_application(&trid, &aid);
                let name = &segments[0];
                let assoc_type = app
                    .assoc_types
                    .iter()
                    .find_map(|(aname, tid)| if aname == name { Some(tid) } else { None })
                    .copied();
                match assoc_type {
                    Some(assoc_tid) => self.unify(b, assoc_tid, span),
                    None => Err(self.type_mismatch(&this_tid, &b, span)),
                }
            }
            (None, Some(aid)) => {
                let (tid, _) = self.get_application(&aid);
                let tid = *tid;
                self.unify(tid, b, span)
            }
            (None, None) => ice("`ThisPath` cannot have missing `TraitId` and `ApplicationId`"),
            (Some(_), None) => todo!(),
        }
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
