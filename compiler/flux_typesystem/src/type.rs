use std::fmt::Display;

use flux_proc_macros::Locatable;
use flux_span::WithSpan;
use lasso::Spur;

use crate::TraitRestriction;

mod pp;

/// A `flux_typesystem` type
///
/// Types consist of a constructor and parameters
///
/// The type `Foo<i32, T, Bar>` has the constructor `Foo` and parameters `[i32, T, Bar]`
///
/// Types always have a constructor, but not always parameters, as such we can store all the information in one vector rather than two to save memory.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Locatable)]
pub struct Type(Vec<TypeKind>);

impl Type {
    /// Create a new [`Type`] with only a constructor
    ///
    /// Stores the constructor as the first element in the vector
    pub fn new(constr: TypeKind) -> Self {
        let types = vec![constr; 1];
        Self(types)
    }

    /// Create a new [`Type`] with a constructor and parameters
    ///
    /// Stores the constructor as the first element in the vector, and fills the rest of the vector with the parameters
    pub fn with_params(constr: TypeKind, params: impl Iterator<Item = TypeKind>) -> Self {
        let types = std::iter::once(constr).chain(params).collect();
        Self(types)
    }

    /// Get a [`Type`]'s type constructor (the first element in the vector)
    pub fn constr(&self) -> &TypeKind {
        &self.0[0]
    }

    /// Get a [`Type`]'s type parameters (everything following the first element in the vector)
    pub fn params(&self) -> Option<&[TypeKind]> {
        self.0.get(1..)
    }
}

/// A `flux_typesystem` type id
///
/// Types are stored in and organized by the type environment -- in order to refer to them, `TypeId`s are used.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TypeId(usize);

impl WithSpan for TypeId {}

impl TypeId {
    pub const fn new(id: usize) -> Self {
        Self(id)
    }

    pub fn get(&self) -> usize {
        self.0
    }
}

impl Display for TypeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "'{}", self.0)
    }
}

/// A `flux_typesystem` type kind
///
/// Type Kinds
/// ----------
///
/// * Concrete
///     - Known
/// * Int
///     - All that is known about the type is that it is an integer
///     - Optionally, the supertype of the integer can be known and stored with a [`TypeId`]
/// * Float
///     - All that is known about the type is that it is an float
///     - Optionally, the supertype of the float can be known and stored with a [`TypeId`]
/// * Ref
///     - Depends on the type of another [`TypeId`]
/// * Generic
///     - Generic type
/// * Unknown
///     - No information is known about this type
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TypeKind {
    AssocPath(Spur),
    Concrete(ConcreteKind),
    Int(Option<TypeId>),
    Float(Option<TypeId>),
    Ref(TypeId),
    // Generic(Vec<(Spur, Vec<TypeId>)>),
    Generic(Spur, Vec<TraitRestriction>),
    Never,
    Unknown,
}

impl Display for TypeKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AssocPath(_) => write!(f, "todo"),
            Self::Concrete(concrete) => write!(f, "{concrete}"),
            Self::Float(_) => write!(f, "float"),
            Self::Generic(name, _) => write!(f, "{name:?}"),
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
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ConcreteKind {
    Array(TypeId, u32),
    Ptr(TypeId),
    Path(Spur, Vec<TypeId>),
    Tuple(Vec<TypeId>),
}
