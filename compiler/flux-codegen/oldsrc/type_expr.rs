// use super::*;

// impl<'a> Codegen<'a> {
// 	pub fn type_expr(&mut self, e: &Spanned<Expr>) -> PITypeResult {
// 		unsafe {
// 			match &**e {
// 				Expr::StructType(struct_ty) => self.struct_type(struct_ty),
// 				Expr::InterfaceType(interface_ty) => self.interface_type(interface_ty),
// 				Expr::PtrType(ptr) => self.ptr_type(ptr),
// 				Expr::PrimitiveType(prim) => match prim.kind {
// 					PrimitiveKind::I64 | PrimitiveKind::U64 => (LLVMInt64TypeInContext(self.ctx), None),
// 					PrimitiveKind::I32 | PrimitiveKind::U32 => (LLVMInt32TypeInContext(self.ctx), None),
// 					PrimitiveKind::I16 | PrimitiveKind::U16 => (LLVMInt16TypeInContext(self.ctx), None),
// 					PrimitiveKind::I8 | PrimitiveKind::U8 => (LLVMInt8TypeInContext(self.ctx), None),
// 					PrimitiveKind::F64 => (LLVMDoubleTypeInContext(self.ctx), None),
// 					PrimitiveKind::F32 => (LLVMFloatTypeInContext(self.ctx), None),
// 					PrimitiveKind::Void => (LLVMVoidTypeInContext(self.ctx), None),
// 					_ => (
// 						ptr::null_mut(),
// 						Some(self.error(
// 							format!("could not codegen type `{:?}`", e),
// 							FluxErrorCode::CodegenCouldNotCodegenTypeExpr,
// 							vec![],
// 						)),
// 					),
// 				},
// 				Expr::Ident(ident) => self.ident_type(&Spanned::new(*ident, e.span.clone())),
// 				_ => (
// 					ptr::null_mut(),
// 					Some(self.error(
// 						format!("could not codegen type `{:?}`", e),
// 						FluxErrorCode::CodegenCouldNotCodegenTypeExpr,
// 						vec![],
// 					)),
// 				),
// 			}
// 		}
// 	}

// 	fn struct_type(&mut self, struct_ty: &flux_ast::StructType) -> PITypeResult {
// 		unsafe {
// 			let mut field_types = vec![];
// 			for (_, field) in struct_ty {
// 				let (ty, err) = self.type_expr(&field.type_);
// 				if let Some(err) = err {
// 					return (ptr::null_mut(), Some(err));
// 				}
// 				field_types.push(ty);
// 			}

// 			let llvm_struct_ty = LLVMStructCreateNamed(
// 				self.ctx,
// 				str_to_cstring(self.tmp_data.cur_type_decl.unwrap().name.as_str()),
// 			);
// 			LLVMStructSetBody(
// 				llvm_struct_ty,
// 				field_types.as_mut_ptr(),
// 				struct_ty.len() as u32,
// 				0,
// 			);
// 			return (llvm_struct_ty, None);
// 		}
// 	}

// 	fn interface_type(&mut self, interface_ty: &InterfaceType) -> PITypeResult {
// 		unsafe {
// 			let llvm_struct_ty = LLVMStructCreateNamed(
// 				self.ctx,
// 				str_to_cstring(self.tmp_data.cur_type_decl.unwrap().name.as_str()),
// 			);
// 			self.types.insert(
// 				self.tmp_data.cur_type_decl.unwrap().name.to_string(),
// 				(self.tmp_data.cur_type_decl.unwrap().clone(), llvm_struct_ty),
// 			);

// 			let mut method_types = vec![];
// 			for (_, method) in interface_ty {
// 				let (proto, err) = self.fn_proto(&method.ret_ty, &method.params);
// 				if let Some(err) = err {
// 					return (ptr::null_mut(), Some(err));
// 				}
// 				method_types.push(proto);
// 			}

// 			LLVMStructSetBody(
// 				llvm_struct_ty,
// 				method_types.as_mut_ptr(),
// 				interface_ty.len() as u32,
// 				0,
// 			);

// 			return (llvm_struct_ty, None);
// 		}
// 	}

// 	#[inline(always)]
// 	fn ptr_type(&mut self, ptr: &PtrType) -> PITypeResult {
// 		unsafe {
// 			let (ty, err) = self.type_expr(&*ptr);
// 			if let Some(err) = err {
// 				return (ptr::null_mut(), Some(err));
// 			}
// 			(LLVMPointerType(ty, 0), None)
// 		}
// 	}

// 	fn ident_type(&mut self, ident: &Spanned<Ident>) -> PITypeResult {
// 		match self.types.get(&ident.to_string()) {
// 			Some((_, ty)) => (*ty, None),
// 			None => (
// 				ptr::null_mut(),
// 				Some(self.error(
// 					format!("could not find type `{}`", ident.to_string()),
// 					FluxErrorCode::CodegenUnknownIdentType,
// 					vec![(
// 						format!("unkown type `{}` referenced", ident.to_string()),
// 						ident.span.clone(),
// 					)],
// 				)),
// 			),
// 		}
// 	}
// }
