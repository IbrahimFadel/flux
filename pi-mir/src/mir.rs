pub struct MirContext<'a> {
	ast: &'a AST,
	builder: MirBuilder,
	cur_block: Option<&'a mut Block>,
}

impl<'a> MirContext<'a> {
	pub fn new(ast: &'a AST) -> Self {
		Self {
			ast,
			builder: MirBuilder::new(),
			cur_block: None,
		}
	}

	pub fn set_cur_block(&mut self, b: &'a mut Block) {
		self.cur_block = Some(b);
	}

	pub fn lower_functions(&mut self) {
		for f in &self.ast.functions {
			let fndecl = self.lower_function(f);
		}
	}

	fn lower_function(&mut self, function: &pi_ast::FnDecl) -> FnDecl {
		let mut f =
			self
				.builder
				.new_function(function.name.to_string(), function.params.clone(), vec![]);
		let entry = self.builder.new_block(&mut f);
		self.cur_block = Some(entry);
		for stmt in &function.block {
			let instrs = self.lower_stmt(&stmt);
			for i in instrs {
				entry.instructions.push(i);
			}
		}
		println!("{:?}", f);
		return f;
	}

	fn lower_stmt(&mut self, stmt: &pi_ast::Stmt) -> Vec<Instruction> {
		match stmt {
			pi_ast::Stmt::VarDecl(x) => self.lower_var_decl(x),
			_ => vec![],
		}
	}

	fn lower_var_decl(&mut self, var: &pi_ast::VarDecl) -> Vec<Instruction> {
		// i32 x = 0;
		// %0 = StackAlloc(i32)
		// Assign(0, %0)
		let mut instructions = vec![];
		let ty = MirContext::type_expr_to_mir_type(&var.type_);
		for ident in &var.names {
			self
				.builder
				.new_stack_alloc(self.cur_block.as_mut().unwrap(), ty.clone());
		}
		instructions
	}

	fn type_expr_to_mir_type(ty: &pi_ast::Expr) -> Type {
		match ty {
			pi_ast::Expr::PrimitiveType(x) => match x.kind {
				pi_ast::PrimitiveKind::I64 => Type::I64,
				pi_ast::PrimitiveKind::I32 => Type::I32,
				_ => Type::I32,
			},
			_ => Type::I32,
		}
	}
}

pub struct MirBuilder {}

impl MirBuilder {
	pub fn new() -> Self {
		Self {}
	}

	pub fn new_function(
		&self,
		name: String,
		params: Vec<pi_ast::FnParam>,
		blocks: Vec<Block>,
	) -> FnDecl {
		FnDecl::new(name, params, blocks)
	}

	pub fn new_block<'a>(&self, fn_: &'a mut FnDecl) -> &'a mut Block {
		let len = fn_.blocks.len();
		fn_.blocks.push(Block::new(len, vec![]));
		&mut fn_.blocks[len]
	}

	pub fn new_stack_alloc<'a>(&self, block: &'a mut Block, ty: Type) -> &'a Instruction {
		let i = Instruction::StackAlloc(ty);
		block.instructions.push(i);
		&block.instructions[block.instructions.len() - 1]
	}
}

#[derive(Debug)]
pub enum Value {
	Instruction(Instruction),
}

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
}

#[derive(Debug)]
pub enum RValue {
	Local(Local),
	BinOp,
	UnaryOp,
	I32(i32),
}

#[derive(Debug)]
pub enum Instruction {
	StackAlloc(Type),
	HeapAlloc,
	Assign(Assign),
}

#[derive(Debug)]
pub struct Local {
	tag: usize,
	mut_: bool,
}

impl Local {
	pub fn new(tag: usize, mut_: bool) -> Self {
		Self { tag, mut_ }
	}
}

#[derive(Debug)]
pub struct Assign {
	val: RValue,
	local: Local,
}

impl Assign {
	pub fn new(val: RValue, local: Local) -> Self {
		Self { val, local }
	}
}

pub struct MirPackage {}

#[derive(Debug)]
pub struct FnDecl {
	name: String,
	params: Vec<pi_ast::FnParam>,
	blocks: Vec<Block>,
}

impl FnDecl {
	pub fn new(name: String, params: Vec<pi_ast::FnParam>, blocks: Vec<Block>) -> Self {
		Self {
			name,
			params,
			blocks,
		}
	}

	pub fn new_block(&mut self) -> &mut Block {
		let len = self.blocks.len();
		self.blocks.push(Block::new(len, vec![]));
		&mut self.blocks[len]
	}
}

#[derive(Debug)]
pub struct Block {
	tag: usize,
	pub instructions: Vec<Instruction>,
}

impl Block {
	pub fn new(tag: usize, instructions: Vec<Instruction>) -> Self {
		Self { tag, instructions }
	}

	pub fn new_stack_alloc(&mut self, ty: Type, mut_: bool) -> Local {
		let i = Instruction::StackAlloc(ty);
		self.instructions.push(i);
		return Local::new(self.instructions.len() - 1, mut_);
	}

	pub fn new_assign(&mut self, val: RValue, local: Local) {
		let i = Instruction::Assign(Assign::new(val, local));
		self.instructions.push(i);
		// return self.instructions.len() - 1;
	}
}

// use smol_str::SmolStr;

// #[derive(Debug)]
// pub struct MIRContext {
// 	functions: Vec<FnDecl>,
// }

// impl MIRContext {
// 	pub fn new() -> Self {
// 		Self { functions: vec![] }
// 	}
// 	pub fn new_function(&mut self, type_: Type, name: SmolStr, params: Vec<Type>) -> &FnDecl {
// 		let f = FnDecl::new(type_, name, params);
// 		self.functions.push(f);
// 		return &self.functions[self.functions.len() - 1];
// 	}
// }

// #[derive(Debug, Clone)]
// pub enum Type {
// 	I64,
// 	U64,
// 	I32,
// 	U32,
// 	I16,
// 	U16,
// 	I8,
// 	U8,
// 	F64,
// 	F32,
// 	Bool,
// 	Void,
// }

// #[derive(Debug)]
// pub struct FnDecl {
// 	type_: Type,
// 	name: SmolStr,
// 	params: Vec<Type>,
// 	blocks: Vec<BasicBlock>,
// }

// impl FnDecl {
// 	pub fn new(type_: Type, name: SmolStr, params: Vec<Type>) -> Self {
// 		Self {
// 			type_,
// 			name,
// 			params,
// 			blocks: vec![],
// 		}
// 	}
// }

// #[derive(Debug)]
// pub enum Instruction {
// 	StackAlloc(Type),
// 	HeapAlloc,
// 	Assign(Assign),
// }

// #[derive(Debug)]
// pub struct Assign {
// 	lhs: Box<Local>,
// 	rhs: RValue,
// }

// impl Assign {
// 	pub fn new(lhs: Box<Local>, rhs: RValue) -> Self {
// 		Self { lhs, rhs }
// 	}
// }

// #[derive(Debug)]
// pub enum Terminator {
// 	Return,
// 	Goto,
// 	// Call
// }

// #[derive(Debug)]
// pub enum RValue {
// 	BinOp,
// 	UnaryOp,
// 	I32(i32),
// }

// pub type Local = usize;

// #[derive(Debug)]
// pub struct BasicBlock {
// 	statements: Vec<Instruction>,
// 	terminator: Terminator,
// }

use std::{
	borrow::BorrowMut,
	ops::{Deref, DerefMut},
};

use pi_ast::AST;
