extern crate llvm_sys as llvm;
use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::ops::Range;
use std::ptr;

use llvm::core::{
	LLVMAddFunction, LLVMAppendBasicBlock, LLVMAppendExistingBasicBlock, LLVMBuildAdd,
	LLVMBuildAlloca, LLVMBuildBr, LLVMBuildCondBr, LLVMBuildICmp, LLVMBuildLoad2, LLVMBuildRet,
	LLVMBuildRetVoid, LLVMBuildStore, LLVMBuildStructGEP2, LLVMConstInt, LLVMConstReal,
	LLVMContextCreate, LLVMCreateBasicBlockInContext, LLVMCreateBuilderInContext,
	LLVMDoubleTypeInContext, LLVMDumpModule, LLVMDumpType, LLVMFloatTypeInContext, LLVMFunctionType,
	LLVMGetBasicBlockName, LLVMGetBasicBlockTerminator, LLVMGetElementType, LLVMGetLastBasicBlock,
	LLVMGetOperand, LLVMGetParam, LLVMGetStructName, LLVMGetTypeKind, LLVMInt16TypeInContext,
	LLVMInt32TypeInContext, LLVMInt64TypeInContext, LLVMInt8TypeInContext, LLVMIntType,
	LLVMModuleCreateWithNameInContext, LLVMPointerType, LLVMPositionBuilderAtEnd,
	LLVMPrintModuleToFile, LLVMStructCreateNamed, LLVMStructSetBody, LLVMTypeOf,
	LLVMVoidTypeInContext,
};
use llvm::prelude::{
	LLVMBasicBlockRef, LLVMBuilderRef, LLVMContextRef, LLVMModuleRef, LLVMTypeRef, LLVMValueRef,
};
use llvm::LLVMTypeKind;
use pi_ast::{
	ApplyBlock, BinOp, BlockStmt, Expr, FloatLit, FnDecl, FnParam, Ident, If, IntLit, InterfaceType,
	OpKind, PrimitiveKind, PtrType, Return, Stmt, TypeDecl, VarDecl, AST,
};
use pi_error::filesystem::FileId;
use pi_error::{PIError, PIErrorCode};
use smol_str::SmolStr;

#[derive(Debug)]
struct SymTab {
	scope_conns: Vec<(String, String)>,
	scope_values: HashMap<String, HashMap<String, LLVMValueRef>>,
	cur_scope: String,
}

impl SymTab {
	pub fn new() -> Self {
		Self {
			scope_conns: vec![],
			scope_values: HashMap::from([(String::from("entry"), HashMap::new())]),
			cur_scope: String::from("entry"),
		}
	}

	pub fn get_value_in_scope(&self, scope: &String, name: &String) -> Option<LLVMValueRef> {
		let t = self.scope_values.get(scope);
		match t {
			Some(val_map) => match val_map.get(name) {
				Some(x) => Some(*x),
				_ => None,
			},
			_ => None,
		}
	}

	pub fn set_value_in_cur_scope(&mut self, name: String, val: LLVMValueRef) {
		if let Some(scope) = self.scope_values.get_mut(&self.cur_scope) {
			scope.insert(name, val);
		}
	}

	pub fn get_scopes_outside_scope(&self, scope: &String) -> Vec<String> {
		let mut scopes = vec![];
		for conn in &self.scope_conns {
			if conn.1 == *scope {
				scopes.push(conn.0.clone());
				scopes.append(&mut self.get_scopes_outside_scope(&conn.0));
			}
		}
		return scopes;
	}
}

#[derive(Debug)]
pub struct Codegen<'a> {
	ctx: LLVMContextRef,
	module: LLVMModuleRef,
	builder: LLVMBuilderRef,
	errs: &'a mut Vec<PIError>,
	file_id: FileId,
	should_continue: bool,
	symbol_tables: HashMap<String, SymTab>,
	// values: HashMap<String, LLVMValueRef>,
	types: HashMap<String, (Expr, LLVMTypeRef)>,
	cur_type_decl_name: String,
	cur_fn_name: String,
	cur_fn: Option<LLVMValueRef>,
	cur_bb: Option<LLVMBasicBlockRef>,
	merging_bb: Option<LLVMBasicBlockRef>,
}

