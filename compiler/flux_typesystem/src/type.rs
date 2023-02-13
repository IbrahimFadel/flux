use std::fmt::Display;

use flux_proc_macros::Locatable;
use flux_span::WithSpan;
use lasso::Spur;

use crate::intern::{Interner, Key};

/// A `flux_typesystem` type
///
/// Types consist of a constructor and parameters
///
/// The type `Foo<i32, T, Bar>` has the constructor `Foo` and parameters `[i32, T, Bar]`
///
/// Types always have a constructor, but not always parameters, as such we can store all the information in one vector rather than two to save memory.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Locatable)]
pub struct Type(Vec<Key>);

impl Type {
    /// Create a new [`Type`] with only a constructor
    ///
    /// Stores the constructor as the first element in the vector
    pub fn new(constr: TypeKind, type_interner: &mut Interner) -> Self {
        let types = vec![type_interner.intern(constr); 1];
        Self(types)
    }

    /// Create a new [`Type`] with a constructor and parameters
    ///
    /// Stores the constructor as the first element in the vector, and fills the rest of the vector with the parameters
    pub fn with_params(
        constr: TypeKind,
        params: impl Iterator<Item = TypeKind>,
        type_interner: &mut Interner,
    ) -> Self {
        let types = std::iter::once(constr)
            .chain(params)
            .map(|kind| type_interner.intern(kind))
            .collect();
        Self(types)
    }

    /// Create a new [`Type`] with a constructor and parameters
    ///
    /// Stores the constructor as the first element in the vector, and fills the rest of the vector with the parameters
    pub fn with_params_as_keys(
        constr: TypeKind,
        params: impl Iterator<Item = Key>,
        type_interner: &mut Interner,
    ) -> Self {
        let types = std::iter::once(type_interner.intern(constr))
            .chain(params)
            .collect();
        Self(types)
    }

    /// Get a [`Type`]'s type constructor (the first element in the vector)
    pub fn constr(&self) -> Key {
        self.0[0].clone()
    }

    /// Get a [`Type`]'s type parameters (everything following the first element in the vector)
    pub fn params(&self) -> Option<&[Key]> {
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
    // Generic(Vec<(Spur, Vec<TypeId>)>),
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

/// A `flux_typesystem` concrete kind
///
/// The kind of [`TypeKind::Concrete`]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ConcreteKind {
    Array(TypeId, u32),
    Ptr(TypeId),
    Path(Spur),
    Tuple(Vec<TypeId>),
    /// A vec of all the fields and their types
    /// We hold on to the field names because its necessary for checking if the field names are correct in struct initialization expressions
    Struct(StructConcreteKind),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StructConcreteKind {
    pub generic_params: Vec<TypeId>,
    pub fields: Vec<(Spur, TypeId)>,
}

impl StructConcreteKind {
    pub const EMPTY: Self = Self {
        generic_params: vec![],
        fields: vec![],
    };

    pub fn new(generic_params: Vec<TypeId>, fields: Vec<(Spur, TypeId)>) -> Self {
        Self {
            generic_params,
            fields,
        }
    }
}
