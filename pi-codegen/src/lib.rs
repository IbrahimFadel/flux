use std::{
	collections::HashMap,
	ffi::{CStr, CString},
	ptr,
};

use llvm_sys::{
	core::{
		LLVMAddFunction, LLVMAppendBasicBlock, LLVMAppendBasicBlockInContext, LLVMBuildAlloca,
		LLVMBuildBitCast, LLVMBuildBr, LLVMBuildCondBr, LLVMBuildICmp, LLVMBuildLoad2, LLVMBuildRet,
		LLVMBuildRetVoid, LLVMBuildStore, LLVMBuildStructGEP2, LLVMConstInt, LLVMConstReal,
		LLVMContextCreate, LLVMCreateBuilderInContext, LLVMCreateFunctionPassManagerForModule,
		LLVMDoubleTypeInContext, LLVMDumpModule, LLVMDumpType, LLVMDumpValue, LLVMFloatTypeInContext,
		LLVMFunctionType, LLVMInitializeFunctionPassManager, LLVMInt16TypeInContext,
		LLVMInt32TypeInContext, LLVMInt64TypeInContext, LLVMInt8TypeInContext,
		LLVMModuleCreateWithNameInContext, LLVMPointerType, LLVMPositionBuilderAtEnd,
		LLVMRunFunctionPassManager, LLVMStructCreateNamed, LLVMStructSetBody, LLVMVectorType,
	},
	prelude::{
		LLVMBasicBlockRef, LLVMBuilderRef, LLVMContextRef, LLVMModuleRef, LLVMPassManagerRef,
		LLVMTypeRef, LLVMValueRef,
	},
	transforms::{
		instcombine::LLVMAddInstructionCombiningPass,
		scalar::{LLVMAddCFGSimplificationPass, LLVMAddGVNPass, LLVMAddReassociatePass},
		util::LLVMAddPromoteMemoryToRegisterPass,
	},
};
use pi_cfg::{CompilationSettings, OptimizationLevel};
use pi_mir::mir;
use pi_mir::MIRModule;

#[derive(Debug)]
pub struct Codegen<'a> {
	settings: &'a CompilationSettings,

	struct_types: HashMap<String, LLVMTypeRef>,
	locals: HashMap<mir::MirID, LLVMValueRef>,
	blocks: HashMap<mir::MirID, LLVMBasicBlockRef>,
	cur_function: Option<LLVMValueRef>,

	ctx: LLVMContextRef,
	module: LLVMModuleRef,
	builder: LLVMBuilderRef,
	fpm: LLVMPassManagerRef,
}

impl<'a> Codegen<'a> {
	pub fn new(name: String, settings: &'a CompilationSettings) -> Self {
		unsafe {
			let ctx = LLVMContextCreate();
			let module = LLVMModuleCreateWithNameInContext(
				CString::from_vec_unchecked(name.into_bytes()).as_ptr(),
				ctx,
			);
			let builder = LLVMCreateBuilderInContext(ctx);
			let fpm = LLVMCreateFunctionPassManagerForModule(module);
			Self {
				settings,

				struct_types: HashMap::new(),
				locals: HashMap::new(),
				blocks: HashMap::new(),
				cur_function: None,

				ctx,
				module,
				builder,
				fpm,
			}
		}
	}

	pub fn init_fpm(&self) {
		unsafe {
			if self.settings.optimization == OptimizationLevel::None {
				return;
			}
			LLVMAddInstructionCombiningPass(self.fpm);
			LLVMAddReassociatePass(self.fpm);
			LLVMAddGVNPass(self.fpm);
			LLVMAddCFGSimplificationPass(self.fpm);
			LLVMAddPromoteMemoryToRegisterPass(self.fpm);

			if self.settings.optimization == OptimizationLevel::Highest {
				// LLVMAddDCEPass(self.fpm);
				// LLVMAddLoopIdiomPass(self.fpm);
				// LLVMAddLICMPass(self.fpm);
				// LLVMAddLoopRerollPass(self.fpm);
				// LLVMAddLoopRotatePass(self.fpm);
				// LLVMAddSCCPPass(self.fpm);
				// LLVMAddLowerAtomicPass(self.fpm);
				// // LLVMAddIPSCCPPass(self.fpm);
				// LLVMAddLoopDeletionPass(self.fpm);
				// LLVMAddNewGVNPass(self.fpm);
				// LLVMAddSLPVectorizePass(self.fpm);
				// // LLVMAddConstantMergePass(self.fpm);
				// LLVMAddLoopVectorizePass(self.fpm);
				// LLVMAddEarlyCSEPass(self.fpm);
			}

			LLVMInitializeFunctionPassManager(self.fpm);
		}
	}

