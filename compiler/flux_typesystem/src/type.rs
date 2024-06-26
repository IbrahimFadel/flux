use std::fmt::Display;

use flux_span::Word;
use itertools::Itertools;

use crate::r#trait::TraitRestriction;

/// A `flux_typesystem` type id
///
/// Types are stored in and organized by the type environment -- in order to refer to them, `TypeId`s are used.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TypeId(usize);

impl TypeId {
    pub const fn new(id: usize) -> Self {
        Self(id)
    }

    pub fn raw(&self) -> usize {
        self.0
    }
}

impl Display for TypeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "'{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TypeKind {
    AssocPath(Word),
    Concrete(ConcreteKind),
    Int(Option<TypeId>),
    Float(Option<TypeId>),
    Ref(TypeId),
    Generic(Generic),
    Never,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Generic {
    pub name: Word,
    pub restrictions: Vec<TraitRestriction>,
}

impl Generic {
    pub fn new(name: Word, restrictions: Vec<TraitRestriction>) -> Self {
        Self { name, restrictions }
    }
}

impl Display for TypeKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AssocPath(_) => write!(f, "todo"),
            Self::Concrete(concrete) => write!(f, "{concrete}"),
            Self::Float(_) => write!(f, "float"),
            Self::Generic(Generic { name, .. }) => write!(f, "{name:?}"),
            Self::Int(_) => write!(f, "int"),
            Self::Ref(id) => write!(f, "Ref({id}"),
            Self::Never => write!(f, "!"),
            Self::Unknown => write!(f, "unknown"),
        }
    }
}

/// A `flux_typesystem` concrete kind
///
/// The kind of [`TypeKind::Concrete`]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ConcreteKind {
    Array(TypeId, u32),
    Ptr(TypeId),
    Path(Vec<Word>, Vec<TypeId>),
    Tuple(Vec<TypeId>),
}

impl Display for ConcreteKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Array(ty, n) => write!(f, "[{ty}; {n}]"),
            Self::Path(path, _) => write!(f, "{path:?}"),
            Self::Ptr(ptr) => write!(f, "*{ptr}"),
            Self::Tuple(types) => {
                write!(f, "({})", types.iter().map(|id| id.to_string()).join(", "))
            }
        }
    }
}
