// extern crate llvm_sys as llvm;
// use std::collections::HashMap;
// use std::ffi::{CStr, CString};
// use std::ops::Range;
// use std::ptr;

// use indexmap::IndexMap;
// use llvm::core::{
// 	LLVMAddFunction, LLVMAppendBasicBlock, LLVMAppendExistingBasicBlock, LLVMBuildAdd,
// 	LLVMBuildAlloca, LLVMBuildBr, LLVMBuildCall2, LLVMBuildCondBr, LLVMBuildICmp, LLVMBuildLoad2,
// 	LLVMBuildRet, LLVMBuildRetVoid, LLVMBuildStore, LLVMBuildStructGEP2, LLVMConstInt, LLVMConstReal,
// 	LLVMContextCreate, LLVMContextDispose, LLVMCreateBasicBlockInContext, LLVMCreateBuilderInContext,
// 	LLVMCreateFunctionPassManagerForModule, LLVMDisposeBuilder, LLVMDisposeModule,
// 	LLVMDoubleTypeInContext, LLVMDumpModule, LLVMDumpType, LLVMDumpValue, LLVMFloatTypeInContext,
// 	LLVMFunctionType, LLVMGetBasicBlockName, LLVMGetBasicBlockTerminator, LLVMGetElementType,
// 	LLVMGetLastBasicBlock, LLVMGetNamedFunction, LLVMGetOperand, LLVMGetParam, LLVMGetReturnType,
// 	LLVMGetStructName, LLVMGetTypeKind, LLVMInitializeFunctionPassManager, LLVMInt16TypeInContext,
// 	LLVMInt32TypeInContext, LLVMInt64TypeInContext, LLVMInt8TypeInContext, LLVMIntType,
// 	LLVMModuleCreateWithNameInContext, LLVMPointerType, LLVMPositionBuilderAtEnd,
// 	LLVMPrintModuleToFile, LLVMRunFunctionPassManager, LLVMStructCreateNamed, LLVMStructSetBody,
// 	LLVMTypeOf, LLVMVoidTypeInContext,
// };
// use llvm::prelude::{
// 	LLVMBasicBlockRef, LLVMBuilderRef, LLVMContextRef, LLVMModuleRef, LLVMPassManagerRef,
// 	LLVMTypeRef, LLVMValueRef,
// };
// use llvm::transforms::instcombine::LLVMAddInstructionCombiningPass;
// use llvm::transforms::scalar::{
// 	LLVMAddCFGSimplificationPass, LLVMAddDCEPass, LLVMAddEarlyCSEPass, LLVMAddGVNPass,
// 	LLVMAddLICMPass, LLVMAddLoopDeletionPass, LLVMAddLoopIdiomPass, LLVMAddLoopRerollPass,
// 	LLVMAddLoopRotatePass, LLVMAddLowerAtomicPass, LLVMAddNewGVNPass, LLVMAddReassociatePass,
// 	LLVMAddSCCPPass,
// };
// use llvm::transforms::util::LLVMAddPromoteMemoryToRegisterPass;
// use llvm::transforms::vectorize::{LLVMAddLoopVectorizePass, LLVMAddSLPVectorizePass};
// use llvm::LLVMTypeKind;
// use pi_ast::{
// 	ApplyBlock, BinOp, BlockStmt, CallExpr, Expr, FloatLit, FnDecl, FnParam, Ident, If, IntLit,
// 	InterfaceType, OpKind, PrimitiveKind, PtrType, Return, Spanned, Stmt, StructExpr, TypeDecl,
// 	Unary, VarDecl, AST,
// };
// use pi_cfg::{CompilationSettings, OptimizationLevel};
// use pi_error::filesystem::FileId;
// use pi_error::{PIError, PIErrorCode};
// use smol_str::SmolStr;
// use uuid::Uuid;

// type PITypeResult = (LLVMTypeRef, Option<PIError>);
// type PIValueResult = (LLVMValueRef, Option<PIError>);

// mod symbol_table;
// use symbol_table::SymbolTable;
// mod stmt;
// mod temp_data;
// use temp_data::CodegenTempData;
// mod type_expr;
// mod val_expr;
// mod val_mapping;
// use val_mapping::CodegenValuesMap;

