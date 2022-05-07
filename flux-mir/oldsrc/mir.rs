use std::vec;

#[derive(Debug)]
pub struct FnDecl {
	name: String,
	params: Vec<flux_ast::FnParam>,
	pub blocks: Vec<Block>,
}

impl FnDecl {
	pub fn new(name: String, params: Vec<flux_ast::FnParam>) -> Self {
		Self {
			name,
			params,
			blocks: vec![],
		}
	}

	/*
		Creates a new block with the correct tag, but does *not* append it to the function's block list
	*/
	pub fn new_block(&self) -> Block {
		Block {
			tag: self.blocks.len(),
			instructions: vec![],
			local_count: 0,
			expecting_type: Type::I32,
		}
	}
}

#[derive(Debug)]
pub struct Block {
	tag: usize,
	instructions: Vec<Instruction>,
	local_count: usize,
	expecting_type: Type,
}

impl Block {
	pub fn lower_stmt(&mut self, stmt: &flux_ast::Stmt) {
		let instructions = match stmt {
			flux_ast::Stmt::VarDecl(x) => self.lower_var_decl(x),
			_ => (),
		};
	}

	fn lower_var_decl(&mut self, var: &flux_ast::VarDecl) {
		let ty = crate::type_expr_to_mir_type(&var.type_);
		self.expecting_type = ty.clone();
		for i in 0..var.names.len() {
			let new_loc = self.new_local(var.mut_, ty.clone());
			let tag = new_loc.tag;
			self.instructions.push(Instruction::NewLocal(new_loc));
			if let Some(vals) = &var.values {
				if vals.len() == 1 {
					self.instructions.push(Instruction::Assign(Assign::new(
						self.lower_expr(&vals[0]),
						tag,
					)));
				} else if vals.len() > 1 {
					self.instructions.push(Instruction::Assign(Assign::new(
						self.lower_expr(&vals[i]),
						tag,
					)));
				}
			}
		}
	}

	fn new_local(&mut self, mut_: bool, type_: Type) -> Local {
		Local::new(self.new_local_tag(), mut_, type_)
	}

	fn new_local_tag(&mut self) -> usize {
		self.local_count += 1;
		self.local_count - 1
	}

	fn lower_expr(&self, expr: &flux_ast::Expr) -> RValue {
		/*

			i32, u32, etc. not just intlit
		*/
		match expr {
			flux_ast::Expr::IntLit(x) => match self.expecting_type {
				Type::I64 => RValue::I64(*x as i64),
				Type::U64 => RValue::U64(*x as u64),
				Type::I32 => RValue::I32(*x as i32),
				Type::U32 => RValue::U32(*x as u32),
				Type::I16 => RValue::I16(*x as i16),
				Type::U16 => RValue::U16(*x as u16),
				Type::I8 => RValue::I8(*x as i8),
				Type::U8 => RValue::U8(*x as u8),
				_ => RValue::I32(*x as i32),
			},
			flux_ast::Expr::BinOp(binop) => self.lower_binop(binop),
			_ => RValue::Null,
		}
	}

	fn lower_binop(&self, binop: &flux_ast::BinOp) -> RValue {
		RValue::BinOp(BinOp {
			lhs: Box::from(self.lower_expr(&*binop.x)),
			op: binop.op,
			rhs: Box::from(self.lower_expr(&*binop.y)),
		})
	}
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
	BinOp(BinOp),
	UnaryOp,
	Null,
	I64(i64),
	U64(u64),
	I32(i32),
	U32(u32),
	I16(i16),
	U16(u16),
	I8(i8),
	U8(u8),
}

#[derive(Debug)]
pub struct BinOp {
	lhs: Box<RValue>,
	op: flux_ast::OpKind,
	rhs: Box<RValue>,
}

#[derive(Debug)]
pub enum Instruction {
	NewLocal(Local),
	StackAlloc(Type),
	HeapAlloc,
	Assign(Assign),
}

#[derive(Debug)]
pub struct Local {
	tag: usize,
	mut_: bool,
	type_: Type,
}

impl Local {
	pub fn new(tag: usize, mut_: bool, type_: Type) -> Self {
		Self { tag, mut_, type_ }
	}
}

#[derive(Debug)]
pub struct Assign {
	val: RValue,
	local_tag: usize,
}

impl Assign {
	pub fn new(val: RValue, local_tag: usize) -> Self {
		Self { val, local_tag }
	}
}
