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

#[derive(Debug, Clone)]
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
	Unit,
}

impl Insert<Spanned<TypeKind>> for TypeEnv {
	fn insert(&mut self, ty: Spanned<TypeKind>) -> TypeId {
		let id = self.id_counter;
		self.id_counter += 1;
		self.vars.insert(id, ty);
		id
	}
}

impl fmt::Display for TypeKind {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::Concrete(t) => write!(f, "{}", t),
			Self::Ref(id) => write!(f, "Ref({})", id),
			Self::Int(id) => {
				if let Some(id) = id {
					write!(f, "Int({})", id)
				} else {
					write!(f, "Int")
				}
			}
			Self::Float(id) => {
				if let Some(id) = id {
					write!(f, "Float({})", id)
				} else {
					write!(f, "Float")
				}
			}
			Self::Unknown => write!(f, "Unknown"),
		}
	}
}

impl fmt::Display for ConcreteKind {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::SInt(n) => write!(f, "i{}", n),
			Self::UInt(n) => write!(f, "u{}", n),
			Self::F64 => write!(f, "f64"),
			Self::F32 => write!(f, "f32"),
			Self::Ident(name) => write!(f, "{}", name),
			Self::Unit => write!(f, "()"),
		}
	}
}