// #[derive(Debug)]
// pub struct Codegen<'a> {
// 	settings: &'a CompilationSettings,
// 	tmp_data: CodegenTempData<'a>,

// 	ctx: LLVMContextRef,
// 	module: LLVMModuleRef,
// 	builder: LLVMBuilderRef,
// 	fpm: LLVMPassManagerRef,
// 	file_id: FileId,

// 	struct_implementations_map: HashMap<String, Vec<String>>, // Struct name -> Vec<Interface Names>
// 	types: HashMap<String, (TypeDecl, LLVMTypeRef)>,

// 	symbol_table: SymbolTable,
// 	values_map: CodegenValuesMap, // curently mapping functions. Eventually, global variables and also type decls
// }

// impl<'a> Codegen<'a> {
// 	pub fn new(name: String, file_id: &FileId, settings: &'a CompilationSettings) -> Self {
// 		unsafe {
// 			let ctx = LLVMContextCreate();
// 			let module = LLVMModuleCreateWithNameInContext(
// 				CString::from_vec_unchecked(name.into_bytes()).as_ptr(),
// 				ctx,
// 			);
// 			let builder = LLVMCreateBuilderInContext(ctx);
// 			let fpm = LLVMCreateFunctionPassManagerForModule(module);
// 			Self {
// 				tmp_data: CodegenTempData::new(),

// 				ctx,
// 				module,
// 				builder,
// 				fpm,

// 				file_id: file_id.clone(),
// 				struct_implementations_map: HashMap::new(),
// 				types: HashMap::new(),

// 				settings,
// 				symbol_table: SymbolTable::new(),
// 				values_map: CodegenValuesMap::new(file_id),
// 			}
// 		}
// 	}

// 	#[inline(always)]
// 	pub fn error(
// 		&self,
// 		msg: String,
// 		code: PIErrorCode,
// 		labels: Vec<(String, Range<usize>)>,
// 	) -> PIError {
// 		PIError::new(msg, code, labels, self.file_id)
// 	}

// 	pub fn dispose(&self) {
// 		unsafe {
// 			LLVMDisposeBuilder(self.builder);
// 			LLVMDisposeModule(self.module);
// 			LLVMContextDispose(self.ctx);
// 		}
// 	}

// 	pub fn write_to_file(&self, path: &String) {
// 		unsafe {
// 			let err_message = [str_to_cstring_mut("")].as_mut_ptr();
// 			LLVMPrintModuleToFile(self.module, str_to_cstring(&path), err_message);
// 		}
// 	}

// 	pub fn init_fpm(&self) {
// 		unsafe {
// 			if self.settings.optimization == OptimizationLevel::None {
// 				return;
// 			}
// 			LLVMAddInstructionCombiningPass(self.fpm);
// 			LLVMAddReassociatePass(self.fpm);
// 			LLVMAddGVNPass(self.fpm);
// 			LLVMAddCFGSimplificationPass(self.fpm);
// 			LLVMAddPromoteMemoryToRegisterPass(self.fpm);

// 			if self.settings.optimization == OptimizationLevel::Highest {
// 				LLVMAddDCEPass(self.fpm);
// 				LLVMAddLoopIdiomPass(self.fpm);
// 				LLVMAddLICMPass(self.fpm);
// 				LLVMAddLoopRerollPass(self.fpm);
// 				LLVMAddLoopRotatePass(self.fpm);
// 				LLVMAddSCCPPass(self.fpm);
// 				LLVMAddLowerAtomicPass(self.fpm);
// 				// LLVMAddIPSCCPPass(self.fpm);
// 				LLVMAddLoopDeletionPass(self.fpm);
// 				LLVMAddNewGVNPass(self.fpm);
// 				LLVMAddSLPVectorizePass(self.fpm);
// 				// LLVMAddConstantMergePass(self.fpm);
// 				LLVMAddLoopVectorizePass(self.fpm);
// 				LLVMAddEarlyCSEPass(self.fpm);
// 			}

// 			LLVMInitializeFunctionPassManager(self.fpm);
// 		}
// 	}