	pub fn struct_ty(&mut self, struct_ty: &mir::StructType, name: &String) {
		unsafe {
			let st = LLVMStructCreateNamed(self.ctx, str_to_cstring(name.as_str()));
			let mut field_tys = vec![];
			for ty in struct_ty {
				field_tys.push(self.mir_ty_to_llvm_ty(ty));
			}
			LLVMStructSetBody(st, field_tys.as_mut_ptr(), struct_ty.len() as u32, 0);
			self.struct_types.insert(name.clone(), st);
		}
	}

	pub fn fn_decl(&mut self, fn_decl: &mir::FnDecl) {
		let proto = self.fn_proto(&fn_decl.ret_ty, &fn_decl.params);

		unsafe {
			let function = LLVMAddFunction(self.module, str_to_cstring(fn_decl.name.as_str()), proto);
			self.cur_function = Some(function);
			for block in &fn_decl.blocks {
				let bb = LLVMAppendBasicBlock(function, str_to_cstring(""));
				self.blocks.insert(block.id, bb);
			}
			for block in &fn_decl.blocks {
				let bb = self.blocks.get(&block.id).unwrap();
				LLVMPositionBuilderAtEnd(self.builder, *bb);
				for instr in &block.instrs {
					self.instr(instr);
				}
			}

			if self.settings.optimization != OptimizationLevel::None {
				LLVMRunFunctionPassManager(self.fpm, function);
			}
		}
	}

	fn fn_proto(&self, ret_ty: &mir::Type, params: &Vec<mir::FnParam>) -> LLVMTypeRef {
		unsafe {
			let mut param_types = vec![];
			for p in params {
				param_types.push(self.mir_ty_to_llvm_ty(&p.ty));
			}
			let ret_ty = self.mir_ty_to_llvm_ty(ret_ty);
			LLVMFunctionType(
				ret_ty,
				param_types.as_mut_ptr(),
				param_types.len() as u32,
				0,
			)
		}
	}

	fn instr(&mut self, instr: &mir::Instruction) {
		use mir::Instruction;
		match instr {
			Instruction::Alloca(alloca) => self.alloca(alloca),
			Instruction::Load(load) => self.load(load),
			Instruction::Store(store) => self.store(store),
			Instruction::IndexAccess(idx_access) => self.idx_access(idx_access),
			Instruction::PtrCast(ptr_cast) => self.ptr_cast(ptr_cast),
			Instruction::CmpEq(cmp_eq) => self.cmp_eq(cmp_eq),
			Instruction::BrCond(br_cond) => self.br_cond(br_cond),
			Instruction::Br(br) => self.br(br),
			Instruction::Ret(ret) => self.ret(ret),
			_ => (),
		}
	}

	fn alloca(&mut self, alloca: &mir::Alloca) {
		let ty = self.mir_ty_to_llvm_ty(&alloca.ty);
		unsafe {
			let v = LLVMBuildAlloca(self.builder, ty, str_to_cstring(""));
			self.locals.insert(alloca.id, v);
		}
	}

	fn load(&mut self, load: &mir::Load) {
		let ty = self.mir_ty_to_llvm_ty(&load.ty);
		let ptr = self.locals.get(&load.ptr).unwrap();
		unsafe {
			let v = LLVMBuildLoad2(self.builder, ty, *ptr, str_to_cstring(""));
			self.locals.insert(load.id, v);
		}
	}

	fn store(&self, store: &mir::Store) {
		let val = self.mir_rval_to_llvm_val(&store.val);
		let ptr = self.locals.get(&store.ptr).unwrap();
		unsafe {
			LLVMBuildStore(self.builder, val, *ptr);
		}
	}

	fn idx_access(&mut self, idx_access: &mir::IndexAccess) {
		let ty = self.mir_ty_to_llvm_ty(&idx_access.ty);
		let ptr = self.locals.get(&idx_access.ptr).unwrap();
		unsafe {
			let v = LLVMBuildStructGEP2(self.builder, ty, *ptr, idx_access.idx, str_to_cstring(""));
			self.locals.insert(idx_access.id, v);
		}
	}

	fn ptr_cast(&mut self, ptr_cast: &mir::PtrCast) {
		let ptr = self.locals.get(&ptr_cast.ptr).unwrap();
		let ty = self.mir_ty_to_llvm_ty(&ptr_cast.to_ty);
		unsafe {
			let v = LLVMBuildBitCast(self.builder, *ptr, ty, str_to_cstring(""));
			self.locals.insert(ptr_cast.id, v);
		}
	}

