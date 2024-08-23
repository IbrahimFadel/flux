use std::{collections::HashMap, mem};

use flux_id::id::{self, InPkg};
use flux_util::{FileId, Span, WithSpan};
use tracing::{trace, warn};

use crate::{r#trait::TraitApplication, r#type::Restriction, ConcreteKind, TEnv, TypeKind};

impl<'a> TEnv<'a> {
    pub fn resolve(&mut self, tid: id::Ty) -> Result<TypeKind, Vec<Restriction>> {
        // trace!(
        //     "resolving '{}: {} - {:?}",
        //     Into::<u32>::into(tid),
        //     self.fmt_tid(tid),
        //     self.get(tid).restrictions
        // );
        loop {
            let initial_restrictions = mem::take(&mut self.get_mut(tid).restrictions);
            let num_initial_restrictions = initial_restrictions.len();

            let final_restrictions: Vec<_> = initial_restrictions
                .into_iter()
                .filter(|restriction| match restriction {
                    Restriction::Equals(other) => {
                        let unification_span = Span::poisoned().in_file(FileId::poisoned());
                        // println!("{} == {}", self.fmt_tid(tid), self.fmt_tid(*other));
                        self.unify(tid, *other, unification_span).is_err()
                    }
                    Restriction::Field(_) => {
                        warn!("unimplemented");
                        false
                    }
                    Restriction::Trait(trait_restriction) => self
                        .resolve_trait_restriction(tid, trait_restriction)
                        .is_err(),
                })
                .collect();

            // println!("{:?} {}", final_restrictions, num_initial_restrictions);
            if final_restrictions.len() == 0 {
                return self
                    .get(tid)
                    .kind
                    .resolve(self)
                    .map_err(|_| final_restrictions);
            } else if final_restrictions.len() == num_initial_restrictions {
                self.get_mut(tid).restrictions = final_restrictions.clone();
                return Err(final_restrictions);
            }

            self.get_mut(tid).restrictions = final_restrictions;
        }
    }
}

impl TypeKind {
    pub fn resolve(&self, tenv: &TEnv) -> Result<TypeKind, ()> {
        use TypeKind::*;
        match &self {
            Concrete(concrete_kind) => {
                use ConcreteKind::*;
                match concrete_kind {
                    Array(_, _) => todo!(),
                    Ptr(ty) => {
                        ty.kind.resolve(tenv)?;
                        Ok(self.clone())
                    }
                    Path(path) => {
                        for arg in &path.args {
                            arg.kind.resolve(tenv)?;
                        }
                        Ok(self.clone())
                    }
                    Tuple(types) => {
                        for ty in types {
                            ty.kind.resolve(tenv)?;
                        }
                        Ok(self.clone())
                    }
                }
            }
            Generic(_) => Ok(self.clone()),
            ThisPath(this_path) => tenv.resolve_this_path(this_path).resolve(tenv),
            Ref(tid) => tenv.get(*tid).kind.resolve(tenv),
            Int => Err(()),
            Float => todo!(),
            Never => todo!(),
            Unknown => Err(()),
        }
    }
}

pub struct TraitResolver {
    pub traits: HashMap<InPkg<id::TraitDecl>, Vec<TraitApplication>>,
    // fields: HashMap<InPkg<id::StructDecl>, Vec<Word>>,
}

impl TraitResolver {
    pub fn new(traits: HashMap<InPkg<id::TraitDecl>, Vec<TraitApplication>>) -> Self {
        Self { traits }
    }
}
