use std::collections::HashSet;

use smol_str::SmolStr;

pub type TypeId = usize;

#[derive(Debug, Clone, PartialEq)]
pub enum TypeKind {
	Concrete(ConcreteKind),
	Int(Option<TypeId>),
	Float(Option<TypeId>),
	Ref(TypeId),
	Generic((SmolStr, HashSet<SmolStr>)),
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
	Ident((SmolStr, Vec<TypeId>)),
	Tuple(Vec<TypeId>),
	Func(Vec<TypeId>, TypeId),
}
