use std::{cell::RefCell, collections::HashMap, fmt, rc::Rc};

use smol_str::SmolStr;

pub mod builder;
mod print;

pub struct MirModule {
	functions: Vec<Rc<RefCell<FnDecl>>>,
}

impl MirModule {
	pub fn new_function<'a>(
		&mut self,
		name: SmolStr,
		params: Vec<FnParam>,
		return_ty: Type,
	) -> Rc<RefCell<FnDecl>> {
		let f = RefCell::new(FnDecl::new(name, return_ty, params));
		self.functions.push(Rc::new(f));
		self.functions.last().unwrap().clone()
	}
}

impl Default for MirModule {
	fn default() -> Self {
		Self { functions: vec![] }
	}
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct MirID(usize);
type BitSize = u32;

#[derive(Debug, Clone)]
pub enum Type {
	Int(BitSize),
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

#[derive(Debug, Clone)]
pub struct VectorTy {
	pub count: u32,
	pub ty: Box<Type>,
}

impl VectorTy {
	pub fn new(count: u32, ty: Box<Type>) -> Self {
		Self { count, ty }
	}
}

#[derive(Debug, Clone)]
pub enum RValue {
	Local(MirID),
	Int(Int),
	F64(f64),
	F32(f32),
	Unit,
}

#[derive(Debug, Clone)]
pub struct Int {
	pub n: u64,
	pub size: BitSize,
}

pub type StructType = Vec<Type>;

#[derive(Debug)]
pub struct FnDecl {
	pub name: SmolStr,
	pub ret_ty: Type,
	pub params: Vec<FnParam>,
	pub blocks: Vec<Rc<RefCell<Block>>>,
	id_count: usize,
}

impl FnDecl {
	pub fn new(name: SmolStr, ret_ty: Type, params: Vec<FnParam>) -> Self {
		Self {
			name: name,
			ret_ty,
			params,
			blocks: vec![],
			id_count: 0,
		}
	}

	pub fn append_new_block(&mut self) -> Rc<RefCell<Block>> {
		let block = Block {
			id: BlockID(self.id_count),
			instrs: vec![],
			terminator: None,
			locals: HashMap::new(),
			local_types: HashMap::new(),
			id_count: 0,
		};
		self.id_count += 1;
		self.blocks.push(Rc::new(RefCell::new(block)));
		self.blocks.last().unwrap().clone()
	}

	pub fn new_block(&mut self) -> Rc<RefCell<Block>> {
		let block = Block {
			id: BlockID(self.id_count),
			instrs: vec![],
			terminator: None,
			locals: HashMap::new(),
			local_types: HashMap::new(),
			id_count: 0,
		};
		self.id_count += 1;
		Rc::new(RefCell::new(block))
	}
}

#[derive(Debug)]
pub struct FnParam {
	pub name: SmolStr,
	pub ty: Type,
}

impl FnParam {
	pub fn new(name: SmolStr, ty: Type) -> Self {
		Self { name, ty }
	}
}

#[derive(Debug, Clone, Copy)]
pub struct BlockID(pub usize);

#[derive(Debug, Clone)]
pub struct Block {
	pub id: BlockID,
	pub instrs: Vec<Instruction>,
	pub terminator: Option<Instruction>,
	pub locals: HashMap<SmolStr, MirID>,
	pub local_types: HashMap<MirID, Type>,
	pub id_count: usize,
}

#[derive(Debug, Clone)]
pub enum Instruction {
	StackAlloc(StackAlloc),
	Store(Store),
	Load(Load),
	Call(Call),
	Br(Br),
	BrNZ(BrNZ),
	Ret(Ret),
	Add(Add),
	ICmp(ICmp),
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
	pub ty: Type,
	pub ptr: MirID,
	pub idx: u32,
}

impl IndexAccess {
	pub fn new(id: MirID, ty: Type, ptr: MirID, idx: u32) -> Self {
		Self { id, ty, ptr, idx }
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
pub struct ICmp {
	pub id: MirID,
	pub kind: ICmpKind,
	pub lhs: RValue,
	pub rhs: RValue,
}

#[derive(Debug, Clone)]
pub enum ICmpKind {
	Eq,
	ULt,
	SLt,
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
	pub val: Option<RValue>,
}

#[derive(Debug, Clone)]
pub struct Br {
	pub to: BlockID,
}

#[derive(Debug, Clone)]
pub struct BrNZ {
	pub val: RValue,
	pub then: BlockID,
	pub else_: BlockID,
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
	pub val: RValue,
	pub ptr: MirID,
}

#[derive(Debug, Clone)]
pub struct StackAlloc {
	pub id: MirID,
	pub ty: Type,
}
