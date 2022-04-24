use std::collections::HashMap;

use pi_ast::OpKind;

pub type MirID = usize;

#[derive(Debug, Clone, Copy)]
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
}

#[derive(Debug, Clone)]
pub enum RValue {
	Local(MirID),
	BinOp(Binop),
	// UnaryOp,
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
pub struct Binop {
	pub lhs: Box<RValue>,
	pub op: OpKind,
	pub rhs: Box<RValue>,
}

impl Binop {
	pub fn new(lhs: Box<RValue>, op: OpKind, rhs: Box<RValue>) -> Self {
		Self { lhs, op, rhs }
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
	Br(Br),
	BrCond(BrCond),
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
