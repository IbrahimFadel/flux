use std::{collections::HashMap, fmt};

pub type MirID = usize;

#[derive(Debug, Clone)]
pub enum Type {
	I64,
	U64,
	I32,
	U32,
	I16,
	U16,
	I8,
	U8,
	F64,
	F32,
	Bool,
	Void,
	Vector(VectorTy),
	StructTy(StructType),
	Ident(Ident),
	Ptr(Ptr),
}

pub type Ident = String;
pub type Ptr = Box<Type>;

impl fmt::Display for Type {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Type::Ptr(ptr) => write!(f, "{}*", *ptr),
			Type::Ident(ident) => write!(f, "{ident}"),
			Type::StructTy(struct_ty) => {
				write!(f, "\\{{ ")?;
				for (i, ty) in struct_ty.iter().enumerate() {
					write!(f, "{}", ty)?;
					if i != struct_ty.len() - 1 {
						write!(f, ", ")?;
					}
				}
				write!(f, " \\}}")?;
				Ok(())
			}
			Type::Vector(vec) => write!(f, "[ {} x {} ]", vec.count, *vec.ty),
			_ => write!(f, "{:?}", self),
		}
	}
}

#[derive(Debug, Clone)]
pub struct VectorTy {
	pub count: usize,
	pub ty: Box<Type>,
}

impl VectorTy {
	pub fn new(count: usize, ty: Box<Type>) -> Self {
		Self { count, ty }
	}
}

#[derive(Debug, Clone)]
pub enum RValue {
	Local(MirID),
	I64(i64),
	I32(i32),
	I16(i16),
	I8(i8),
	U64(u64),
	U32(u32),
	U16(u16),
	U8(u8),
	F64(f64),
	F32(f32),
}

pub type StructType = Vec<Type>;

#[derive(Debug)]
pub struct FnDecl {
	pub name: String,
	pub blocks: Vec<Block>,
	pub block_count: usize,
	pub instr_count: usize,
	pub cur_block_id: MirID,
	pub locals: HashMap<String, MirID>,
	pub local_types: HashMap<MirID, Type>,
}

impl FnDecl {
	pub fn new(name: String) -> Self {
		Self {
			name: name,
			blocks: vec![],
			block_count: 0,
			instr_count: 0,
			cur_block_id: 0,
			locals: HashMap::new(),
			local_types: HashMap::new(),
		}
	}
}

#[derive(Debug, Clone)]
pub struct Block {
	pub id: MirID,
	pub instrs: Vec<Instruction>,
}

#[derive(Debug, Clone)]
pub enum Instruction {
	Alloca(Alloca),
	Store(Store),
	Load(Load),
	Call(Call),
	Br(Br),
	BrCond(BrCond),
	Ret(Ret),
	Add(Add),
	CmpEq(CmpEq),
	IndexAccess(IndexAccess),
	PtrCast(PtrCast),
}

#[derive(Debug, Clone)]
pub struct PtrCast {
	pub id: MirID,
	pub ptr: MirID,
	pub to_ty: Type,
}

impl PtrCast {
	pub fn new(id: MirID, ptr: MirID, to_ty: Type) -> Self {
		Self { id, ptr, to_ty }
	}
}

#[derive(Debug, Clone)]
pub struct IndexAccess {
	pub id: MirID,
	pub ptr: MirID,
	pub idx: u32,
}

impl IndexAccess {
	pub fn new(id: MirID, ptr: MirID, idx: u32) -> Self {
		Self { id, ptr, idx }
	}
}

#[derive(Debug, Clone)]
pub struct Call {
	pub id: MirID,
	pub callee: RValue,
	pub args: Vec<RValue>,
}

impl Call {
	pub fn new(id: MirID, callee: RValue, args: Vec<RValue>) -> Self {
		Self { id, callee, args }
	}
}

#[derive(Debug, Clone)]
pub struct CmpEq {
	pub id: MirID,
	pub ty: Type,
	pub lhs: RValue,
	pub rhs: RValue,
}

impl CmpEq {
	pub fn new(id: MirID, ty: Type, lhs: RValue, rhs: RValue) -> Self {
		Self { id, ty, lhs, rhs }
	}
}

#[derive(Debug, Clone)]
pub struct Add {
	pub id: MirID,
	pub lhs: RValue,
	pub rhs: RValue,
}

impl Add {
	pub fn new(id: MirID, lhs: RValue, rhs: RValue) -> Self {
		Self { id, lhs, rhs }
	}
}

#[derive(Debug, Clone)]
pub struct Ret {
	pub id: MirID,
	pub ty: Type,
	pub val: Option<RValue>,
}

impl Ret {
	pub fn new(id: MirID, ty: Type, val: Option<RValue>) -> Self {
		Self { id, ty, val }
	}
}

#[derive(Debug, Clone)]
pub struct Br {
	pub id: MirID,
	pub to: MirID,
}

impl Br {
	pub fn new(id: MirID, to: MirID) -> Self {
		Self { id, to }
	}
}

#[derive(Debug, Clone)]
pub struct BrCond {
	pub id: MirID,
	pub cond: RValue,
	pub then: MirID,
	pub else_: MirID,
}

impl BrCond {
	pub fn new(id: MirID, cond: RValue, then: MirID, else_: MirID) -> Self {
		Self {
			id,
			cond,
			then,
			else_,
		}
	}
}

#[derive(Debug, Clone)]
pub struct Load {
	pub id: MirID,
	pub ty: Type,
	pub ptr: MirID,
}

impl Load {
	pub fn new(id: MirID, ty: Type, ptr: MirID) -> Self {
		Self { id, ty, ptr }
	}
}

#[derive(Debug, Clone)]
pub struct Store {
	pub id: MirID,
	pub ty: Type,
	pub val: RValue,
	pub ptr: MirID,
}

impl Store {
	pub fn new(id: MirID, ty: Type, val: RValue, ptr: MirID) -> Self {
		Self { id, ty, val, ptr }
	}
}

#[derive(Debug, Clone)]
pub struct Alloca {
	pub id: MirID,
	pub ty: Type,
}

impl Alloca {
	pub fn new(id: MirID, ty: Type) -> Self {
		Self { id, ty }
	}
}