impl<'a> Codegen<'a> {
	pub fn new(name: String, errs: &'a mut Vec<PIError>, file_id: &FileId) -> Self {
		unsafe {
			let ctx = LLVMContextCreate();
			let module = LLVMModuleCreateWithNameInContext(
				CString::from_vec_unchecked(name.into_bytes()).as_ptr(),
				ctx,
			);
			let builder = LLVMCreateBuilderInContext(ctx);
			Self {
				ctx,
				module,
				builder,
				errs,
				file_id: file_id.clone(),
				should_continue: true,
				symbol_tables: HashMap::new(),
				// values: HashMap::new(),
				types: HashMap::new(),
				cur_type_decl_name: String::new(),
				cur_fn_name: String::new(),
				cur_fn: None,
				cur_bb: None,
				merging_bb: None,
			}
		}
	}

	#[inline(always)]
	pub fn error(&mut self, msg: String, code: PIErrorCode, labels: Vec<(String, Range<usize>)>) {
		self
			.errs
			.push(PIError::new(msg, code, labels, self.file_id));
	}

	#[inline(always)]
	fn get_cur_sym_tab_mut(&mut self) -> &mut SymTab {
		return self
			.symbol_tables
			.get_mut(&self.cur_fn_name)
			.expect("function symbol table not initialized: internal error");
	}

	#[inline(always)]
	fn get_cur_sym_tab(&self) -> &SymTab {
		return self
			.symbol_tables
			.get(&self.cur_fn_name)
			.expect("function symbol table not initialized: internal error");
	}

	pub fn generate_ir(&mut self, ast: &mut AST) {
		// TODO: Don't order the typedecls, just forward decl?

		// let ordered_types = self.order_type_decls(&ast.types);
		for ty in &ast.types {
			if !self.should_continue {
				return;
			}
			self.type_decl(ty);
		}

		for apply_block in &mut ast.apply_blocks {
			self.apply_block(apply_block);
		}

		for f in &ast.functions {
			if !self.should_continue {
				return;
			}
			self.fn_decl(f);
		}
	}

	fn apply_block(&mut self, apply_block: &mut ApplyBlock) {
		for method in &mut apply_block.methods {
			let mut start = String::from("");
			if let Some(name) = &apply_block.interface_name {
				start = name.val.to_string() + "_";
			}
			method.name.val = SmolStr::from(
				start + apply_block.struct_name.val.as_str() + "_" + method.name.val.as_str(),
			);
			self.fn_decl(&method);
		}
	}

	// fn order_type_decls(&self, types: &Vec<TypeDecl>) -> Vec<TypeDecl> {
	// let mut new_vec = vec![];
	// for ty in types {
	// 	match &ty.type_ {
	// 		Expr::StructType(struct_ty) => {
	// 			for field in struct_ty {
	// 				if let Expr::Ident(x) = &field.type_ {
	// 					let idx = self.find_type_in_type_decls(types, &x.val.to_string());
	// 					if idx != -1 {
	// 						new_vec.push(types[idx as usize].clone());
	// 						new_vec.push(ty.clone());
	// 					}
	// 				}
	// 			}
	// 		}
	// 		_ => (),
	// 	}
	// }
	// return new_vec;
	// }

	// fn find_type_in_type_decls(&self, types: &Vec<TypeDecl>, name: &String) -> i32 {
	// 	let mut i = 0;
	// 	for ty in types {
	// 		if ty.name.val.to_string() == *name {
	// 			return i;
	// 		}
	// 		i += 1;
	// 	}
	// 	return -1;
	// }

	fn fn_proto(&mut self, ret_ty: &Expr, params: &Vec<FnParam>) -> LLVMTypeRef {
		unsafe {
			let mut param_types: Vec<LLVMTypeRef> = params
				.iter()
				.map(|param| self.type_expr(&param.type_))
				.collect();

			LLVMFunctionType(
				self.type_expr(ret_ty),
				param_types.as_mut_ptr(),
				param_types.len() as u32,
				0,
			)
		}
	}

