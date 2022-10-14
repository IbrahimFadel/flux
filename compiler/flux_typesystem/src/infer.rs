use std::{collections::VecDeque, fmt::Display};

use itertools::Itertools;
use lasso::ThreadedRodeo;
use owo_colors::OwoColorize;

use crate::{
    constraint::Constraint,
    intern::{Interner, Key},
    r#type::{ConcreteKind, Type, TypeId, TypeKind},
};

pub struct TEnv {
    interner: Interner,
    entries: Vec<TEntry>,
    constraints: VecDeque<Constraint>,
}

/// A `flux_typesystem` type entry
///
/// Stores the [`Key`] of the type constructor and a list of constraints
struct TEntry {
    pub key: Key,
    pub constraints: Vec<TypeId>,
}

impl TEntry {
    pub fn new(key: Key) -> Self {
        Self {
            key,
            constraints: vec![],
        }
    }

    pub fn with_constraints(key: Key, constraints: Vec<TypeId>) -> Self {
        Self { key, constraints }
    }
}

impl TEnv {
    pub fn new(string_interner: &'static ThreadedRodeo) -> Self {
        Self {
            interner: Interner::with_preinterned(string_interner),
            entries: vec![],
            constraints: VecDeque::new(),
        }
    }

    fn get_entry(&self, id: TypeId) -> &TEntry {
        self.entries
            .get(id.get())
            .expect("internal compiler error: tried retrieving type with invalid type id")
    }

    // fn get_type(&self, id: TypeId) -> &TypeKind {
    //     self.interner.resolve(self.get_entry(id).key)
    // }

    pub fn insert(&mut self, ty: Type) -> TypeId {
        let key = self.interner.intern(ty.constr());
        self.entries.push(TEntry::new(key));
        key.into_type_id()
    }

    pub fn insert_with_constraints(&mut self, ty: Type, constraints: Vec<TypeId>) -> TypeId {
        let key = self.interner.intern(ty.constr());
        self.entries
            .push(TEntry::with_constraints(key, constraints));
        key.into_type_id()
    }
}

impl Display for TEnv {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}\n{}\n  {}",
            self.interner,
            "TEnv".green(),
            self.entries
                .iter()
                .enumerate()
                .format_with("\n  ", |(idx, entry), f| {
                    f(&format_args!(
                        "{}{} {} {}",
                        "'".blue(),
                        idx.blue(),
                        "->".purple(),
                        entry
                    ))
                })
        )
    }
}

impl Display for TEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:?}{}{}",
            self.key,
            if self.constraints.len() > 0 {
                format!(": ")
            } else {
                format!("")
            },
            self.constraints
                .iter()
                .format_with(", ", |constraint, f| f(&format_args!("{}", constraint)))
        )
    }
}

impl Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.constr() {
            TypeKind::Concrete(cncrt) => write!(f, "{}", cncrt),
            TypeKind::Float(_) => write!(f, "float"),
            TypeKind::Generic => write!(f, "generic"),
            TypeKind::Int(_) => write!(f, "int"),
            TypeKind::Ref(id) => write!(f, "ref('{id})"),
            TypeKind::Unknown => write!(f, "unknown"),
        }
    }
}

impl Display for ConcreteKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            // Self::F32 => write!(f, "f32"),
            // Self::F64 => write!(f, "f64"),
            Self::Path(path) => write!(f, "{:?}", path),
            Self::Ptr(ptr) => write!(f, "*'{}", ptr),
            // Self::SInt(bitwidth) => write!(f, "i{}", bitwidth),
            Self::Tuple(_) => write!(f, "()"),
            // Self::UInt(bitwidth) => write!(f, "u{}", bitwidth),
        }
    }
}

// impl Display for BitWidth {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         match self {
//             Self::Eight => write!(f, "8"),
//             Self::Sixteen => write!(f, "16"),
//             Self::ThirtyTwo => write!(f, "32"),
//             Self::SixtyFour => write!(f, "64"),
//         }
//     }
// }

impl Display for Constraint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
        // match self {
        //     Self::HasField(field) => write!(f, "has field {:?}", field),
        //     Self::ImplementsTrait(trt) => write!(f, "has trait {:?}", trt),
        // }
    }
}