// 	pub fn generate_ir(&mut self, file_ast_map: &'a mut IndexMap<FileId, AST>) -> Option<PIError> {
// 		// for (_, ast) in file_ast_map {
// 		// 	self.tmp_data.mod_name = ast.name.clone();
// 		// 	for (struct_name, interfaces) in &ast.struct_implementations {
// 		// 		let mut interface_names = vec![];
// 		// 		for interface in interfaces {
// 		// 			interface_names.push(interface.name.val.to_string());
// 		// 		}
// 		// 		self
// 		// 			.struct_implementations_map
// 		// 			.insert(struct_name.val.to_string(), interface_names);
// 		// 	}

// 		// 	for ty in &ast.types {
// 		// 		if let Some(err) = self.type_decl(ty) {
// 		// 			return Some(err);
// 		// 		}
// 		// 	}

// 		// 	for apply_block in &mut ast.apply_blocks {
// 		// 		if let Some(err) = self.apply_block(apply_block) {
// 		// 			return Some(err);
// 		// 		}
// 		// 	}

// 		// 	for f in &ast.functions {
// 		// 		if let Some(err) = self.fn_decl(f) {
// 		// 			return Some(err);
// 		// 		}
// 		// 	}
// 		// }

// 		return None;
// 	}

// 	fn apply_block(&mut self, apply_block: &mut ApplyBlock) -> Option<PIError> {
// 		for method in &mut apply_block.methods {
// 			let mut start = String::from("");
// 			if let Some(name) = &apply_block.interface_name {
// 				start = name.to_string() + "_";
// 			}
// 			method.name = Spanned::new(
// 				SmolStr::from(start + apply_block.struct_name.as_str() + "_" + method.name.as_str()),
// 				0..0,
// 			);
// 			if let Some(err) = self.fn_decl(&method) {
// 				return Some(err);
// 			}
// 		}
// 		return None;
// 	}

// 	fn fn_proto(
// 		&mut self,
// 		ret_ty: &Spanned<Expr>,
// 		params: &Spanned<Vec<Spanned<FnParam>>>,
// 	) -> PITypeResult {
// 		unsafe {
// 			let mut param_types = vec![];
// 			for param in &**params {
// 				let (ty, err) = self.type_expr(&param.type_);
// 				if let Some(err) = err {
// 					return (ptr::null_mut(), Some(err));
// 				}
// 				param_types.push(ty);
// 			}

// 			match self.type_expr(ret_ty) {
// 				(_, Some(err)) => return (ptr::null_mut(), Some(err)),
// 				(ty, None) => (
// 					LLVMFunctionType(ty, param_types.as_mut_ptr(), param_types.len() as u32, 0),
// 					None,
// 				),
// 			}
// 		}
// 	}

// 	fn fn_decl(&mut self, f: &FnDecl) -> Option<PIError> {
// 		self.tmp_data.cur_fn_name = f.name.to_string();
// 		unsafe {
// 			let (fn_ty, err) = self.fn_proto(&f.ret_ty, &f.params);
// 			if let Some(err) = err {
// 				return Some(err);
// 			}

// 			let uuid = Uuid::new_v4();
// 			let function = LLVMAddFunction(
// 				self.module,
// 				str_to_cstring(uuid.to_string().as_str()),
// 				fn_ty,
// 			);

// 			let modified_function_name = match self.tmp_data.mod_name.as_str() {
// 				"main" => f.name.to_string(),
// 				x => String::from(x) + "_" + f.name.as_str(),
// 			};
// 			self
// 				.values_map
// 				.set_new_function(modified_function_name, uuid, function);
// 			let entry = LLVMAppendBasicBlock(function, str_to_cstring("entry"));
// 			LLVMPositionBuilderAtEnd(self.builder, entry);
// 			self.tmp_data.cur_bb = Some(entry);
// 			self.tmp_data.cur_fn = Some(function);

// 			for (i, param) in f.params.iter().enumerate() {
// 				let param_val = LLVMGetParam(function, i as u32);
// 				let (ty, err) = self.type_expr(&param.type_);
// 				if let Some(err) = err {
// 					return Some(err);
// 				}
// 				let ptr = LLVMBuildAlloca(self.builder, ty, str_to_cstring(param.name.as_str()));
// 				LLVMBuildStore(self.builder, param_val, ptr);

