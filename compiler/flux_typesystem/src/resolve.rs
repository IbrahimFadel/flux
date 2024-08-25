use std::{collections::HashMap, mem};

use flux_id::id::{self, InPkg};
use flux_util::{FileId, Span, WithSpan};
use tracing::{trace, warn};

use crate::{
    r#trait::TraitApplication, r#type::Restriction, ConcreteKind, TEnv, ThisCtx, Type, TypeKind,
};

impl<'a> TEnv<'a> {
    pub fn resolve(&mut self, tid: id::Ty) -> Result<TypeKind, Vec<Restriction>> {
        trace!(
            "resolving '{}: {} - {:?}",
            Into::<u32>::into(tid),
            self.fmt_tid(tid),
            self.get(tid).restrictions
        );
        loop {
            let initial_restrictions = mem::take(&mut self.get_mut(tid).restrictions);
            let num_initial_restrictions = initial_restrictions.len();

            let final_restrictions: Vec<_> = initial_restrictions
                .into_iter()
                .filter(|restriction| match restriction {
                    Restriction::Equals(other) => {
                        let unification_span = Span::poisoned().in_file(FileId::poisoned());
                        self.unify(tid, *other, unification_span).is_err()
                    }
                    Restriction::EqualsOneOf(tkinds) => {
                        let final_tkinds: Vec<_> = tkinds
                            .iter()
                            .filter(|tkind| self.types_unify(tkind, &self.get(tid).kind))
                            .collect();
                        match final_tkinds.len() {
                            0 => true,
                            1 => {
                                let eq_tid = self.insert(
                                    Type::new(final_tkinds[0].clone(), vec![]).at(Span::poisoned()),
                                );
                                self.add_equality(tid, eq_tid);
                                false
                            }
                            _ => todo!(),
                        }
                    }
                    Restriction::AssocTypeOf(of, trait_restriction) => {
                        let app_to_kind = self.get(*of).kind.clone();
                        let potential_applications =
                            match self.resolve_trait_restriction(*of, trait_restriction) {
                                Ok(apps) => apps,
                                Err(_) => return true,
                            };

                        match &mut self.get_inner_mut(tid).kind {
                            TypeKind::ThisPath(this_path) => {
                                let potential_this_ctx = potential_applications
                                    .into_iter()
                                    .map(|app| {
                                        ThisCtx::TraitApplication(
                                            Box::new(app_to_kind.clone()),
                                            app.assoc_types.clone(),
                                        )
                                    })
                                    .collect();
                                this_path.potential_this_ctx = potential_this_ctx;
                                this_path.potential_this_ctx.is_empty()
                            }
                            _ => true,
                        }
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
            ThisPath(this_path) => {
                let potential_this = tenv.resolve_this_path(this_path);
                if potential_this.len() == 1 {
                    potential_this[0].resolve(tenv)
                } else {
                    Err(())
                }
            }
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
