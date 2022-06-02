use std::{collections::HashMap, fs};

use flux_ast::{
	CallExpr, EnumExpr, Expr, Ident, OpKind, PrimitiveType, Spanned, Stmt, TypeDecl, AST,
};

mod cfg;
pub mod mir;
use mir::*;

pub struct MIRModule {
	pub name: String,

	pub functions: HashMap<String, FnDecl>,
	pub struct_tys: HashMap<String, StructType>,
	enum_tags: HashMap<String, Vec<String>>,

	cur_fn: String,
}

impl MIRModule {
	pub fn new(name: String) -> Self {
		Self {
			name,

			functions: HashMap::new(),
			struct_tys: HashMap::new(),
			enum_tags: HashMap::new(),

			cur_fn: String::new(),
		}
	}
}

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
}

impl MIRModule {
	pub fn lower_type_decls(&mut self, types: &Vec<Spanned<TypeDecl>>) {
		for ty_decl in types {
			match &*ty_decl.type_ {
				Expr::EnumType(enum_) => self.lower_enum_type(enum_, ty_decl.name.to_string()),
				_ => (),
			};
		}
	}

	fn lower_enum_type(&mut self, enum_ty: &flux_ast::EnumType, name: String) {
		let mut ty_vec = StructType::new();
		ty_vec.push(Type::U8);
		let mut max_ty_size = 1;
		for (_, expr) in enum_ty {
			let ty = self.expr_ty_to_mir_ty(&expr);
			let size = self.sizeof_type(&ty);
			if size > max_ty_size {
				max_ty_size = size;
			}
		}

		let min_bytes_for_enum = if max_ty_size % 8 == 0 {
			max_ty_size / 8
		} else {
			max_ty_size / 8 + 1
		};

		ty_vec.push(Type::Vector(VectorTy::new(
			min_bytes_for_enum,
			Box::new(Type::U8),
		)));
		self.struct_tys.insert(name.clone(), ty_vec);
		for (tag, ty) in enum_ty {
			let mut ty_vec = StructType::new();
			ty_vec.push(Type::U8); // for tag
			ty_vec.push(self.expr_ty_to_mir_ty(ty)); // for assigned type
			self
				.struct_tys
				.insert(name.clone() + "." + tag.as_str(), ty_vec);
		}

		let tags: Vec<String> = enum_ty.keys().map(|tag| tag.to_string()).collect();
		self.enum_tags.insert(name, tags);
	}

	pub fn lower_function_decls(&mut self, functions: &Vec<Spanned<flux_ast::FnDecl>>) {
		for fn_decl in functions {
			self.lower_fn(&fn_decl);
			let cfg = cfg::print_fn(self.functions.get(&fn_decl.name.to_string()).unwrap());

			let _ = fs::write("examples/crate-1/cfg.dot", cfg);
		}
	}

	fn get_cur_fn(&mut self) -> &mut FnDecl {
		self.functions.get_mut(&self.cur_fn).unwrap()
	}

	fn lower_fn(&mut self, fn_decl: &flux_ast::FnDecl) {
		let ret_ty = self.expr_ty_to_mir_ty(&fn_decl.ret_ty);
		let mut params = vec![];
		for p in &*fn_decl.params {
			let ty = self.expr_ty_to_mir_ty(&p.type_);
			params.push(FnParam::new(*p.mut_, p.name.to_string(), ty));
		}
		let f = FnDecl::new(fn_decl.name.to_string(), ret_ty, params);
		self.functions.insert(fn_decl.name.to_string(), f);
		self.cur_fn = fn_decl.name.to_string();
		self.get_cur_fn().cur_block_id = self.get_cur_fn().append_new_block();
		for stmt in &fn_decl.block {
			self.lower_stmt(stmt);
		}
	}

	fn lower_stmt(&mut self, stmt: &flux_ast::Stmt) {
		match stmt {
			Stmt::VarDecl(var) => self.lower_var(var),
			Stmt::If(if_stmt) => self.lower_if(if_stmt),
			Stmt::Return(ret) => self.lower_ret(ret),
			_ => (),
		}
	}