	fn fn_decl(&mut self, f: &FnDecl) {
		self.cur_fn_name = f.name.val.to_string();
		self
			.symbol_tables
			.insert(f.name.val.to_string(), SymTab::new());
		unsafe {
			let fn_ty = self.fn_proto(&f.ret_ty, &f.params);

			let function = LLVMAddFunction(self.module, str_to_cstring(f.name.val.as_str()), fn_ty);
			let entry = LLVMAppendBasicBlock(function, str_to_cstring("entry"));
			LLVMPositionBuilderAtEnd(self.builder, entry);
			self.cur_bb = Some(entry);
			self.cur_fn = Some(function);

			for i in 0..f.params.len() {
				let param = LLVMGetParam(function, i as u32);
				let ptr = LLVMBuildAlloca(
					self.builder,
					self.type_expr(&f.params[i].type_),
					str_to_cstring(f.params[i].name.val.as_str()),
				);
				LLVMBuildStore(self.builder, param, ptr);
			}

			self.block_stmt(&f.block);

			let x = LLVMGetBasicBlockTerminator(LLVMGetLastBasicBlock(function));
			if x.is_null() {
				LLVMBuildRetVoid(self.builder);
			}
		}
	}

	fn block_stmt(&mut self, block: &BlockStmt) {
		for stmt in block {
			if !self.should_continue {
				return;
			}
			self.stmt(stmt);
		}
	}

	#[inline(always)]
	fn stmt(&mut self, stmt: &Stmt) {
		match stmt {
			Stmt::If(if_) => self.if_stmt(if_),
			Stmt::VarDecl(var) => self.var_decl(var),
			Stmt::Return(ret) => self.ret_stmt(ret),
			Stmt::ExprStmt(expr) => {
				self.expr(expr);
			}
			_ => (),
		}
	}

	fn if_stmt(&mut self, if_: &If) {
		let cond = self.expr(&if_.condition);

		unsafe {
			let f = self
				.cur_fn
				.expect("expected to be in function: internal compiler error");
			let then = LLVMAppendBasicBlock(f, str_to_cstring("then"));
			let else_ = LLVMCreateBasicBlockInContext(self.ctx, str_to_cstring("else"));
			let mut cont: LLVMBasicBlockRef = ptr::null_mut();
			if if_.else_.is_some() {
				cont = LLVMCreateBasicBlockInContext(self.ctx, str_to_cstring("continue"));
				self.merging_bb = Some(cont);
			} else {
				self.merging_bb = Some(else_);
			}

			LLVMBuildCondBr(self.builder, cond, then, else_);

			self.cur_bb = Some(then);
			LLVMPositionBuilderAtEnd(self.builder, then);

			let name = get_basicblock_name(then);
			let syms = self
				.symbol_tables
				.get_mut(&self.cur_fn_name)
				.expect("expected symbol table: internal compiler error");
			let initial_scope = syms.cur_scope.clone();
			syms.scope_conns.push((initial_scope.clone(), name.clone()));
			syms.scope_values.insert(name.clone(), HashMap::new());
			syms.cur_scope = name;

			self.block_stmt(&if_.then);
			if if_.else_.is_some() {
				LLVMBuildBr(self.builder, cont);
			} else {
				LLVMBuildBr(self.builder, else_);
			}

			self.cur_bb = Some(else_);
			LLVMPositionBuilderAtEnd(self.builder, else_);

			if let Some(else_block) = &if_.else_ {
				let name = get_basicblock_name(else_);
				let syms = self
					.symbol_tables
					.get_mut(&self.cur_fn_name)
					.expect("expected symbol table: internal compiler error");
				syms.scope_conns.push((initial_scope, name.clone()));
				syms.scope_values.insert(name.clone(), HashMap::new());
				syms.cur_scope = name;

				LLVMAppendExistingBasicBlock(f, else_);
				self.block_stmt(else_block);

				LLVMBuildBr(self.builder, cont);
				LLVMAppendExistingBasicBlock(f, cont);
				self.cur_bb = Some(cont);
				LLVMPositionBuilderAtEnd(self.builder, cont);
			} else {
				LLVMAppendExistingBasicBlock(f, else_);
				LLVMPositionBuilderAtEnd(self.builder, else_);
			}
		}
	}