// 				self
// 					.symbol_table
// 					.set_value_in_cur_scope(param.name.to_string(), param_val);
// 			}

// 			if let Some(err) = self.block_stmt(&f.block) {
// 				return Some(err);
// 			}

// 			let x = LLVMGetBasicBlockTerminator(LLVMGetLastBasicBlock(function));
// 			if x.is_null() {
// 				LLVMBuildRetVoid(self.builder);
// 			}

// 			// LLVMVerifyFunction(
// 			// 	function,
// 			// 	llvm::analysis::LLVMVerifierFailureAction::LLVMPrintMessageAction,
// 			// );
// 			if self.settings.optimization != OptimizationLevel::None {
// 				LLVMRunFunctionPassManager(self.fpm, function);
// 			}

// 			self.symbol_table.clear();

// 			return None;
// 		}
// 	}

// 	fn var_decl(&mut self, var: &VarDecl) -> Option<PIError> {
// 		let (ty, err) = self.type_expr(&var.type_);
// 		if let Some(err) = err {
// 			return Some(err);
// 		}
// 		let single_val = var.values.len() == 1;
// 		unsafe {
// 			for i in 0..var.names.len() {
// 				let ptr = LLVMBuildAlloca(self.builder, ty, str_to_cstring(var.names[i].as_str()));
// 				if single_val {
// 					let (v, err) = self.expr(&var.values[0]);
// 					if let Some(err) = err {
// 						return Some(err);
// 					}
// 					LLVMBuildStore(self.builder, v, ptr);
// 				} else {
// 					let (v, err) = self.expr(&var.values[i]);
// 					if let Some(err) = err {
// 						return Some(err);
// 					}
// 					LLVMBuildStore(self.builder, v, ptr);
// 				}
// 				self
// 					.symbol_table
// 					.set_value_in_cur_scope(var.names[i].to_string(), ptr);
// 			}
// 		}
// 		return None;
// 	}

// 	fn type_decl(&mut self, ty: &'a TypeDecl) -> Option<PIError> {
// 		self.tmp_data.cur_type_decl = Some(ty);
// 		let (ty_val, err) = self.type_expr(&ty.type_);
// 		if let Some(err) = err {
// 			return Some(err);
// 		}
// 		self.types.insert(ty.name.to_string(), (ty.clone(), ty_val));
// 		return None;
// 	}
// }

// /*
//  * TODO: Figure out llvm-sys/C API equivalent of printAsOperand (https://stackoverflow.com/questions/26281823/llvm-how-to-get-the-label-of-basic-blocks)
//  * BasicBlocks must be created with names in order for this function to return a meaningful value until this is resolved.
// */
// fn get_basicblock_name(bb: LLVMBasicBlockRef) -> String {
// 	unsafe {
// 		let name = cstring_to_string(LLVMGetBasicBlockName(bb));
// 		if name.is_empty() {
// 			String::from("ERROR")
// 		} else {
// 			return name;
// 		}
// 	}
// }

// unsafe fn str_to_cstring(s: &str) -> *const i8 {
// 	let string = String::from(s);
// 	CString::from_vec_unchecked(string.into_bytes()).into_raw()
// }

// unsafe fn str_to_cstring_mut(s: &str) -> *mut i8 {
// 	let string = String::from(s);
// 	CString::from_vec_unchecked(string.into_bytes()).into_raw()
// }

// unsafe fn cstring_to_string(s: *const i8) -> String {
// 	CStr::from_ptr(s).to_str().unwrap().to_owned()
// }

// pub fn codegen_ast<'a>(
// 	file_ast_map: &'a mut IndexMap<FileId, AST>,
// 	settings: &'a CompilationSettings,
// ) -> (Codegen<'a>, Option<PIError>) {
// 	let entry_file_id = FileId(0);
// 	let mut codegen = Codegen::new(String::from("main"), &entry_file_id, settings);
// 	codegen.init_fpm();
// 	let result = codegen.generate_ir(file_ast_map);
// 	unsafe {
// 		LLVMDumpModule(codegen.module);
// 	}

// 	return (codegen, result);
// }
