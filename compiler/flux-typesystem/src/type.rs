use smol_str::SmolStr;

pub type TypeId = usize;

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
	Func(Vec<TypeId>, Box<TypeId>),
}
