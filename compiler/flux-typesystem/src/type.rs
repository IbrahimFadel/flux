use std::collections::HashSet;

use lasso::Spur;

pub type TypeId = usize;

#[derive(Debug, Clone, PartialEq)]
pub enum TypeKind {
	Concrete(ConcreteKind),
	Int(Option<TypeId>),
	Float(Option<TypeId>),
	Ref(TypeId),
	Generic((Spur, HashSet<(Spur, Vec<TypeId>)>)),
	Unknown,
}

type BitSize = u32;

#[derive(Debug, Clone, PartialEq)]
pub enum ConcreteKind {
	SInt(BitSize),
	UInt(BitSize),
	F64,
	F32,
	Ptr(TypeId),
	Ident((Spur, Vec<TypeId>)),
	Tuple(Vec<TypeId>),
	Func(Vec<TypeId>, TypeId),
}
