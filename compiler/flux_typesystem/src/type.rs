use std::fmt::Display;

use flux_span::Span;
use lasso::Spur;
use tinyvec::TinyVec;

/// A `flux_typesystem` type
///
/// Types consist of a constructor and parameters
///
/// The type `Foo<i32, T, Bar>` has the constructor `Foo` and parameters `[i32, T, Bar]`
///
/// Types always have a constructor, but not always parameters, as such we can store all the information in once vector rather than two to save memory.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Type {
    types: Vec<TypeKind>,
    span: Span,
}

impl Type {
    /// Create a new [`Type`] with only a constructor
    ///
    /// Stores the constructor as the first element in the vector
    pub fn new(constr: TypeKind, span: Span) -> Self {
        let mut types = Vec::with_capacity(1);
        types.push(constr);
        Self { types, span }
    }

    /// Create a new [`Type`] with a constructor and parameters
    ///
    /// Stores the constructor as the first element in the vector, and fills the rest of the vector with the parameters
    pub fn with_params(
        constr: TypeKind,
        params: impl Iterator<Item = TypeKind>,
        span: Span,
    ) -> Self {
        let types = std::iter::once(constr).chain(params).collect();
        Self { types, span }
    }

    /// Get a [`Type`]'s type constructor (the first element in the vector)
    pub fn constr(&self) -> TypeKind {
        self.types[0].clone()
    }

    /// Get a [`Type`]'s type parameters (everything following the first element in the vector)
    pub fn params(&self) -> &[TypeKind] {
        &self.types[1..]
    }
}

/// A `flux_typesystem` type id
///
/// Types are stored in and organized by the type environment -- in order to refer to them, `TypeId`s are used.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TypeId(usize);

impl TypeId {
    pub fn new(id: usize) -> Self {
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
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TypeKind {
    Concrete(ConcreteKind),
    Int(Option<TypeId>),
    Float(Option<TypeId>),
    Ref(TypeId),
    Generic,
    Unknown,
}

impl Display for TypeKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Concrete(concrete) => write!(f, "{}", concrete),
            Self::Float(_) => write!(f, "float"),
            Self::Generic => write!(f, "generic"),
            Self::Int(_) => write!(f, "int"),
            Self::Ref(id) => write!(f, "Ref({id}"),
            Self::Unknown => write!(f, "unknown"),
        }
    }
}

// /// A `flux_typesystem` bit width
// ///
// /// The number of bits required to represent an integer
// #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
// #[repr(u8)]
// pub enum BitWidth {
//     Eight,
//     Sixteen,
//     ThirtyTwo,
//     SixtyFour,
// }

/// A `flux_typesystem` concrete kind
///
/// The kind of [`TypeKind::Concrete`]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ConcreteKind {
    Ptr(TypeId),
    Path(TinyVec<[Spur; 2]>),
    Tuple(TinyVec<[TypeId; 2]>),
}

// impl Display for ConcreteKind {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         match self {
//             Self::F32 => write!(f, "f32"),
//             Self::F64 => write!(f, "f32"),
//             _ => write!(f, "{:?}", self),
//         }
//     }
// }
