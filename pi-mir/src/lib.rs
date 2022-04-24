use std::fs;

use pi_ast::{Expr, Ident, PrimitiveType, Stmt, AST};

mod cfg;
mod mir;
use mir::*;

// struct MIRModule {
// 	functions: Vec<FnDecl>,
// }

impl FnDecl {
	fn new_block(&mut self) -> Block {
		let id = self.block_count;
		self.block_count += 1;
		Block { id, instrs: vec![] }
	}

	fn append_new_block(&mut self) -> MirID {
		let id = self.block_count;
		self.block_count += 1;
		self.blocks.push(Block { id, instrs: vec![] });
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
		let id = self.instr_count;
		self.instr_count += 1;
		let alloca = Alloca::new(id, ty);
		self.local_types.insert(id, ty);
		self
			.get_cur_block()
			.instrs
			.push(Instruction::Alloca(alloca));
		return id;
	}

	fn build_store(&mut self, ty: Type, val: RValue, ptr: MirID) -> MirID {
		let id = self.instr_count;
		self.instr_count += 1;
		let store = Store::new(id, ty, val, ptr);
		self.get_cur_block().instrs.push(Instruction::Store(store));
		return id;
	}

	fn build_load(&mut self, ty: Type, ptr: MirID) -> MirID {
		let id = self.instr_count;
		self.instr_count += 1;
		let load = Load::new(id, ty, ptr);
		self.local_types.insert(id, ty);
		self.get_cur_block().instrs.push(Instruction::Load(load));
		return id;
	}

	fn build_brcond(&mut self, cond: RValue, then: MirID, else_: MirID) -> MirID {
		let id = self.instr_count;
		self.instr_count += 1;
		let brcond = BrCond::new(id, cond, then, else_);
		self
			.get_cur_block()
			.instrs
			.push(Instruction::BrCond(brcond));
		return id;
	}

	fn build_br(&mut self, to: MirID) -> MirID {
		let id = self.instr_count;
		self.instr_count += 1;
		let br = Br::new(id, to);
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
		let cond = self.expr_to_rval(&*if_stmt.condition);
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
		let ty = self.expr_ty_to_mir_ty(&var.type_);

		let mut single_val_loaded = None;
		if var.values.len() == 1 && var.names.len() > 1 {
			let alloca = self.build_alloca(ty);
			let v = self.expr_to_rval(&var.values[0]);
			self.build_store(ty, v, alloca);
			single_val_loaded = Some(self.build_load(ty, alloca));
		}
		for (i, _) in var.names.iter().enumerate() {
			let alloca = self.build_alloca(ty);
			self.locals.insert(var.names[i].to_string(), alloca);
			if let Some(single_val_loaded) = &single_val_loaded {
				self.build_store(ty, RValue::Local(*single_val_loaded), alloca);
			} else {
				let v = self.expr_to_rval(&var.values[i]);
				self.build_store(ty, v, alloca);
			}
		}
	}

	fn expr_to_rval(&mut self, e: &Expr) -> RValue {
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
			Expr::BinOp(binop) => self.expr_binop_to_rval(binop),
			Expr::Ident(ident) => self.expr_ident_to_rval(ident),
			_ => panic!(),
		}
	}

	fn expr_ident_to_rval(&mut self, ident: &Ident) -> RValue {
		let id = *self.locals.get(&ident.to_string()).expect("");
		let ty = self.local_types.get(&id).expect("").clone();
		RValue::Local(self.build_load(ty, id))
	}

	fn expr_binop_to_rval(&mut self, binop: &pi_ast::BinOp) -> RValue {
		let x = self.expr_to_rval(&*binop.x);
		let y = self.expr_to_rval(&*binop.y);
		RValue::BinOp(Binop::new(Box::from(x), binop.op, Box::from(y)))
	}

	fn expr_ty_to_mir_ty(&self, e: &Expr) -> Type {
		match e {
			Expr::PrimitiveType(prim) => match prim {
				PrimitiveType::I64 => Type::I64,
				PrimitiveType::I32 => Type::I32,
				PrimitiveType::I16 => Type::I16,
				PrimitiveType::I8 => Type::I8,
				PrimitiveType::U64 => Type::U64,
				PrimitiveType::U32 => Type::U32,
				PrimitiveType::U16 => Type::U16,
				PrimitiveType::U8 => Type::U8,
				PrimitiveType::F64 => Type::F64,
				PrimitiveType::F32 => Type::F32,
				PrimitiveType::Bool => Type::Bool,
				PrimitiveType::Void => Type::Void,
			},
			_ => Type::I32,
		}
	}
}

fn lower_fn(fn_decl: &pi_ast::FnDecl) -> FnDecl {
	let mut f = FnDecl::new(fn_decl.name.to_string());
	f.cur_block_id = f.append_new_block();
	for stmt in &fn_decl.block {
		f.lower_stmt(stmt);
	}

	return f;
}

pub fn lower_ast(ast: &AST) {
	for fn_decl in &ast.functions {
		let f = lower_fn(&fn_decl);
		let cfg = cfg::print_fn(&f);

		let _ = fs::write("examples/crate-1/cfg.dot", cfg);
	}
}
