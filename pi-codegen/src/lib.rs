extern crate llvm_sys as llvm;
use std::borrow::BorrowMut;
use std::collections::HashMap;
use std::ffi::CString;
use std::hash::Hash;

use llvm::core::{
	LLVMAddFunction, LLVMAppendBasicBlock, LLVMBuildAdd, LLVMBuildAlloca, LLVMBuildLoad2,
	LLVMBuildRet, LLVMBuildRetVoid, LLVMBuildStore, LLVMConstInt, LLVMConstReal, LLVMContextCreate,
	LLVMCreateBuilderInContext, LLVMDoubleTypeInContext, LLVMDumpModule, LLVMDumpType, LLVMDumpValue,
	LLVMFloatTypeInContext, LLVMFunctionType, LLVMGetBasicBlockTerminator, LLVMGetElementType,
	LLVMGetLastBasicBlock, LLVMGetOperand, LLVMGetParam, LLVMInt16TypeInContext,
	LLVMInt32TypeInContext, LLVMInt64TypeInContext, LLVMInt8TypeInContext, LLVMIntType,
	LLVMModuleCreateWithNameInContext, LLVMPointerType, LLVMPositionBuilderAtEnd,
	LLVMPrintModuleToFile, LLVMPrintModuleToString, LLVMPrintTypeToString, LLVMPrintValueToString,
	LLVMStructCreateNamed, LLVMStructSetBody, LLVMStructTypeInContext, LLVMTypeOf,
};
use llvm::prelude::{LLVMBuilderRef, LLVMContextRef, LLVMModuleRef, LLVMTypeRef, LLVMValueRef};
use pi_ast::{
	BinOp, BlockStmt, Expr, FloatLit, FnDecl, Ident, IntLit, OpKind, PrimitiveKind, PtrType, Return,
	Stmt, TypeDecl, VarDecl, AST,
};

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

	pub fn is_entry_scope(&self) -> bool {
		self.cur_scope == "entry"
	}

	pub fn get_value_in_cur_scope(&self, name: String) -> Option<LLVMValueRef> {
		let t = self.scope_values.get(&self.cur_scope);
		match t {
			Some(val_map) => match val_map.get(&name) {
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
}

#[derive(Debug)]
pub struct Codegen {
	ctx: LLVMContextRef,
	module: LLVMModuleRef,
	builder: LLVMBuilderRef,
	symbol_tables: HashMap<String, SymTab>,
	values: HashMap<String, LLVMValueRef>,
	types: HashMap<String, LLVMTypeRef>,
	cur_struct_name: String,
	cur_fn_name: String,
}

impl Codegen {
	pub fn new(name: String) -> Self {
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
				symbol_tables: HashMap::new(),
				values: HashMap::new(),
				types: HashMap::new(),
				cur_struct_name: String::new(),
				cur_fn_name: String::new(),
			}
		}
	}

	fn get_cur_sym_tab(&mut self) -> &mut SymTab {
		return self
			.symbol_tables
			.get_mut(&self.cur_fn_name)
			.expect("function symbol table not initialized: internal error");
	}

	pub fn generate_ir(&mut self, ast: &AST) {
		for ty in &ast.types {
			self.type_decl(ty);
		}

		for f in &ast.functions {
			self.fn_decl(f);
		}
	}

	fn fn_decl(&mut self, f: &FnDecl) {
		self.cur_fn_name = f.name.to_string();
		self.symbol_tables.insert(f.name.to_string(), SymTab::new());
		unsafe {
			let mut param_types: Vec<LLVMTypeRef> = f
				.params
				.iter()
				.map(|param| self.pi_type_to_llvm_type(&param.type_))
				.collect();
			let fn_ty = LLVMFunctionType(
				self.pi_type_to_llvm_type(&f.ret_ty),
				param_types.as_mut_ptr(),
				f.params.len() as u32,
				0,
			);

			let function = LLVMAddFunction(self.module, str_to_cstring(f.name.as_str()), fn_ty);
			let entry = LLVMAppendBasicBlock(function, str_to_cstring("entry"));
			LLVMPositionBuilderAtEnd(self.builder, entry);

			for i in 0..f.params.len() {
				let param = LLVMGetParam(function, i as u32);
				let ptr = LLVMBuildAlloca(
					self.builder,
					self.pi_type_to_llvm_type(&f.params[i].type_),
					str_to_cstring(f.params[i].name.as_str()),
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
			self.stmt(stmt);
		}
	}

	fn stmt(&mut self, stmt: &Stmt) {
		match stmt {
			Stmt::VarDecl(var) => self.var_decl(var),
			Stmt::Return(ret) => self.ret_stmt(ret),
			_ => (),
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
		let ty = self.pi_type_to_llvm_type(&var.type_);
		let single_val = match &var.values {
			Some(vals) => vals.len() == 1,
			_ => false,
		};
		unsafe {
			for i in 0..var.names.len() {
				let ptr = LLVMBuildAlloca(self.builder, ty, str_to_cstring(var.names[i].as_str()));
				if let Some(vals) = &var.values {
					if single_val {
						LLVMBuildStore(self.builder, self.expr(&vals[0]), ptr);
					} else {
						LLVMBuildStore(self.builder, self.expr(&vals[i]), ptr);
					}
				}
				let syms = self.get_cur_sym_tab();
				syms.set_value_in_cur_scope(var.names[i].to_string(), ptr);
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
		if let Some(val) = syms.get_value_in_cur_scope(ident.to_string()) {
			unsafe {
				return LLVMBuildLoad2(
					self.builder,
					LLVMGetElementType(LLVMTypeOf(val)),
					val,
					str_to_cstring(""),
				);
			}
		} else {
			panic!("could not find var");
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
				_ => panic!("unexpected binop"),
			}
		}
	}

	#[inline]
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

	#[inline]
	fn int(&self, int: &IntLit) -> LLVMValueRef {
		unsafe { LLVMConstInt(LLVMIntType(int.bits as u32), int.val, int.signed as i32) }
	}

	fn type_decl(&mut self, ty: &TypeDecl) {
		unsafe {
			self.cur_struct_name = ty.name.to_string();
			let ty_val = self.type_expr(&ty.type_);
			self.types.insert(ty.name.to_string(), ty_val);
		}
	}

	unsafe fn type_expr(&self, e: &Expr) -> LLVMTypeRef {
		match e {
			Expr::StructType(struct_ty) => self.struct_type(struct_ty),
			Expr::PtrType(ptr) => self.ptr_type(ptr),
			_ => {
				panic!("unexpected expression: could not codegen {:?}", e);
			}
		}
	}

	fn ptr_type(&self, ptr: &PtrType) -> LLVMTypeRef {
		unsafe { LLVMPointerType(self.type_expr(&*ptr), 0) }
	}

	fn ident_type(&self, ident: &Ident) -> LLVMTypeRef {
		match self.types.get(&ident.to_string()) {
			Some(ty) => *ty,
			None => panic!("test"),
		}
	}

	unsafe fn struct_type(&self, struct_ty: &pi_ast::StructType) -> LLVMTypeRef {
		let mut field_types: Vec<LLVMTypeRef> = struct_ty
			.iter()
			.map(|field| self.pi_type_to_llvm_type(&field.type_))
			.collect();

		let llvm_struct_ty =
			LLVMStructCreateNamed(self.ctx, str_to_cstring(self.cur_struct_name.as_str()));
		LLVMStructSetBody(
			llvm_struct_ty,
			field_types.as_mut_ptr(),
			struct_ty.len() as u32,
			0,
		);
		return llvm_struct_ty;
	}

	fn pi_type_to_llvm_type(&self, ty: &Expr) -> LLVMTypeRef {
		unsafe {
			match ty {
				Expr::PrimitiveType(prim) => match prim.kind {
					PrimitiveKind::I64 | PrimitiveKind::U64 => LLVMInt64TypeInContext(self.ctx),
					PrimitiveKind::I32 | PrimitiveKind::U32 => LLVMInt32TypeInContext(self.ctx),
					PrimitiveKind::I16 | PrimitiveKind::U16 => LLVMInt16TypeInContext(self.ctx),
					PrimitiveKind::I8 | PrimitiveKind::U8 => LLVMInt8TypeInContext(self.ctx),
					PrimitiveKind::F64 => LLVMDoubleTypeInContext(self.ctx),
					PrimitiveKind::F32 => LLVMFloatTypeInContext(self.ctx),
					_ => LLVMInt32TypeInContext(self.ctx),
				},
				Expr::Ident(ident) => self.ident_type(ident),
				_ => LLVMInt32TypeInContext(self.ctx),
			}
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

pub fn codegen_ast(ast: &mut AST) {
	let mut codegen = Codegen::new(ast.name.clone());
	codegen.generate_ir(ast);
	unsafe {
		LLVMDumpModule(codegen.module);
		let err_message = [str_to_cstring_mut("error!")].as_mut_ptr();
		LLVMPrintModuleToFile(codegen.module, str_to_cstring("module.ll"), err_message);
	}
}