	fn cmp_eq(&mut self, cmp_eq: &mir::CmpEq) {
		use mir::Type;
		let lhs = self.mir_rval_to_llvm_val(&cmp_eq.lhs);
		let rhs = self.mir_rval_to_llvm_val(&cmp_eq.rhs);
		unsafe {
			match cmp_eq.ty {
				Type::I64
				| Type::U64
				| Type::I32
				| Type::U32
				| Type::I16
				| Type::U16
				| Type::I8
				| Type::U8 => {
					let v = LLVMBuildICmp(
						self.builder,
						llvm_sys::LLVMIntPredicate::LLVMIntEQ,
						lhs,
						rhs,
						str_to_cstring(""),
					);
					self.locals.insert(cmp_eq.id, v);
				}
				_ => (),
			}
		}
	}

	fn br_cond(&self, br_cond: &mir::BrCond) {
		let cond = self.mir_rval_to_llvm_val(&br_cond.cond);
		let then = self.blocks.get(&br_cond.then).unwrap();
		let else_ = self.blocks.get(&br_cond.else_).unwrap();
		unsafe {
			LLVMBuildCondBr(self.builder, cond, *then, *else_);
		}
	}

	fn br(&self, br: &mir::Br) {
		let to = self.blocks.get(&br.to).unwrap();
		unsafe {
			LLVMBuildBr(self.builder, *to);
		}
	}

	fn ret(&self, ret: &mir::Ret) {
		if let Some(v) = &ret.val {
			let v = self.mir_rval_to_llvm_val(v);
			unsafe {
				LLVMBuildRet(self.builder, v);
			}
		} else {
			unsafe {
				LLVMBuildRetVoid(self.builder);
			}
		}
	}

	fn mir_rval_to_llvm_val(&self, rval: &mir::RValue) -> LLVMValueRef {
		use mir::RValue;
		unsafe {
			match rval {
				RValue::I64(x) => LLVMConstInt(LLVMInt64TypeInContext(self.ctx), *x as u64, 1),
				RValue::U64(x) => LLVMConstInt(LLVMInt64TypeInContext(self.ctx), *x as u64, 0),
				RValue::I32(x) => LLVMConstInt(LLVMInt32TypeInContext(self.ctx), *x as u64, 1),
				RValue::U32(x) => LLVMConstInt(LLVMInt32TypeInContext(self.ctx), *x as u64, 0),
				RValue::I16(x) => LLVMConstInt(LLVMInt16TypeInContext(self.ctx), *x as u64, 1),
				RValue::U16(x) => LLVMConstInt(LLVMInt16TypeInContext(self.ctx), *x as u64, 0),
				RValue::I8(x) => LLVMConstInt(LLVMInt8TypeInContext(self.ctx), *x as u64, 1),
				RValue::U8(x) => LLVMConstInt(LLVMInt8TypeInContext(self.ctx), *x as u64, 0),
				RValue::F64(x) => LLVMConstReal(LLVMDoubleTypeInContext(self.ctx), *x),
				RValue::F32(x) => LLVMConstReal(LLVMFloatTypeInContext(self.ctx), *x as f64),
				RValue::Local(local) => *self.locals.get(local).unwrap(),
			}
		}
	}

	fn mir_ty_to_llvm_ty(&self, ty: &mir::Type) -> LLVMTypeRef {
		use mir::Type;
		unsafe {
			match ty {
				Type::I64 | Type::U64 => LLVMInt64TypeInContext(self.ctx),
				Type::I32 | Type::U32 => LLVMInt32TypeInContext(self.ctx),
				Type::I16 | Type::U16 => LLVMInt16TypeInContext(self.ctx),
				Type::I8 | Type::U8 => LLVMInt8TypeInContext(self.ctx),
				Type::F64 => LLVMDoubleTypeInContext(self.ctx),
				Type::F32 => LLVMFloatTypeInContext(self.ctx),
				Type::Vector(vec) => LLVMVectorType(self.mir_ty_to_llvm_ty(&vec.ty), vec.count),
				Type::Ident(ident) => *self.struct_types.get(ident).unwrap(),
				Type::Ptr(ptr) => LLVMPointerType(self.mir_ty_to_llvm_ty(&*ptr), 0),
				_ => panic!(),
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

unsafe fn cstring_to_string(s: *const i8) -> String {
	CStr::from_ptr(s).to_str().unwrap().to_owned()
}

pub fn lower_mir_module(mir_module: MIRModule, settings: &CompilationSettings) {
	let mut codegen = Codegen::new(mir_module.name, settings);
	codegen.init_fpm();
	for (name, ty) in &mir_module.struct_tys {
		codegen.struct_ty(ty, &name);
	}

	for (_, f) in &mir_module.functions {
		codegen.fn_decl(f);
	}

	unsafe {
		LLVMDumpModule(codegen.module);
	}
}