	fn ret_stmt(&mut self, ret: &Return) {
		unsafe {
			match &ret.val {
				Some(x) => LLVMBuildRet(self.builder, self.expr(&x)),
				None => LLVMBuildRetVoid(self.builder),
			};
		}
	}

	fn var_decl(&mut self, var: &VarDecl) {
		let ty = self.type_expr(&var.type_);
		let single_val = match &var.values {
			Some(vals) => vals.len() == 1,
			_ => false,
		};
		unsafe {
			for i in 0..var.names.len() {
				let ptr = LLVMBuildAlloca(self.builder, ty, str_to_cstring(var.names[i].val.as_str()));
				if let Some(vals) = &var.values {
					if single_val {
						let v = self.expr(&vals[0]);
						LLVMBuildStore(self.builder, v, ptr);
					} else {
						LLVMBuildStore(self.builder, self.expr(&vals[i]), ptr);
					}
				}
				let syms = self.get_cur_sym_tab_mut();
				syms.set_value_in_cur_scope(var.names[i].val.to_string(), ptr);
			}
		}
	}

	fn expr(&mut self, expr: &Expr) -> LLVMValueRef {
		match expr {
			Expr::IntLit(int) => self.int(int),
			Expr::FloatLit(float) => self.float(float),
			Expr::BinOp(binop) => self.binop(binop),
			Expr::Ident(ident) => self.ident(ident),
			_ => panic!("unexpected expression"),
		}
	}

	fn ident(&mut self, ident: &Ident) -> LLVMValueRef {
		let syms = self.get_cur_sym_tab();
		if let Some(x) = self.find_val_in_scope(syms, &syms.cur_scope, &ident.val.to_string()) {
			unsafe {
				return LLVMBuildLoad2(
					self.builder,
					LLVMGetElementType(LLVMTypeOf(x)),
					x,
					str_to_cstring(""),
				);
			}
		} else {
			panic!("could not find var");
		}
	}

	fn find_val_in_scope(
		&self,
		syms: &SymTab,
		scope: &String,
		name: &String,
	) -> Option<LLVMValueRef> {
		let v = syms.get_value_in_scope(&scope, &name);
		if v.is_none() {
			let outer_scopes = syms.get_scopes_outside_scope(&scope);
			for s in outer_scopes {
				if let Some(x) = self.find_val_in_scope(syms, &s, name) {
					return Some(x);
				}
			}
			None
		} else {
			return v;
		}
	}

	fn binop(&mut self, binop: &BinOp) -> LLVMValueRef {
		unsafe {
			match binop.op {
				OpKind::Plus => LLVMBuildAdd(
					self.builder,
					self.expr(&*binop.x),
					self.expr(&*binop.y),
					str_to_cstring(""),
				),
				OpKind::CmpEQ => self.binop_cmp_eq(binop),
				OpKind::Eq => self.binop_eq(binop),
				OpKind::Period => self.binop_period(binop),
				_ => panic!("unexpected binop"),
			}
		}
	}

	fn binop_period(&mut self, binop: &BinOp) -> LLVMValueRef {
		let lhs = self.expr(&binop.x);
		let field_name = match &*binop.y {
			Expr::Ident(name) => name.val.to_string(),
			_ => panic!("rhs of struct access should be ident"),
		};

		unsafe {
			if LLVMGetTypeKind(LLVMTypeOf(lhs)) != LLVMTypeKind::LLVMStructTypeKind {
				panic!("expected struct on lhs of `.` binop expression");
			}
			let struct_ty_name = cstring_to_string(LLVMGetStructName(LLVMTypeOf(lhs)));

			let (ast_struct_ty_expr, _) = self
				.types
				.get(&struct_ty_name)
				.expect("type decl not in types map: internal compiler error");

			let ast_struct_ty = match ast_struct_ty_expr {
				Expr::StructType(struct_ty) => struct_ty,
				_ => panic!("expected lhs to be struct type"),
			};

			let mut i = 0;
			for (name, _) in ast_struct_ty {
				if name.val == field_name {
					break;
				}
				i += 1;
			}

			let gep = LLVMBuildStructGEP2(
				self.builder,
				LLVMTypeOf(lhs),
				LLVMGetOperand(lhs, 0),
				i,
				str_to_cstring(""),
			);
			LLVMBuildLoad2(
				self.builder,
				LLVMGetElementType(LLVMTypeOf(gep)),
				gep,
				str_to_cstring(""),
			)
		}
	}

