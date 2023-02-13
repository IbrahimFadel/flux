use std::fmt::Display;

use flux_proc_macros::Locatable;
use flux_span::{FileSpanned, WithSpan};
use lasso::Spur;

use crate::hir::Visibility;

/// A `flux_typesystem` type
///
/// Types consist of a constructor and parameters
///
/// The type `Foo<i32, T, Bar>` has the constructor `Foo` and parameters `[i32, T, Bar]`
///
/// Types always have a constructor, but not always parameters, as such we can store all the information in one vector rather than two to save memory.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Locatable)]
pub struct Type {
    pub constr: TypeKind,
    pub params: Vec<Type>,
}

impl Type {
    /// Create a new [`Type`] with only a constructor
    pub fn new(constr: TypeKind) -> Self {
        Self {
            constr,
            params: vec![],
        }
    }

    /// Create a new [`Type`] with a constructor and parameters
    pub fn with_params(constr: TypeKind, params: Vec<Type>) -> Self {
        Self { constr, params }
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
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Locatable)]
pub enum TypeKind {
    Concrete(ConcreteKind),
    Int(Option<TypeId>),
    Float(Option<TypeId>),
    Ref(TypeId),
    Generic(Spur),
    Unknown,
}

impl Display for TypeKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Concrete(concrete) => write!(f, "{concrete}"),
            Self::Float(_) => write!(f, "float"),
            Self::Generic(name) => write!(f, "generic<{:?}>", name),
            Self::Int(_) => write!(f, "int"),
            Self::Ref(id) => write!(f, "Ref({id}"),
            Self::Unknown => write!(f, "unknown"),
        }
    }
}

impl Display for ConcreteKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Array(t, n) => write!(f, "[{}; {}]", t, n),
            Self::Path(path) => write!(f, "{:?}", path),
            Self::Ptr(ptr) => write!(f, "*'{}", ptr),
            Self::Tuple(_) => write!(f, "()"),
            Self::Struct(_) => write!(f, "todo (lazy pos)"),
        }
    }
}

/// A `flux_typesystem` concrete kind
///
/// The kind of [`TypeKind::Concrete`]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ConcreteKind {
    Ptr(TypeId),
    Path(Spur),
    Tuple(Vec<TypeId>),
    Array(TypeId, u32),
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

#[derive(Debug)]
pub struct FnSignature {
    visibility: Visibility,
    param_types: Vec<FileSpanned<Type>>,
    return_ty: FileSpanned<Type>,
}

impl FnSignature {
    pub fn new(
        visibility: Visibility,
        param_types: Vec<FileSpanned<Type>>,
        return_ty: FileSpanned<Type>,
    ) -> Self {
        Self {
            visibility,
            param_types,
            return_ty,
        }
    }
}
