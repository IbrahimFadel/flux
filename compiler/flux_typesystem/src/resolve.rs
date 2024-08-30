use std::{collections::HashMap, mem};

use flux_id::id::{self, InPkg};
use flux_util::{FileId, Path, Span, WithSpan};
use tracing::{trace, warn};

use crate::{
    r#trait::TraitApplication,
    r#type::{Restriction, ThisPath},
    ConcreteKind, TEnv, ThisCtx, Type, TypeKind,
};

impl<'a> TEnv<'a> {
    pub fn resolve(&mut self, tid: id::Ty) -> Result<TypeKind, Vec<Restriction>> {
        trace!(
            "resolving '{}: {} - {}",
            Into::<u32>::into(tid),
            self.fmt_tid(tid),
            self.get(tid)
                .restrictions
                .iter()
                .map(|restriction| self.fmt_restriction(tid, restriction))
                .collect::<Vec<_>>()
                .join(", ")
        );
        loop {
            let initial_restrictions = mem::take(&mut self.get_mut(tid).restrictions);

            let final_restrictions: Vec<_> = initial_restrictions
                .clone()
                .into_iter()
                .filter_map(|restriction| match &restriction {
                    Restriction::Equals(other) => {
                        // for r in &self.get(tid).restrictions {
                        //     match r {
                        //         Restriction::Equals(other) => match self.get(*other).kind {
                        //             _ => {}
                        //         },
                        //         _ => {}
                        //     }
                        // }

                        let unification_span = Span::poisoned().in_file(FileId::poisoned());
                        self.unify(tid, *other, unification_span)
                            .map_err(|_| restriction)
                            .err()
                    }
                    Restriction::EqualsOneOf(tkinds) => {
                        let mut final_tkinds: Vec<_> = tkinds
                            .iter()
                            .filter(|tkind| self.types_unify(tkind, &self.get(tid).kind))
                            .cloned()
                            .collect();

                        // Deduplicate
                        if !final_tkinds.is_empty() {
                            final_tkinds = Some(final_tkinds[0].clone())
                                .into_iter()
                                .chain(
                                    // always include first element
                                    final_tkinds
                                        .windows(2)
                                        .filter(|w| !self.types_unify(&w[0], &w[1]))
                                        .map(|w| w[1].clone()),
                                )
                                .collect();
                        }

                        match final_tkinds.len() {
                            0 => Some(restriction),
                            1 => {
                                let eq_tid = self.insert(
                                    Type::new(final_tkinds[0].clone(), vec![])
                                        .at(self.get_span(tid)),
                                );
                                Some(Restriction::Equals(eq_tid))
                            }
                            _ => Some(Restriction::EqualsOneOf(final_tkinds)),
                        }
                    }
                    Restriction::AssocTypeOf(of, trait_restriction, name) => {
                        let app_to_kind = self.get(*of).kind.clone();
                        let potential_applications =
                            match self.resolve_trait_restriction(*of, trait_restriction) {
                                Ok(apps) => apps,
                                Err(_) => return Some(restriction),
                            };

                        let potential_this_ctx = potential_applications
                            .iter()
                            .map(|app| {
                                ThisCtx::TraitApplication(
                                    Box::new(app_to_kind.clone()),
                                    app.assoc_types.clone(),
                                )
                            })
                            .collect();
                        match &mut self.get_inner_mut(tid).kind {
                            TypeKind::ThisPath(this_path) => {
                                this_path.potential_this_ctx = potential_this_ctx;
                                if this_path.potential_this_ctx.is_empty() {
                                    Some(restriction)
                                } else {
                                    None
                                }
                            }
                            _ => {
                                for r in &initial_restrictions {
                                    match r {
                                        Restriction::Equals(other) => {
                                            match &mut self.get_inner_mut(*other).kind {
                                                TypeKind::ThisPath(this_path) => {
                                                    this_path.potential_this_ctx =
                                                        potential_this_ctx.clone();
                                                }
                                                _ => {}
                                            }
                                        }
                                        _ => {}
                                    }
                                }

                                if potential_this_ctx.len() == 1 {
                                    None
                                } else {
                                    Some(restriction)
                                }
                            }
                        }
                    }
                    Restriction::Field(_) => {
                        warn!("unimplemented");
                        None
                    }
                    Restriction::Trait(trait_restriction) => self
                        .resolve_trait_restriction(tid, &trait_restriction)
                        .map_err(|_| restriction)
                        .err(),
                })
                .collect();

            if final_restrictions.len() == 0 {
                return self
                    .get(tid)
                    .kind
                    .resolve(self)
                    .map_err(|_| final_restrictions);
            } else if final_restrictions == initial_restrictions {
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
                    Addr(ty) => {
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
                    Fn(signature) => {
                        for param in signature.parameters() {
                            param.kind.resolve(tenv)?;
                        }
                        signature.return_ty().kind.resolve(tenv)?;
                        Ok(self.clone())
                    }
                }
            }
            Generic(_, _) => Ok(self.clone()),
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