	fn binop_eq(&mut self, binop: &BinOp) -> LLVMValueRef {
		let lhs = self.expr(&binop.x);
		let rhs = self.expr(&binop.y);

		unsafe { LLVMBuildStore(self.builder, rhs, LLVMGetOperand(lhs, 0)) }
	}

	fn binop_cmp_eq(&mut self, binop: &BinOp) -> LLVMValueRef {
		let lhs = self.expr(&binop.x);
		let rhs = self.expr(&binop.y);

		unsafe {
			let lhs_ty = LLVMTypeOf(lhs);
			match LLVMGetTypeKind(lhs_ty) {
				LLVMTypeKind::LLVMIntegerTypeKind => LLVMBuildICmp(
					self.builder,
					llvm::LLVMIntPredicate::LLVMIntEQ,
					lhs,
					rhs,
					str_to_cstring(""),
				),
				_ => panic!("cant compare those two things"),
			}
		}
	}

	fn float(&self, float: &FloatLit) -> LLVMValueRef {
		unsafe {
			let ty = match float.bits {
				64 => LLVMDoubleTypeInContext(self.ctx),
				32 => LLVMFloatTypeInContext(self.ctx),
				_ => LLVMFloatTypeInContext(self.ctx),
			};
			let val = match float.signed {
				true => float.val * -1.0,
				_ => float.val,
			};
			LLVMConstReal(ty, val)
		}
	}

	#[inline(always)]
	fn int(&self, int: &IntLit) -> LLVMValueRef {
		unsafe { LLVMConstInt(LLVMIntType(int.bits as u32), int.val, int.signed as i32) }
	}

	fn type_decl(&mut self, ty: &TypeDecl) {
		self.cur_type_decl_name = ty.name.val.to_string();
		let ty_val = self.type_expr(&ty.type_);
		self
			.types
			.insert(ty.name.val.to_string(), (ty.type_.clone(), ty_val));
	}

	fn type_expr(&mut self, e: &Expr) -> LLVMTypeRef {
		unsafe {
			match e {
				Expr::StructType(struct_ty) => self.struct_type(struct_ty),
				Expr::InterfaceType(interface_ty) => self.interface_type(interface_ty),
				Expr::PtrType(ptr) => self.ptr_type(ptr),
				Expr::PrimitiveType(prim) => match prim.kind {
					PrimitiveKind::I64 | PrimitiveKind::U64 => LLVMInt64TypeInContext(self.ctx),
					PrimitiveKind::I32 | PrimitiveKind::U32 => LLVMInt32TypeInContext(self.ctx),
					PrimitiveKind::I16 | PrimitiveKind::U16 => LLVMInt16TypeInContext(self.ctx),
					PrimitiveKind::I8 | PrimitiveKind::U8 => LLVMInt8TypeInContext(self.ctx),
					PrimitiveKind::F64 => LLVMDoubleTypeInContext(self.ctx),
					PrimitiveKind::F32 => LLVMFloatTypeInContext(self.ctx),
					PrimitiveKind::Void => LLVMVoidTypeInContext(self.ctx),
					_ => LLVMInt32TypeInContext(self.ctx),
				},
				Expr::Ident(ident) => self.ident_type(ident),
				_ => {
					panic!("unexpected expression: could not codegen {:?}", e);
				}
			}
		}
	}

