use std::{collections::HashSet, ops::Deref};

use flux_diagnostics::ice;
use flux_id::id;
use flux_util::{Path, Word};
use tracing::warn;

use crate::r#trait::ThisCtx;

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub struct Typed<T> {
    pub tid: id::Ty,
    pub inner: T,
}

impl<T> Deref for Typed<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

pub trait WithType {
    fn with_type(self, tid: id::Ty) -> Typed<Self>
    where
        Self: Sized,
    {
        Typed { tid, inner: self }
    }
}

impl WithType for id::Expr {}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Type {
    Concrete(ConcreteKind),
    Generic(Generic),
    ThisPath(ThisPath),
    Int(Option<id::Ty>),
    Float(Option<id::Ty>),
    Ref(id::Ty),
    Never,
    Unknown,
}

impl Type {
    pub const fn path(path: Path<Word, Type>) -> Self {
        Self::Concrete(ConcreteKind::Path(path))
    }

    pub fn this_path<A>(path: Path<Word, A>, this_ctx: ThisCtx) -> Self {
        Self::ThisPath(ThisPath::new(path.discard_args(), this_ctx))
    }

    pub const fn tuple(types: Vec<Type>) -> Self {
        Self::Concrete(ConcreteKind::Tuple(types))
    }

    pub fn array(ty: Type, n: u64) -> Self {
        Self::Concrete(ConcreteKind::Array(Box::new(ty), n))
    }

    pub fn ptr(ty: Type) -> Self {
        Self::Concrete(ConcreteKind::Ptr(Box::new(ty)))
    }

    pub const fn unit() -> Self {
        Self::Concrete(ConcreteKind::Tuple(vec![]))
    }

    pub const fn int() -> Self {
        Self::Int(None)
    }

    pub const fn float() -> Self {
        Self::Float(None)
    }

    pub fn generics_used(&self, set: &mut HashSet<Word>) {
        match self {
            Type::Concrete(concrete_kind) => match concrete_kind {
                ConcreteKind::Array(ty, _) => ty.generics_used(set),
                ConcreteKind::Ptr(ty) => ty.generics_used(set),
                ConcreteKind::Path(path) => path.args.iter().for_each(|ty| ty.generics_used(set)),
                ConcreteKind::Tuple(types) => types.iter().for_each(|ty| ty.generics_used(set)),
            },
            Type::Generic(generic) => {
                set.insert(generic.name);
            }
            Type::Ref(_) => ice("`Type::Ref` should not be constructed before lowering package bodies, and generics should only be checked before then"),
            Type::Int(_) | Type::Float(_) | Type::ThisPath(_) | Type::Never | Type::Unknown => {}
        }
    }

    pub fn set_this_ctx(&mut self, this_ctx: ThisCtx) {
        // warn!("`Type::set_this_ctx` needs to take `TEnv` be fully implemented");
        match self {
            Type::ThisPath(this_path) => {
                this_path.ctx = this_ctx;
            }
            _ => {}
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum ConcreteKind {
    Array(Box<Type>, u64),
    Ptr(Box<Type>),
    Path(Path<Word, Type>),
    Tuple(Vec<Type>),
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Generic {
    pub(super) name: Word,
    bounds: Vec<Path<Word, Type>>,
}

impl Generic {
    pub fn new(name: Word, bounds: Vec<Path<Word, Type>>) -> Self {
        Self { name, bounds }
    }
}
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ThisPath {
    pub(super) path: Path<Word>,
    pub(super) ctx: ThisCtx,
}

impl ThisPath {
    pub fn new(path: Path<Word>, ctx: ThisCtx) -> Self {
        Self { path, ctx }
    }
}

#[derive(Debug, Clone)]
pub struct FnSignature(Vec<Type>);

impl FnSignature {
    pub fn new(parameters: impl Iterator<Item = Type>, return_ty: Type) -> Self {
        Self(parameters.chain(std::iter::once(return_ty)).collect())
    }

    pub fn from_type_ids(type_ids: impl Iterator<Item = Type>) -> Self {
        Self(type_ids.collect())
    }

    pub fn parameters(&self) -> &[Type] {
        self.0.get(..self.0.len() - 1).unwrap_or(&[])
    }

    pub fn return_ty(&self) -> &Type {
        self.0.last().unwrap()
    }
}
