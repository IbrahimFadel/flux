use std::{collections::HashMap, fmt};

use flux_error::{FluxError, Span};
use smol_str::SmolStr;

pub mod check;
pub use check::{ErrorHandler, TypeChecker};
pub mod infer;
pub use infer::TypeEnv;

pub type TypeId = usize;

// Just... pretend i didnt do this please
#[derive(Debug, Clone)]
pub struct Spanned<T> {
	pub inner: T,
	pub span: Span,
}

pub trait Insert<T> {
	fn insert(&mut self, ty: T) -> TypeId;
}

pub struct Type(pub TypeKind);

#[derive(Debug, Clone, PartialEq)]
pub enum TypeKind {
	Concrete(ConcreteKind),
	Int(Option<TypeId>),
	Float(Option<TypeId>),
	Ref(TypeId),
	Unknown,
}

type BitSize = u32;

#[derive(Debug, Clone, PartialEq)]
pub enum ConcreteKind {
	SInt(BitSize),
	UInt(BitSize),
	F64,
	F32,
	Ident(SmolStr),
	Tuple(Vec<TypeId>),
	Func(Vec<TypeKind>, Box<TypeKind>),
}

impl Insert<Spanned<TypeKind>> for TypeEnv {
	fn insert(&mut self, ty: Spanned<TypeKind>) -> TypeId {
		let id = self.id_counter;
		self.id_counter += 1;
		self.vars.insert(id, ty);
		id
	}
}
