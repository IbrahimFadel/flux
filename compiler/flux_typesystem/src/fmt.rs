use std::fmt::Display;

use flux_id::id;

use crate::{r#type::Restriction, ConcreteKind, TEnv, TraitRestriction, TypeKind};

impl Display for TEnv {
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

impl TEnv {
    pub fn fmt_tid(&self, tid: id::Ty) -> String {
        self.fmt_typekind(&self.types.get(tid).kind)
    }

    pub fn fmt_typekind(&self, ty: &TypeKind) -> String {
        use crate::TypeKind::*;
        match ty {
            ThisPath(this_path) => std::iter::once("This")
                .chain(this_path.iter().map(|key| self.interner.resolve(key)))
                .collect::<Vec<_>>()
                .join("::"),
            Concrete(concrete_kind) => self.fmt_concrete_kind(concrete_kind),
            Ref(tid) => self.fmt_tid(*tid),
            Int => format!("int"),
            Float => format!("float"),
            Generic(name) => format!("{}", self.interner.resolve(&name)),
            Never => format!("!"),
            Unknown => format!("unknown"),
        }
    }

    pub fn fmt_concrete_kind(&self, concrete_kind: &ConcreteKind) -> String {
        use ConcreteKind::*;
        match concrete_kind {
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
        }
    }

    fn fmt_trait_restriction(&self, trait_restriction: &TraitRestriction) -> String {
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
                        .map(|arg| format!("'{}", Into::<u32>::into(*arg)))
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
        )
    }
}
