use pi_ast::{Expr, OpKind, PrimitiveKind, Stmt, AST};

struct MIRModule {
	functions: Vec<FnDecl>,
}

#[derive(Debug)]
struct FnDecl {
	name: String,
	blocks: Vec<Block>,
	block_count: usize,
	cur_block_id: MirID,
}

impl FnDecl {
	fn new_block(&mut self) -> Block {
		let id = self.block_count;
		self.block_count += 1;
		Block {
			id,
			instr_count: 0,
			instrs: vec![],
		}
	}

	fn append_new_block(&mut self) -> MirID {
		let id = self.block_count;
		self.block_count += 1;
		self.blocks.push(Block {
			id,
			instr_count: 0,
			instrs: vec![],
		});
		return id;
	}

	fn append_existing_block(&mut self, bb: Block) {
		self.blocks.push(bb);
	}

	fn get_cur_block(&mut self) -> &mut Block {
		for block in &mut self.blocks {
			if block.id == self.cur_block_id {
				return block;
			}
		}
		panic!()
	}

	fn build_alloca(&mut self, ty: Type) -> MirID {
		let id = self.get_cur_block().instr_count;
		self.get_cur_block().instr_count += 1;
		let alloca = Alloca { ty, id };
		self
			.get_cur_block()
			.instrs
			.push(Instruction::Alloca(alloca));
		return id;
	}

	fn build_store(&mut self, val: RValue, ptr: MirID) -> MirID {
		let id = self.get_cur_block().instr_count;
		self.get_cur_block().instr_count += 1;
		let store = Store { val, ptr, id };
		self.get_cur_block().instrs.push(Instruction::Store(store));
		return id;
	}

	fn build_load(&mut self, ptr: MirID) -> MirID {
		let id = self.get_cur_block().instr_count;
		self.get_cur_block().instr_count += 1;
		let load = Load { ptr, id };
		self.get_cur_block().instrs.push(Instruction::Load(load));
		return id;
	}

	fn build_brcond(&mut self, cond: RValue, then: MirID, else_: MirID) -> MirID {
		let id = self.get_cur_block().instr_count;
		self.get_cur_block().instr_count += 1;
		let brcond = BrCond {
			id,
			cond,
			then,
			else_,
		};
		self
			.get_cur_block()
			.instrs
			.push(Instruction::BrCond(brcond));
		return id;
	}

	fn build_br(&mut self, to: MirID) -> MirID {
		let id = self.get_cur_block().instr_count;
		self.get_cur_block().instr_count += 1;
		let br = Br { id, to };
		self.get_cur_block().instrs.push(Instruction::Br(br));
		return id;
	}

	fn lower_stmt(&mut self, stmt: &pi_ast::Stmt) {
		match stmt {
			Stmt::VarDecl(var) => self.lower_var(var),
			Stmt::If(if_stmt) => self.lower_if(if_stmt),
			_ => (),
		}
	}

	fn lower_if(&mut self, if_stmt: &pi_ast::If) {
		let cond = expr_to_rval(&*if_stmt.condition);
		let then = self.append_new_block();
		let else_ = self.new_block();
		let merge = match if_stmt.else_.is_some() {
			true => self.new_block(),
			false => else_.clone(),
		};

		self.build_brcond(cond, then, else_.id.clone());

		self.cur_block_id = then;
		for stmt in &if_stmt.then {
			self.lower_stmt(stmt);
		}

		if if_stmt.else_.is_some() {
			self.build_br(merge.id);
		} else {
			self.build_br(else_.id);
		}

		self.cur_block_id = else_.id;
		if let Some(if_else_block) = &if_stmt.else_ {
			// append else
			self.append_existing_block(else_);
			for stmt in if_else_block {
				self.lower_stmt(stmt);
			}
			self.build_br(merge.id);
			// append merge
			self.cur_block_id = merge.id;
			self.append_existing_block(merge);
		} else {
			// append else
			self.cur_block_id = else_.id;
			self.append_existing_block(else_);
		}
	}

	fn lower_var(&mut self, var: &pi_ast::VarDecl) {
		let ty = expr_ty_to_mir_ty(&var.type_);

		let mut single_val_loaded = None;
		if var.values.len() == 1 && var.names.len() > 1 {
			let alloca = self.build_alloca(ty.clone());
			self.build_store(expr_to_rval(&var.values[0]), alloca);
			single_val_loaded = Some(self.build_load(alloca));
		}
		for (i, _) in var.names.iter().enumerate() {
			let alloca = self.build_alloca(ty.clone());
			if let Some(single_val_loaded) = &single_val_loaded {
				self.build_store(RValue::Local(*single_val_loaded), alloca);
			} else {
				self.build_store(expr_to_rval(&var.values[i]), alloca);
			}
		}
	}
}

