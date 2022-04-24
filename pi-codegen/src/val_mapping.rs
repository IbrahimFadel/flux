// use std::{collections::HashMap, ops::Range};

// use llvm_sys::prelude::LLVMValueRef;
// use pi_ast::Ident;
// use pi_error::{filesystem::FileId, PIError, PIErrorCode};
// use std::ptr;
// use uuid::Uuid;

// use crate::PIValueResult;

// #[derive(Debug)]
// pub struct CodegenValuesMap {
// 	file_id: FileId,
// 	functions_map: HashMap<Uuid, LLVMValueRef>,
// 	functions_uuid_map: HashMap<String, Uuid>,
// }

// impl CodegenValuesMap {
// 	pub fn new(file_id: &FileId) -> Self {
// 		Self {
// 			file_id: file_id.clone(),
// 			functions_map: HashMap::new(),
// 			functions_uuid_map: HashMap::new(),
// 		}
// 	}

// 	#[inline(always)]
// 	fn error(&self, msg: String, code: PIErrorCode, labels: Vec<(String, Range<usize>)>) -> PIError {
// 		PIError::new(msg, code, labels, self.file_id)
// 	}

// 	pub fn set_new_function(&mut self, name: String, uuid: Uuid, function: LLVMValueRef) {
// 		self.functions_uuid_map.insert(name, uuid);
// 		self.functions_map.insert(uuid, function);
// 	}

// 	pub fn get_function(&self, name: &Ident) -> PIValueResult {
// 		if let Some(uuid) = self.functions_uuid_map.get(&name.to_string()) {
// 			if let Some(f) = self.functions_map.get(uuid) {
// 				(*f, None)
// 			} else {
// 				(
// 					ptr::null_mut(),
// 					Some(self.error(
// 						format!("unknown function `{}` referenced", name.to_string()),
// 						PIErrorCode::CodegenUnknownFnReferenced,
// 						vec![(
// 							format!("could not find function `{}`", name.to_string()),
// 							name.span.clone(),
// 						)],
// 					)),
// 				)
// 			}
// 		} else {
// 			(
// 				ptr::null_mut(),
// 				Some(self.error(
// 					format!("unknown function `{}` referenced", name.to_string()),
// 					PIErrorCode::CodegenUnknownFnReferenced,
// 					vec![(
// 						format!("could not find function `{}`", name.to_string()),
// 						name.span.clone(),
// 					)],
// 				)),
// 			)
// 		}
// 	}
// }