	fn lower_ret(&mut self, ret: &flux_ast::Return) {
		if let Some(val) = &ret.val {
			let v = self.expr_to_rval(val);
			self.build_ret(Some(v));
		} else {
			self.build_ret(None);
		}
	}

	fn lower_if(&mut self, if_stmt: &flux_ast::If) {
		let cond = self.expr_to_rval(&*if_stmt.condition);
		let then = self.get_cur_fn().append_new_block();
		let else_ = self.get_cur_fn().new_block();
		let merge = match if_stmt.else_.is_some() {
			true => self.get_cur_fn().new_block(),
			false => else_.clone(),
		};

		self.build_brcond(cond, then, else_.id.clone());

		self.get_cur_fn().cur_block_id = then;
		let mut then_block_has_terminator = false;
		for stmt in &if_stmt.then {
			self.lower_stmt(stmt);
			if let Stmt::Return(_) = **stmt {
				then_block_has_terminator = true;
			}
		}

		if if_stmt.else_.is_some() {
			self.build_br(merge.id);
		} else if !then_block_has_terminator {
			self.build_br(else_.id);
		}

		self.get_cur_fn().cur_block_id = else_.id;
		if let Some(if_else_block) = &if_stmt.else_ {
			self.get_cur_fn().append_existing_block(else_);
			for stmt in if_else_block {
				self.lower_stmt(stmt);
			}
			self.build_br(merge.id);
			self.get_cur_fn().cur_block_id = merge.id;
			self.get_cur_fn().append_existing_block(merge);
		} else {
			self.get_cur_fn().cur_block_id = else_.id;
			self.get_cur_fn().append_existing_block(else_);
		}
	}

	fn lower_var(&mut self, var: &flux_ast::VarDecl) {
		let ty = self.expr_ty_to_mir_ty(&var.type_);

		let mut single_val_loaded = None;
		if var.values.len() == 1 && var.names.len() > 1 {
			let alloca = self.build_alloca(ty.clone());
			let v = self.expr_to_rval(&var.values[0]);
			self.build_store(ty.clone(), v, alloca);
			single_val_loaded = Some(self.build_load(ty.clone(), alloca));
		}
		for (i, _) in var.names.iter().enumerate() {
			let alloca = self.build_alloca(ty.clone());
			self
				.get_cur_fn()
				.locals
				.insert(var.names[i].to_string(), alloca);
			if let Some(single_val_loaded) = &single_val_loaded {
				self.build_store(ty.clone(), RValue::Local(*single_val_loaded), alloca);
			} else {
				let v = self.expr_to_rval(&var.values[i]);
				self.build_store(ty.clone(), v, alloca);
			}
		}
	}

	fn sizeof_type(&self, ty: &Type) -> u32 {
		match ty {
			Type::I64 | Type::U64 | Type::F64 => 64,
			Type::I32 | Type::U32 | Type::F32 => 32,
			Type::I16 | Type::U16 => 16,
			Type::I8 | Type::U8 => 8,
			Type::Bool => 1,
			Type::StructTy(struct_ty) => {
				let mut size = 0;
				for ty in struct_ty {
					size += self.sizeof_type(ty);
				}
				size
			}
			Type::Vector(vec_ty) => self.sizeof_type(&vec_ty.ty) * vec_ty.count as u32,
			Type::Ident(ident) => {
				let struct_ty = self.struct_tys.get(&ident.to_string()).unwrap();
				let mut size = 0;
				for ty in struct_ty {
					size += self.sizeof_type(ty);
				}
				size
			}
			Type::Ptr(_) => 64, // hmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmm ummmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmm aaaaaahhhh???
			Type::Void => panic!(),
		}
	}

