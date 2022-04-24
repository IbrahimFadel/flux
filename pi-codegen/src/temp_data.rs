// use llvm_sys::prelude::{LLVMBasicBlockRef, LLVMValueRef};
// use pi_ast::TypeDecl;

// #[derive(Debug)]
// pub struct CodegenTempData<'a> {
// 	pub cur_type_decl: Option<&'a TypeDecl>,
// 	pub cur_fn_name: String,
// 	pub cur_fn: Option<LLVMValueRef>,
// 	pub cur_bb: Option<LLVMBasicBlockRef>,
// 	pub merging_bb: Option<LLVMBasicBlockRef>,
// 	pub mod_name: String,
// }

// impl<'a> CodegenTempData<'a> {
// 	pub fn new() -> Self {
// 		Self {
// 			cur_type_decl: None,
// 			cur_fn_name: String::new(),
// 			cur_fn: None,
// 			cur_bb: None,
// 			merging_bb: None,
// 			mod_name: String::new(),
// 		}
// 	}
// }
