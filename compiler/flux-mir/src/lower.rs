// use std::{
// 	cell::{Ref, RefCell, RefMut},
// 	rc::Rc,
// };

// use crate::mir::{builder::Builder, *};
// use flux_hir as hir;
// use flux_syntax::ast::Spanned;
// use hir::ExprIdx;
// use la_arena::Arena;
// use smol_str::SmolStr;

// struct MirLoweringCtx<'a> {
// 	module: MirModule,
// 	builder: Builder,
// 	exprs: &'a Arena<Spanned<hir::Expr>>,
// }

// impl<'a> MirLoweringCtx<'a> {
// 	pub fn new(exprs: &'a Arena<Spanned<hir::Expr>>) -> Self {
// 		Self {
// 			exprs,
// 			module: MirModule::default(),
// 			builder: Builder::default(),
// 		}
// 	}

// 	fn cur_block(&self) -> Ref<Block> {
// 		self.builder.cur_block.as_ref().unwrap().borrow()
// 	}

// 	fn cur_block_mut(&self) -> RefMut<Block> {
// 		self.builder.cur_block.as_ref().unwrap().borrow_mut()
// 	}

// 	pub fn lower_function(&mut self, hir_fn: &hir::FnDecl) {
// 		let params: Vec<_> = hir_fn
// 			.params
// 			.node
// 			.iter()
// 			.map(|p| FnParam {
// 				ty: hir_ty_to_mir_ty(&p.ty.node),
// 				name: p.name.as_ref().unwrap().clone(),
// 			})
// 			.collect();
// 		let mir_function = self.module.new_function(
// 			hir_fn.name.as_ref().unwrap().node.clone(),
// 			params,
// 			hir_ty_to_mir_ty(&hir_fn.return_type),
// 		);

// 		self.builder.cur_fn = Some(mir_function.clone());
// 		self.lower_expr(hir_fn.block);
// 		println!("{}", mir_function.borrow());
// 	}

// 	fn lower_block(&mut self, hir_block: &hir::Block) -> RValue {
// 		let block = self.builder.append_new_block();
// 		self.builder.cur_block = Some(block.clone());

// 		for stmt in &hir_block.0 {
// 			self.lower_stmt(&stmt.as_ref().unwrap());
// 		}

// 		RValue::Unit
// 	}

// 	fn lower_stmt(&mut self, stmt: &hir::Stmt) {
// 		match stmt {
// 			hir::Stmt::VarDecl(var) => self.lower_var_stmt(var),
// 			hir::Stmt::Return(ret) => self.lower_ret_stmt(ret),
// 			hir::Stmt::Expr(expr) => {
// 				self.lower_expr(*expr);
// 				()
// 			}
// 		}
// 	}

// 	fn lower_ret_stmt(&mut self, ret: &hir::Return) {
// 		let v = match &self.exprs[ret.value].node {
// 			hir::Expr::Missing => None,
// 			_ => Some(self.lower_expr(ret.value)),
// 		};
// 		self.builder.new_ret(v);
// 	}

// 	// Don't ask me how this works
// 	fn lower_if_expr(&mut self, if_stmt: &hir::If) -> RValue {
// 		let cond = self.lower_expr(if_stmt.condition);
// 		let then = self.builder.append_new_block();
// 		let else_ = self.builder.new_block();
// 		let merge = if let hir::Expr::Missing = &self.exprs[if_stmt.else_].node {
// 			else_.clone()
// 		} else {
// 			self.builder.new_block()
// 		};

// 		self
// 			.builder
// 			.new_brnz(cond, then.borrow().id, else_.borrow().id);

// 		self.builder.cur_block = Some(then.clone());
// 		if let hir::Expr::Block(block) = &self.exprs[if_stmt.then].node {
// 			for stmt in &block.0 {
// 				self.lower_stmt(&stmt.as_ref().unwrap());
// 			}
// 		}

// 		if let hir::Expr::Missing = &self.exprs[if_stmt.else_].node {
// 			if then.borrow().terminator.is_none() {
// 				self.builder.new_br(else_.borrow().id);
// 			}
// 		} else {
// 			self.builder.new_br(merge.borrow().id);
// 		}

// 		self.builder.cur_block = Some(else_.clone());
// 		match &self.exprs[if_stmt.else_].node {
// 			hir::Expr::Missing => {
// 				self.builder.append_existing_block(else_);
// 				RValue::Unit
// 			}
// 			hir::Expr::If(if_) => {
// 				self.builder.append_existing_block(else_.clone());
// 				let v = self.lower_if_expr(if_);
// 				self.builder.new_br(merge.borrow().id);
// 				self.builder.cur_block = Some(merge.clone());
// 				self.builder.append_existing_block(merge);
// 				v
// 			}
// 			hir::Expr::Block(block) => {
// 				for stmt in &block.0 {
// 					self.lower_stmt(stmt.as_ref().unwrap());
// 				}
// 				self.builder.append_existing_block(else_.clone());
// 				self.builder.cur_block = Some(else_.clone());
// 				self.builder.new_br(merge.borrow().id);
// 				self.builder.cur_block = Some(merge.clone());
// 				self.builder.append_existing_block(merge);
// 				RValue::Unit
// 			}
// 			_ => unreachable!(),
// 		}
// 	}