	fn type_of_rval(&mut self, rval: &RValue) -> Type {
		match rval {
			RValue::I64(_) => Type::I64,
			RValue::I32(_) => Type::I32,
			RValue::I16(_) => Type::I16,
			RValue::I8(_) => Type::I8,
			RValue::U64(_) => Type::U64,
			RValue::U32(_) => Type::U32,
			RValue::U16(_) => Type::U16,
			RValue::U8(_) => Type::U8,
			RValue::Local(loc) => (*self.get_cur_fn().local_types.get(loc).unwrap()).clone(),
			_ => panic!(),
		}
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
			Expr::Ident(ident) => {
				// let x = (*self.struct_tys.get(&ident.to_string()).as_ref().unwrap()).clone();
				// Type::StructTy(x)
				Type::Ident(ident.to_string())
			}
			_ => panic!(),
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
			Expr::CallExpr(call) => self.expr_call_to_rval(call),
			Expr::EnumExpr(enum_expr) => self.expr_enum_to_rval(enum_expr),
			_ => panic!(),
		}
	}

	fn expr_enum_to_rval(&mut self, enum_expr: &EnumExpr) -> RValue {
		let base_ty = Type::Ident(enum_expr.enum_name.to_string());
		let base_alloca = self.build_alloca(base_ty.clone());
		let idx_access = self.build_idx_access(base_ty.clone(), base_alloca, 0);
		let tags = self
			.enum_tags
			.get(&enum_expr.enum_name.to_string())
			.unwrap();
		let tag_idx = tags
			.iter()
			.position(|t| *t == enum_expr.tag_name.to_string())
			.unwrap();
		let tagged_enum_name = enum_expr.enum_name.to_string() + "." + enum_expr.tag_name.as_str();
		let tagged_enum = self.struct_tys.get(&tagged_enum_name).unwrap();
		let ty = tagged_enum[1].clone();
		self.build_store(Type::U8, RValue::U8(tag_idx as u8), idx_access);

		let tagged_enum_ptr = self.build_ptr_cast(
			base_alloca,
			Type::Ptr(Box::new(Type::Ident(tagged_enum_name.clone()))),
		);
		let idx_access = self.build_idx_access(Type::Ident(tagged_enum_name), tagged_enum_ptr, 1);
		let val = self.expr_to_rval(&*enum_expr.val);
		self.build_store(base_ty, val, idx_access);
		let loaded = self.build_load(
			Type::Ptr(Box::new(Type::Ident(enum_expr.enum_name.to_string()))),
			base_alloca,
		);
		return RValue::Local(loaded);
	}

	fn expr_call_to_rval(&mut self, call: &CallExpr) -> RValue {
		RValue::Local(0)
	}

	fn expr_ident_to_rval(&mut self, ident: &Ident) -> RValue {
		let id = *self.get_cur_fn().locals.get(&ident.to_string()).expect("");
		let ty = self.get_cur_fn().local_types.get(&id).expect("").clone();
		RValue::Local(self.build_load(Type::Ptr(Box::new(ty)), id))
	}

	fn expr_binop_to_rval(&mut self, binop: &flux_ast::BinOp) -> RValue {
		let x = self.expr_to_rval(&*binop.x);
		let y = self.expr_to_rval(&*binop.y);
		let ty = self.type_of_rval(&x);

		match binop.op {
			OpKind::Plus => RValue::Local(self.build_add(x, y)),
			OpKind::CmpEQ => RValue::Local(self.build_cmp_eq(ty, x, y)),
			_ => panic!(),
		}
		// RValue::BinOp(Binop::new(Box::from(x), binop.op, Box::from(y)))
	}

	fn build_alloca(&mut self, ty: Type) -> MirID {
		let id = self.get_cur_fn().instr_count;
		self.get_cur_fn().instr_count += 1;
		let alloca = Alloca::new(id, ty.clone());
		self.get_cur_fn().local_types.insert(id, ty.clone());
		self
			.get_cur_fn()
			.get_cur_block()
			.instrs
			.push(Instruction::Alloca(alloca));
		return id;
	}

	fn build_store(&mut self, ty: Type, val: RValue, ptr: MirID) -> MirID {
		let id = self.get_cur_fn().instr_count;
		self.get_cur_fn().instr_count += 1;
		let store = Store::new(id, ty, val, ptr);
		self
			.get_cur_fn()
			.get_cur_block()
			.instrs
			.push(Instruction::Store(store));
		return id;
	}

	fn build_load(&mut self, ty: Type, ptr: MirID) -> MirID {
		let id = self.get_cur_fn().instr_count;
		self.get_cur_fn().instr_count += 1;
		let load = Load::new(id, ty.clone(), ptr);
		let ty = match ty {
			Type::Ptr(ptr) => *ptr,
			_ => panic!(),
		};
		self.get_cur_fn().local_types.insert(id, ty.clone());
		self
			.get_cur_fn()
			.get_cur_block()
			.instrs
			.push(Instruction::Load(load));
		return id;
	}

	fn build_brcond(&mut self, cond: RValue, then: MirID, else_: MirID) -> MirID {
		let id = self.get_cur_fn().instr_count;
		self.get_cur_fn().instr_count += 1;
		let brcond = BrCond::new(id, cond, then, else_);
		self
			.get_cur_fn()
			.get_cur_block()
			.instrs
			.push(Instruction::BrCond(brcond));
		return id;
	}

	fn build_br(&mut self, to: MirID) -> MirID {
		let id = self.get_cur_fn().instr_count;
		self.get_cur_fn().instr_count += 1;
		let br = Br::new(id, to);
		self
			.get_cur_fn()
			.get_cur_block()
			.instrs
			.push(Instruction::Br(br));
		return id;
	}

	fn build_ret(&mut self, val: Option<RValue>) -> MirID {
		let id = self.get_cur_fn().instr_count;
		self.get_cur_fn().instr_count += 1;
		let ret = Ret::new(id, val);
		self
			.get_cur_fn()
			.get_cur_block()
			.instrs
			.push(Instruction::Ret(ret));
		return id;
	}

	fn build_add(&mut self, lhs: RValue, rhs: RValue) -> MirID {
		let id = self.get_cur_fn().instr_count;
		self.get_cur_fn().instr_count += 1;
		let add = Add::new(id, lhs, rhs);
		self
			.get_cur_fn()
			.get_cur_block()
			.instrs
			.push(Instruction::Add(add));
		return id;
	}

	fn build_cmp_eq(&mut self, ty: Type, lhs: RValue, rhs: RValue) -> MirID {
		let id = self.get_cur_fn().instr_count;
		self.get_cur_fn().instr_count += 1;
		let cmp_eq = CmpEq::new(id, ty, lhs, rhs);
		self
			.get_cur_fn()
			.get_cur_block()
			.instrs
			.push(Instruction::CmpEq(cmp_eq));
		return id;
	}

	fn build_call(&mut self, callee: RValue, args: Vec<RValue>) -> MirID {
		let id = self.get_cur_fn().instr_count;
		self.get_cur_fn().instr_count += 1;
		let call = Call::new(id, callee, args);
		self
			.get_cur_fn()
			.get_cur_block()
			.instrs
			.push(Instruction::Call(call));
		return id;
	}

	fn build_idx_access(&mut self, ty: Type, ptr: MirID, idx: u32) -> MirID {
		let id = self.get_cur_fn().instr_count;
		self.get_cur_fn().instr_count += 1;
		let idx_access = IndexAccess::new(id, ty, ptr, idx);
		self
			.get_cur_fn()
			.get_cur_block()
			.instrs
			.push(Instruction::IndexAccess(idx_access));
		return id;
	}

	fn build_ptr_cast(&mut self, ptr: MirID, to_ty: Type) -> MirID {
		let id = self.get_cur_fn().instr_count;
		self.get_cur_fn().instr_count += 1;
		let ptr_cast = PtrCast::new(id, ptr, to_ty);
		self
			.get_cur_fn()
			.get_cur_block()
			.instrs
			.push(Instruction::PtrCast(ptr_cast));
		return id;
	}
}

pub fn lower_ast(ast: &AST) -> MIRModule {
	let mut module = MIRModule::new(ast.name.clone());

	module.lower_type_decls(&ast.types);
	module.lower_function_decls(&ast.functions);

	return module;
}
