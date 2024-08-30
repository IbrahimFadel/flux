use std::{collections::HashSet, ops::Deref};

use flux_id::id::{self, InPkg};
use flux_util::{Path, Word};

use crate::ThisCtx;

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub struct Type {
    pub kind: TypeKind,
    pub restrictions: Vec<Restriction>,
}

impl Type {
    pub const fn new(kind: TypeKind, restrictions: Vec<Restriction>) -> Self {
        Self { kind, restrictions }
    }

    pub const fn unknown() -> Self {
        Self {
            kind: TypeKind::Unknown,
            restrictions: vec![],
        }
    }

    pub const fn never() -> Self {
        Self {
            kind: TypeKind::Never,
            restrictions: vec![],
        }
    }

    pub const fn path(path: Path<Word, Type>) -> Self {
        Self {
            kind: TypeKind::Concrete(ConcreteKind::Path(path)),
            restrictions: vec![],
        }
    }

    pub const fn tuple(types: Vec<Type>) -> Self {
        Self {
            kind: TypeKind::Concrete(ConcreteKind::Tuple(types)),
            restrictions: vec![],
        }
    }

    pub const fn unit() -> Self {
        Self::tuple(vec![])
    }

    pub const fn r#ref(tid: id::Ty) -> Self {
        Self {
            kind: TypeKind::Ref(tid),
            restrictions: vec![],
        }
    }

    pub fn array(ty: Type, n: u64) -> Self {
        Self {
            kind: TypeKind::Concrete(ConcreteKind::Array(Box::new(ty), n)),
            restrictions: vec![],
        }
    }

    pub fn ptr(ty: Type) -> Self {
        Self {
            kind: TypeKind::Concrete(ConcreteKind::Ptr(Box::new(ty))),
            restrictions: vec![],
        }
    }

    pub const fn function(signature: FnSignature) -> Self {
        Self {
            kind: TypeKind::Concrete(ConcreteKind::Fn(signature)),
            restrictions: vec![],
        }
    }

    pub fn r#address(ty: Type) -> Self {
        Self {
            kind: TypeKind::Concrete(ConcreteKind::Addr(Box::new(ty))),
            restrictions: vec![],
        }
    }

    pub const fn generic(name: Word, restrictions: Vec<TraitRestriction>) -> Self {
        Self {
            kind: TypeKind::Generic(name, restrictions),
            restrictions: vec![],
        }
    }

    pub fn this_path(path: Path<Word, Type>, potential_this_ctx: Vec<ThisCtx>) -> Self {
        Self {
            kind: TypeKind::ThisPath(ThisPath::new(path, potential_this_ctx)),
            restrictions: vec![],
        }
    }

    pub const fn int() -> Self {
        Self {
            kind: TypeKind::Int,
            restrictions: vec![],
        }
    }

    pub fn with_trait_restrictions(self, restrictions: Vec<TraitRestriction>) -> Self {
        Self {
            kind: self.kind,
            restrictions: restrictions
                .into_iter()
                .map(|restriction| Restriction::Trait(restriction))
                .collect(),
        }
    }

    pub fn has_field(&self, name: &Word) -> bool {
        self.restrictions
            .iter()
            .find(|restriction| match restriction {
                Restriction::Field(field_name) => field_name == name,
                _ => false,
            })
            .is_some()
    }

    pub fn generics_used(&self, set: &mut HashSet<Word>) {
        match &self.kind {
            TypeKind::Concrete(concrete_kind) => match concrete_kind {
                ConcreteKind::Array(ty, _) => ty.generics_used(set),
                ConcreteKind::Ptr(ty) => ty.generics_used(set),
                ConcreteKind::Addr(ty) => ty.generics_used(set),
                ConcreteKind::Path(path) => path.args.iter().for_each(|ty| ty.generics_used(set)),
                ConcreteKind::Tuple(types) => types.iter().for_each(|ty| ty.generics_used(set)),
                ConcreteKind::Fn(signature) => {
                    signature.0.iter().for_each(|ty| ty.generics_used(set))
                }
            },
            TypeKind::Generic(name, _) => {
                set.insert(*name);
            }
            TypeKind::Ref(_)
            | TypeKind::Int
            | TypeKind::Float
            | TypeKind::ThisPath(_)
            | TypeKind::Never
            | TypeKind::Unknown => {}
        }
    }

    pub(crate) fn push_restriction(&mut self, restriction: Restriction) {
        self.restrictions.push(restriction);
    }

    pub fn set_kind(self, kind: TypeKind) -> Self {
        Self {
            kind,
            restrictions: self.restrictions,
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub enum TypeKind {
    Concrete(ConcreteKind),
    Generic(Word, Vec<TraitRestriction>),
    ThisPath(ThisPath), // `This` or associated type like `This::Foo`
    Ref(id::Ty),
    Int,
    Float,
    Never,
    Unknown,
}

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub struct ThisPath {
    pub path: Path<Word, Type>,
    pub potential_this_ctx: Vec<ThisCtx>,
}

impl ThisPath {
    pub const fn new(path: Path<Word, Type>, potential_this_ctx: Vec<ThisCtx>) -> Self {
        Self {
            path,
            potential_this_ctx,
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub enum ConcreteKind {
    Array(Box<Type>, u64),
    Ptr(Box<Type>),
    Addr(Box<Type>),
    Path(Path<Word, Type>),
    Tuple(Vec<Type>),
    Fn(FnSignature),
}

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub enum Restriction {
    Equals(id::Ty),
    EqualsOneOf(Vec<TypeKind>),
    AssocTypeOf(id::Ty, TraitRestriction, Word),
    // PossibleAssocTypes(Vec<TypeKind>),
    Field(Word),
    Trait(TraitRestriction),
}

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub struct TraitRestriction {
    pub(super) trait_id: InPkg<id::TraitDecl>,
    pub(super) args: Vec<id::Ty>,
}

impl TraitRestriction {
    pub fn new(trait_id: InPkg<id::TraitDecl>, args: Vec<id::Ty>) -> Self {
        Self { trait_id, args }
    }
}

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

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
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