// 	fn lower_var_stmt(&mut self, var: &hir::VarDecl) {
// 		let ty = hir_ty_to_mir_ty(&var.ty);
// 		let ptr = self.builder.new_alloca(ty.clone());
// 		self.cur_block_mut().locals.insert(var.name.clone(), ptr);
// 		self.cur_block_mut().local_types.insert(ptr, ty);
// 		let val = self.lower_expr(var.value);
// 		self.builder.new_store(ptr, val);
// 	}

// 	fn lower_expr(&mut self, expr: ExprIdx) -> RValue {
// 		match &self.exprs[expr].node {
// 			hir::Expr::Int(int) => self.lower_int_expr(int),
// 			hir::Expr::Float(float) => self.lower_float_expr(float),
// 			hir::Expr::Binary(binary) => self.lower_binary_expr(binary),
// 			hir::Expr::Path(path) => self.lower_path_expr(path),
// 			hir::Expr::Block(block) => self.lower_block(block),
// 			hir::Expr::If(if_) => self.lower_if_expr(if_),
// 			hir::Expr::Missing => RValue::Unit,
// 			_ => todo!("unimplemented expr: {:#?}", self.exprs[expr].node),
// 		}
// 	}

// 	fn lower_path_expr(&self, ident: &hir::Path) -> RValue {
// 		let name: SmolStr = SmolStr::from(
// 			ident
// 				.iter()
// 				.map(|s| s.node.clone())
// 				.collect::<Vec<_>>()
// 				.join("::"),
// 		);
// 		RValue::Local(self.cur_block().locals[&name])
// 	}

// 	fn lower_binary_expr(&mut self, binary_expr: &hir::Binary) -> RValue {
// 		let lhs = self.lower_expr(binary_expr.lhs);
// 		let rhs = self.lower_expr(binary_expr.rhs);
// 		let ty = self.type_of_rval(lhs.clone());
// 		let instr = match binary_expr.op {
// 			hir::BinaryOp::CmpEq => match &ty {
// 				Type::Int(_) => self.builder.new_icmp(ICmpKind::Eq, lhs, rhs),
// 				_ => todo!(),
// 			},
// 			_ => todo!(),
// 		};
// 		RValue::Local(instr)
// 	}

// 	fn lower_int_expr(&self, int_expr: &hir::Int) -> RValue {
// 		let size = match int_expr.ty {
// 			hir::Type::UInt(n) | hir::Type::SInt(n) => n,
// 			_ => panic!("hir int expression did not have int type"),
// 		};
// 		RValue::Int(Int {
// 			n: int_expr.n,
// 			size,
// 		})
// 	}

// 	fn lower_float_expr(&self, float_expr: &hir::Float) -> RValue {
// 		match &float_expr.ty {
// 			hir::Type::F32 => RValue::F32(float_expr.n as f32),
// 			hir::Type::F64 => RValue::F64(float_expr.n),
// 			_ => panic!("hir float expression did not have float type"),
// 		}
// 	}

// 	fn type_of_rval(&self, rval: RValue) -> Type {
// 		match rval {
// 			RValue::F32(_) => Type::F32,
// 			RValue::F64(_) => Type::F64,
// 			RValue::Int(int) => Type::Int(int.size),
// 			RValue::Local(id) => self
// 				.builder
// 				.cur_block
// 				.as_ref()
// 				.unwrap()
// 				.borrow()
// 				.local_types[&id]
// 				.clone(),
// 			RValue::Unit => Type::Void,
// 		}
// 	}
// }

// fn hir_ty_to_mir_ty(ty: &hir::Type) -> Type {
// 	match ty {
// 		hir::Type::UInt(n) => Type::Int(*n),
// 		hir::Type::Unit => Type::Void,
// 		hir::Type::Unknown => Type::Void,
// 		_ => todo!("unimplemented hir ty: {:#?}", ty),
// 	}
// }

// pub fn lower_module(hir_module: &hir::HirModule) -> MirModule {
// 	let mut ctx = MirLoweringCtx::new(&hir_module.exprs);

// 	for f in &hir_module.functions {
// 		ctx.lower_function(f);
// 	}

// 	return ctx.module;
// }
