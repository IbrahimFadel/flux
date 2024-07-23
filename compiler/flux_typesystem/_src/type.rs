use std::ops::Deref;
use std::{fmt::Display, iter::once};

use flux_span::{Interner, Word};
use itertools::Itertools;

use crate::r#trait::ThisCtx;
use crate::{
    r#trait::{ApplicationId, TraitRestriction},
    TraitId,
};

/// A `flux_typesystem` type id
///
/// Types are stored in and organized by the type environment -- in order to refer to them, `TypeId`s are used.
#[repr(transparent)]
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

#[derive(Debug, Clone)]
pub enum TypeKind {
    ThisPath(ThisPath),
    Concrete(ConcreteKind),
    Int(Option<TypeId>),
    Float(Option<TypeId>),
    Ref(TypeId),
    Generic(Generic),
    Never,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct ThisPath {
    pub segments: Vec<Word>,
    pub this_ctx: ThisCtx,
}

impl ThisPath {
    pub fn new(segments: Vec<Word>, this_ctx: ThisCtx) -> Self {
        Self { segments, this_ctx }
    }
}

#[derive(Debug, Clone)]
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
            Self::ThisPath(_) => write!(f, "todo"),
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
#[derive(Debug, Clone)]
pub enum ConcreteKind {
    Array(TypeId, u64),
    Ptr(TypeId),
    Path(Path),
    Tuple(Vec<TypeId>),
}

#[derive(Debug, Clone)]
pub struct Path {
    pub segments: Vec<Word>,
    pub generic_args: Vec<TypeId>,
}

impl Path {
    const INT_PATHS: [&'static str; 8] = ["u64", "u32", "u16", "u8", "s64", "s32", "s16", "s8"];
    const FLOAT_PATHS: [&'static str; 2] = ["f64", "f32"];

    pub fn new(segments: Vec<Word>, generic_args: Vec<TypeId>) -> Self {
        Self {
            segments,
            generic_args,
        }
    }

    pub fn is_int(&self, interner: &'static Interner) -> bool {
        if self.segments.len() == 1 {
            let name = self.segments[0];
            Self::INT_PATHS
                .iter()
                .find(|path| interner.get_or_intern_static(path) == name)
                .is_some()
        } else {
            false
        }
    }

    pub fn is_float(&self, interner: &'static Interner) -> bool {
        if self.segments.len() == 1 {
            let name = self.segments[0];
            Self::FLOAT_PATHS
                .iter()
                .find(|path| interner.get_or_intern_static(path) == name)
                .is_some()
        } else {
            false
        }
    }

    pub fn to_string(&self, interner: &'static Interner) -> String {
        self.segments
            .iter()
            .map(|seg| interner.resolve(seg))
            .join("::")
    }
}

impl Display for ConcreteKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Array(ty, n) => write!(f, "[{ty}; {n}]"),
            Self::Path(path) => write!(f, "{path:?}"),
            Self::Ptr(ptr) => write!(f, "{ptr}*"),
            Self::Tuple(types) => {
                write!(f, "({})", types.iter().map(|id| id.to_string()).join(", "))
            }
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub struct Typed<T> {
    pub tid: TypeId,
    pub inner: T,
}

impl<T> Deref for Typed<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

pub trait WithType {
    fn with_type(self, tid: TypeId) -> Typed<Self>
    where
        Self: Sized,
    {
        Typed { tid, inner: self }
    }
}

#[derive(Debug, Clone)]
pub struct FnSignature(Vec<TypeId>);

impl FnSignature {
    pub fn new(parameters: impl Iterator<Item = TypeId>, return_ty: TypeId) -> Self {
        Self(parameters.chain(once(return_ty)).collect())
    }

    pub fn from_type_ids(type_ids: impl Iterator<Item = TypeId>) -> Self {
        Self(type_ids.collect())
    }

    pub fn parameters(&self) -> &[TypeId] {
        self.0.get(..self.0.len() - 1).unwrap_or(&[])
    }

    pub fn return_ty(&self) -> &TypeId {
        self.0.last().unwrap()
    }
}