#[derive(Debug, Clone)]
enum RValue {
	Local(MirID),
	BinOp(Binop),
	UnaryOp,
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

#[derive(Debug, Clone)]
struct Binop {
	lhs: Box<RValue>,
	op: OpKind,
	rhs: Box<RValue>,
}

#[derive(Debug, Clone)]
struct Block {
	id: MirID,
	instr_count: usize,
	instrs: Vec<Instruction>,
}

type MirID = usize;

#[derive(Debug, Clone)]
enum Instruction {
	Alloca(Alloca),
	Store(Store),
	Load(Load),
	Br(Br),
	BrCond(BrCond),
}

#[derive(Debug, Clone)]
struct Br {
	id: MirID,
	to: MirID,
}

#[derive(Debug, Clone)]
struct BrCond {
	id: MirID,
	cond: RValue,
	then: MirID,
	else_: MirID,
}

#[derive(Debug, Clone)]
struct Load {
	id: MirID,
	ptr: MirID,
}

#[derive(Debug, Clone)]
struct Store {
	val: RValue,
	ptr: MirID,
	id: MirID,
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

#[derive(Debug, Clone)]
struct Alloca {
	id: MirID,
	ty: Type,
}

fn expr_to_rval(e: &Expr) -> RValue {
	match e {
		Expr::IntLit(int) => {
			if int.signed {
				let mut val = *int.val as i64;
				if *int.negative {
					val *= -1;
				}
				match int.bits {
					64 => RValue::I64(val),
					32 => RValue::I32(val as i32),
					16 => RValue::I16(val as i16),
					8 => RValue::I8(val as i8),
					_ => RValue::I32(val as i32),
				}
			} else {
				match int.bits {
					64 => RValue::U64(*int.val as u64),
					32 => RValue::U32(*int.val as u32),
					16 => RValue::U16(*int.val as u16),
					8 => RValue::U8(*int.val as u8),
					_ => RValue::U32(*int.val as u32),
				}
			}
		}
		Expr::FloatLit(float) => {
			let mut val = *float.val as f64;
			if *float.negative {
				val *= -1.0;
			};
			match float.bits {
				64 => RValue::F64(val),
				32 => RValue::F32(val as f32),
				_ => RValue::F32(val as f32),
			}
		}
		Expr::BinOp(binop) => expr_binop_to_rval(binop),
		_ => panic!(),
	}
}

fn expr_binop_to_rval(binop: &pi_ast::BinOp) -> RValue {
	RValue::BinOp(Binop {
		lhs: Box::from(expr_to_rval(&*binop.x)),
		op: binop.op,
		rhs: Box::from(expr_to_rval(&*binop.y)),
	})
}

fn expr_ty_to_mir_ty(e: &Expr) -> Type {
	match e {
		Expr::PrimitiveType(prim) => match prim.kind {
			PrimitiveKind::I64 => Type::I64,
			PrimitiveKind::I32 => Type::I32,
			PrimitiveKind::I16 => Type::I16,
			PrimitiveKind::I8 => Type::I8,
			PrimitiveKind::U64 => Type::U64,
			PrimitiveKind::U32 => Type::U32,
			PrimitiveKind::U16 => Type::U16,
			PrimitiveKind::U8 => Type::U8,
			PrimitiveKind::F64 => Type::F64,
			PrimitiveKind::F32 => Type::F32,
			PrimitiveKind::Bool => Type::Bool,
			PrimitiveKind::Void => Type::Void,
		},
		_ => Type::I32,
	}
}

fn lower_fn(fn_decl: &pi_ast::FnDecl) -> FnDecl {
	let mut f = FnDecl {
		name: fn_decl.name.to_string(),
		blocks: vec![],
		block_count: 0,
		cur_block_id: 0,
	};
	f.cur_block_id = f.append_new_block();
	for stmt in &fn_decl.block {
		f.lower_stmt(stmt);
	}

	return f;
}

pub fn lower_ast(ast: &AST) {
	for fn_decl in &ast.functions {
		let f = lower_fn(&fn_decl);
		println!("{:#?}", f);
	}
}