	fn interface_type(&mut self, interface_ty: &InterfaceType) -> LLVMTypeRef {
		unsafe {
			let llvm_struct_ty =
				LLVMStructCreateNamed(self.ctx, str_to_cstring(self.cur_type_decl_name.as_str()));
			self.types.insert(
				self.cur_type_decl_name.clone(),
				(Expr::InterfaceType(interface_ty.clone()), llvm_struct_ty),
			);

			let mut method_types: Vec<LLVMTypeRef> = interface_ty
				.iter()
				.map(|(_, method)| LLVMPointerType(self.fn_proto(&method.ret_ty, &method.params), 0))
				.collect();

			LLVMStructSetBody(
				llvm_struct_ty,
				method_types.as_mut_ptr(),
				interface_ty.len() as u32,
				0,
			);

			LLVMDumpType(llvm_struct_ty);

			return llvm_struct_ty;
		}
	}

	fn struct_type(&mut self, struct_ty: &pi_ast::StructType) -> LLVMTypeRef {
		unsafe {
			let mut field_types: Vec<LLVMTypeRef> = struct_ty
				.iter()
				.map(|(_, field)| self.type_expr(&field.type_))
				.collect();

			let llvm_struct_ty =
				LLVMStructCreateNamed(self.ctx, str_to_cstring(self.cur_type_decl_name.as_str()));
			LLVMStructSetBody(
				llvm_struct_ty,
				field_types.as_mut_ptr(),
				struct_ty.len() as u32,
				0,
			);
			return llvm_struct_ty;
		}
	}

	#[inline(always)]
	fn ptr_type(&mut self, ptr: &PtrType) -> LLVMTypeRef {
		unsafe { LLVMPointerType(self.type_expr(&*ptr), 0) }
	}

	fn ident_type(&mut self, ident: &Ident) -> LLVMTypeRef {
		match self.types.get(&ident.val.to_string()) {
			Some((_, ty)) => *ty,
			None => {
				self.error(
					format!("could not find type `{}`", ident.val.to_string()),
					PIErrorCode::CodegenUnknownIdentType,
					vec![(
						format!("unkown type `{}` referenced", ident.val.to_string()),
						ident.span.clone(),
					)],
				);
				ptr::null_mut()
			}
		}
	}
}

/*
 * TODO: Figure out llvm-sys/C API equivalent of printAsOperand (https://stackoverflow.com/questions/26281823/llvm-how-to-get-the-label-of-basic-blocks)
 * BasicBlocks must be created with names in order for this function to return a meaningful value until this is resolved.
*/
fn get_basicblock_name(bb: LLVMBasicBlockRef) -> String {
	unsafe {
		let name = cstring_to_string(LLVMGetBasicBlockName(bb));
		if name.is_empty() {
			String::from("ERROR")
		} else {
			return name;
		}
	}
}

unsafe fn str_to_cstring(s: &str) -> *const i8 {
	let string = String::from(s);
	CString::from_vec_unchecked(string.into_bytes()).into_raw()
}

unsafe fn str_to_cstring_mut(s: &str) -> *mut i8 {
	let string = String::from(s);
	CString::from_vec_unchecked(string.into_bytes()).into_raw()
}

unsafe fn cstring_to_string(s: *const i8) -> String {
	CStr::from_ptr(s).to_str().unwrap().to_owned()
}

pub fn codegen_ast(ast: &mut AST, file_id: &FileId) -> Vec<PIError> {
	let mut errs = vec![];
	let mut codegen = Codegen::new(ast.name.clone(), &mut errs, file_id);
	codegen.generate_ir(ast);
	unsafe {
		LLVMDumpModule(codegen.module);
		let err_message = [str_to_cstring_mut("")].as_mut_ptr();
		LLVMPrintModuleToFile(codegen.module, str_to_cstring("module.ll"), err_message);

		// let out_message = [str_to_cstring_mut("")].as_mut_ptr();
		// LLVMVerifyModule(
		// 	codegen.module,
		// 	llvm::analysis::LLVMVerifierFailureAction::LLVMPrintMessageAction,
		// 	out_message,
		// );
	}
	return errs;
}
