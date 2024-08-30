use std::fmt::Display;

use flux_id::id;

use crate::{
    r#type::Restriction, ConcreteKind, TEnv, TraitApplication, TraitRestriction, TypeKind,
};

impl<'a> Display for TEnv<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut relevant_types = vec![];
        for (tid, ty) in self.types.iter() {
            if !ty.restrictions.is_empty() {
                if !relevant_types.contains(&tid) {
                    relevant_types.push(tid);
                }
            }
            for restriction in &ty.restrictions {
                match restriction {
                    Restriction::Equals(other) => {
                        if !relevant_types.contains(other) {
                            relevant_types.push(*other);
                        }
                    }
                    Restriction::EqualsOneOf(_) => todo!(),
                    Restriction::AssocTypeOf(_, _, _) => todo!(),
                    Restriction::Field(_) => {}
                    Restriction::Trait(_) => {}
                }
            }
        }
        let types = relevant_types
            .iter()
            .map(|tid| {
                format!(
                    "'{}: {}{}\n",
                    Into::<u32>::into(*tid),
                    self.fmt_tid(*tid),
                    match self.try_get_local_by_tid(*tid) {
                        Some(name) => format!(" ({})", self.interner.resolve(&name)),
                        None => format!(""),
                    }
                )
            })
            .collect::<String>();
        let restrictions = relevant_types
            .iter()
            .filter(|tid| !self.get(**tid).restrictions.is_empty())
            .map(|tid| {
                format!(
                    "{}",
                    self.get(*tid)
                        .restrictions
                        .iter()
                        .map(|restriction| match restriction {
                            Restriction::Equals(other) => format!(
                                "'{} == '{}",
                                Into::<u32>::into(*tid),
                                Into::<u32>::into(*other),
                            ),
                            Restriction::EqualsOneOf(_) => todo!(),
                            Restriction::AssocTypeOf(_, _, _) => todo!(),
                            Restriction::Field(name) => format!(
                                "'{} has field `{}`",
                                Into::<u32>::into(*tid),
                                self.interner.resolve(name)
                            ),
                            Restriction::Trait(trait_restriction) => format!(
                                "'{} implements {}",
                                Into::<u32>::into(*tid),
                                self.fmt_trait_restriction(trait_restriction)
                            ),
                        })
                        .collect::<Vec<_>>()
                        .join("\n")
                )
            })
            .collect::<Vec<_>>()
            .join("\n");
        write!(f, "Types:\n{types}\nRestrictions:\n{restrictions}\n")
    }
}

impl<'a> TEnv<'a> {
    pub fn fmt_tid(&self, tid: id::Ty) -> String {
        self.fmt_typekind(&self.types.get(tid).kind)
    }

    // pub fn fmt_tid_include_resolved(&mut self, tid: id::Ty) -> Vec<String> {
    //     let tkind_b = self.get(tid).kind.resolve(self);
    //     let tkind_a = &self.get(tid).kind;
    //     match tkind_b {
    //         Ok(tkind_b) if tkind_b != *tkind_a => {
    //             vec![self.fmt_typekind(tkind_a), self.fmt_typekind(&tkind_b)]
    //         }
    //         Ok(_) | Err(_) => vec![self.fmt_typekind(tkind_a)],
    //     }
    // }

    pub fn fmt_typekind(&self, ty: &TypeKind) -> String {
        use crate::TypeKind::*;
        match ty {
            ThisPath(this_path) => std::iter::once("This")
                .chain(this_path.path.iter().map(|key| self.interner.resolve(key)))
                .collect::<Vec<_>>()
                .join("::"),
            Concrete(concrete_kind) => self.fmt_concrete_kind(concrete_kind),
            Ref(tid) => self.fmt_tid(*tid),
            Int => format!("int"),
            Float => format!("float"),
            Generic(name, _) => format!("{}", self.interner.resolve(&name)),
            Never => format!("!"),
            Unknown => format!("unknown"),
        }
    }

    pub fn fmt_concrete_kind(&self, concrete_kind: &ConcreteKind) -> String {
        use ConcreteKind::*;
        match concrete_kind {
            Addr(ty) => format!("{}&", self.fmt_typekind(&ty.kind)),
            Array(ty, n) => format!("[{}; {n}]", self.fmt_typekind(&ty.kind)),
            Ptr(ty) => format!("{}*", self.fmt_typekind(&ty.kind)),
            Path(path) => format!(
                "{}{}",
                path.segments
                    .iter()
                    .map(|key| self.interner.resolve(key))
                    .collect::<Vec<_>>()
                    .join("::"),
                if path.args.is_empty() {
                    String::new()
                } else {
                    format!(
                        "<{}>",
                        path.args
                            .iter()
                            .map(|ty| self.fmt_typekind(&ty.kind))
                            .collect::<Vec<_>>()
                            .join(", ")
                    )
                }
            ),
            Tuple(tids) => format!(
                "({})",
                tids.iter()
                    .map(|ty| self.fmt_typekind(&ty.kind))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Fn(signature) => format!(
                "Fn({}) -> {}",
                signature
                    .parameters()
                    .iter()
                    .map(|param| self.fmt_typekind(&param.kind))
                    .collect::<Vec<_>>()
                    .join(", "),
                self.fmt_typekind(&signature.return_ty().kind)
            ),
        }
    }

    pub fn fmt_trait_restriction(&self, trait_restriction: &TraitRestriction) -> String {
        format!(
            "TrId({}){}",
            Into::<u32>::into(trait_restriction.trait_id.inner),
            if trait_restriction.args.is_empty() {
                format!("")
            } else {
                format!(
                    "<{}>",
                    trait_restriction
                        .args
                        .iter()
                        .map(|arg| self.fmt_tid(*arg))
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
        )
    }

    pub fn fmt_restriction(&self, tid: id::Ty, restriction: &Restriction) -> String {
        match restriction {
            Restriction::Equals(other) => {
                format!("{} == {}", self.fmt_tid(tid), self.fmt_tid(*other))
            }
            Restriction::EqualsOneOf(types) => format!(
                "{} == [{}]",
                self.fmt_tid(tid),
                types
                    .iter()
                    .map(|tkind| self.fmt_typekind(tkind))
                    .collect::<Vec<_>>()
                    .join(" | ")
            ),
            Restriction::AssocTypeOf(ty, trait_restriction, name) => format!(
                "{} associated type `{}` of {} with trait restriction {}",
                self.fmt_tid(tid),
                self.interner.resolve(name),
                self.fmt_tid(*ty),
                self.fmt_trait_restriction(trait_restriction)
            ),
            Restriction::Field(_) => todo!(),
            Restriction::Trait(trait_restriction) => format!(
                "{}: {}",
                self.fmt_tid(tid),
                self.fmt_trait_restriction(trait_restriction)
            ),
        }
    }

    pub fn fmt_trait_application(&self, app: &TraitApplication) -> String {
        format!(
            "to {}{}",
            self.fmt_typekind(&app.to),
            if app.args.is_empty() {
                format!("")
            } else {
                format!(
                    " with args {}",
                    app.args
                        .iter()
                        .map(|arg| self.fmt_typekind(arg))
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
        )
    }
}
