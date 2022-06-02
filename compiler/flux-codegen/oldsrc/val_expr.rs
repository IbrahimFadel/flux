// use super::*;

// impl<'a> Codegen<'a> {
// 	pub fn expr(&mut self, expr: &Expr) -> PIValueResult {
// 		match expr {
// 			Expr::IntLit(int) => self.int(int),
// 			Expr::FloatLit(float) => self.float(float),
// 			Expr::BinOp(binop) => self.binop(binop),
// 			Expr::Ident(ident) => self.ident(ident),
// 			Expr::CallExpr(call) => self.call(call),
// 			Expr::Unary(unary) => self.unary(unary),
// 			Expr::StructExpr(struct_expr) => self.struct_expr(struct_expr),
// 			_ => panic!("unexpected expression"),
// 		}
// 	}

// 	fn struct_expr(&mut self, struct_expr: &Spanned<StructExpr>) -> PIValueResult {
// 		unsafe {
// 			let mut values = vec![];
// 			for (_, val) in &struct_expr.fields {
// 				values.push(self.expr(val.as_ref().unwrap()));
// 			}

// 			let (_, dest_ty) = self
// 				.types
// 				.get(&struct_expr.name.to_string())
// 				.expect("internal compiler error");

// 			let ptr = LLVMBuildAlloca(self.builder, *dest_ty, str_to_cstring(""));
// 			let loaded = LLVMBuildLoad2(self.builder, *dest_ty, ptr, str_to_cstring(""));

// 			let mut i = 0;
// 			for (val, err) in values {
// 				if let Some(err) = err {
// 					return (ptr::null_mut(), Some(err));
// 				}
// 				let gep = LLVMBuildStructGEP2(self.builder, LLVMTypeOf(loaded), ptr, i, str_to_cstring(""));
// 				LLVMBuildStore(self.builder, val, gep);
// 				i += 1;
// 			}

// 			return (
// 				LLVMBuildLoad2(self.builder, *dest_ty, ptr, str_to_cstring("")),
// 				None,
// 			);
// 		}
// 	}

// 	fn unary(&mut self, unary: &Unary) -> PIValueResult {
// 		unsafe {
// 			match unary.op {
// 				OpKind::Ampersand => {
// 					let (v, err) = self.expr(&*unary.val);
// 					if let Some(err) = err {
// 						return (ptr::null_mut(), Some(err));
// 					}
// 					(LLVMGetOperand(v, 0), None)
// 				}
// 				_ => (ptr::null_mut(), None),
// 			}
// 		}
// 	}

// 	fn call(&mut self, call: &CallExpr) -> PIValueResult {
// 		unsafe {
// 			let (callee, err): PIValueResult = match &*call.callee {
// 				Expr::Ident(name) => self.values_map.get_function(name),
// 				Expr::BinOp(binop) => {
// 					if binop.op == OpKind::Doublecolon {
// 						self.double_colon_callee(binop)
// 					} else {
// 						let (e, err) = self.expr(&*call.callee);
// 						if let Some(err) = err {
// 							(ptr::null_mut(), Some(err))
// 						} else {
// 							(e, None)
// 						}
// 					}
// 				}
// 				_ => {
// 					let (e, err) = self.expr(&*call.callee);
// 					if let Some(err) = err {
// 						(ptr::null_mut(), Some(err))
// 					} else {
// 						(e, None)
// 					}
// 				}
// 			};
// 			if let Some(err) = err {
// 				return (ptr::null_mut(), Some(err));
// 			}

// 			let mut args = vec![];
// 			for arg in &call.args {
// 				let (v, err) = self.expr(&*arg);
// 				if let Some(err) = err {
// 					return (ptr::null_mut(), Some(err));
// 				}
// 				args.push(v);
// 			}

// 			(
// 				LLVMBuildCall2(
// 					self.builder,
// 					LLVMGetReturnType(LLVMTypeOf(callee)),
// 					callee,
// 					args.as_mut_ptr(),
// 					args.len() as u32,
// 					str_to_cstring(""),
// 				),
// 				None,
// 			)
// 		}
// 	}

// 	fn double_colon_callee(&self, binop: &BinOp) -> PIValueResult {
// 		if let Expr::Ident(lhs) = &*binop.x {
// 			if let Expr::Ident(rhs) = &*binop.y {
// 				let name_val = lhs.val.to_string() + "_" + rhs.as_str();
// 				let name = Ident::new(lhs.span.start..rhs.span.end, SmolStr::from(name_val));
// 				return self.values_map.get_function(&name);
// 			}
// 		}
// 		(ptr::null_mut(), None)
// 	}

// 	fn ident(&mut self, ident: &Ident) -> PIValueResult {
// 		if let Some(x) = self
// 			.symbol_table
// 			.find_val_in_scope(&self.symbol_table.cur_scope, &ident.to_string())
// 		{
// 			unsafe {
// 				return (
// 					LLVMBuildLoad2(
// 						self.builder,
// 						LLVMGetElementType(LLVMTypeOf(x)),
// 						x,
// 						str_to_cstring(""),
// 					),
// 					None,
// 				);
// 			}
// 		} else {
// 			return (
// 				ptr::null_mut(),
// 				Some(self.error(
// 					format!("unknown variable referenced `{}`", ident.to_string()),
// 					FluxErrorCode::CodegenUnknownVarReferenced,
// 					vec![(
// 						format!("could not find variable `{}`", ident.to_string()),
// 						ident.span.clone(),
// 					)],
// 				)),
// 			);
// 		}
// 	}

// 	fn binop(&mut self, binop: &BinOp) -> PIValueResult {
// 		unsafe {
// 			match binop.op {
// 				OpKind::Plus => {
// 					let (x, err) = self.expr(&*binop.x);
// 					if let Some(err) = err {
// 						return (ptr::null_mut(), Some(err));
// 					}
// 					let (y, err) = self.expr(&*binop.y);
// 					if let Some(err) = err {
// 						return (ptr::null_mut(), Some(err));
// 					}
// 					(LLVMBuildAdd(self.builder, x, y, str_to_cstring("")), None)
// 				}
// 				OpKind::CmpEQ => self.binop_cmp_eq(binop),
// 				OpKind::Eq => self.binop_eq(binop),
// 				OpKind::Period => self.binop_period(binop),
// 				_ => panic!("unexpected binop"),
// 			}
// 		}
// 	}

// 	fn binop_period(&mut self, binop: &BinOp) -> PIValueResult {
// 		let (lhs, err) = self.expr(&binop.x);
// 		if let Some(err) = err {
// 			return (ptr::null_mut(), Some(err));
// 		}
// 		let field_name = match &*binop.y {
// 			Expr::Ident(name) => name,
// 			_ => panic!("rhs of struct access should be ident"),
// 		};

// 		unsafe {
// 			if LLVMGetTypeKind(LLVMTypeOf(lhs)) != LLVMTypeKind::LLVMStructTypeKind {
// 				panic!("expected struct on lhs of `.` binop expression");
// 			}
// 			let struct_ty_name = cstring_to_string(LLVMGetStructName(LLVMTypeOf(lhs)));

// 			let (ast_struct_ty_expr, _) = self
// 				.types
// 				.get(&struct_ty_name)
// 				.expect("type decl not in types map: internal compiler error");

// 			let ast_struct_ty = match &ast_struct_ty_expr.type_ {
// 				Expr::StructType(struct_ty) => struct_ty,
// 				_ => panic!("expected lhs to be struct type"),
// 			};

// 			if let Some(_) = ast_struct_ty.get(field_name) {
// 				let mut i: i32 = -1;
// 				for (name, _) in ast_struct_ty {
// 					if name == field_name {
// 						break;
// 					}
// 					i += 1;
// 				}

// 				let gep = LLVMBuildStructGEP2(
// 					self.builder,
// 					LLVMTypeOf(lhs),
// 					LLVMGetOperand(lhs, 0),
// 					(i + 1) as u32,
// 					str_to_cstring(""),
// 				);
// 				(
// 					LLVMBuildLoad2(
// 						self.builder,
// 						LLVMGetElementType(LLVMTypeOf(gep)),
// 						gep,
// 						str_to_cstring(""),
// 					),
// 					None,
// 				)
// 			} else {
// 				let method_name =
// 					ast_struct_ty_expr.name.to_string() + &String::from("_") + &field_name.to_string();
// 				let f = LLVMGetNamedFunction(self.module, str_to_cstring(method_name.as_str()));
// 				if f == ptr::null_mut() {
// 					if let Some(interfaces) = self
// 						.struct_implementations_map
// 						.get(&ast_struct_ty_expr.name.to_string())
// 					{
// 						for interface in interfaces {
// 							let method_name = interface.to_owned()
// 								+ "_" + &ast_struct_ty_expr.name.to_string()
// 								+ &String::from("_")
// 								+ &field_name.to_string();
// 							let f = LLVMGetNamedFunction(self.module, str_to_cstring(method_name.as_str()));
// 							if f != ptr::null_mut() {
// 								return (f, None);
// 							}
// 						}
// 					}
// 					return (
// 						ptr::null_mut(),
// 						Some(self.error(
// 							format!("could not find method `{}`", field_name.to_string()),
// 							FluxErrorCode::CodegenCouldNotFindMethod,
// 							vec![],
// 						)),
// 					);
// 				} else {
// 					return (f, None);
// 				}
// 			}
// 		}
// 	}

// 	fn binop_eq(&mut self, binop: &BinOp) -> PIValueResult {
// 		unsafe {
// 			let (lhs, err) = self.expr(&binop.x);
// 			if let Some(err) = err {
// 				return (ptr::null_mut(), Some(err));
// 			}
// 			let (rhs, err) = self.expr(&binop.y);
// 			if let Some(err) = err {
// 				return (ptr::null_mut(), Some(err));
// 			}

// 			(
// 				LLVMBuildStore(self.builder, rhs, LLVMGetOperand(lhs, 0)),
// 				None,
// 			)
// 		}
// 	}

// 	fn binop_cmp_eq(&mut self, binop: &BinOp) -> PIValueResult {
// 		let (lhs, err) = self.expr(&binop.x);
// 		if let Some(err) = err {
// 			return (ptr::null_mut(), Some(err));
// 		}
// 		let (rhs, err) = self.expr(&binop.y);
// 		if let Some(err) = err {
// 			return (ptr::null_mut(), Some(err));
// 		}

// 		unsafe {
// 			let lhs_ty = LLVMTypeOf(lhs);
// 			match LLVMGetTypeKind(lhs_ty) {
// 				LLVMTypeKind::LLVMIntegerTypeKind => (
// 					LLVMBuildICmp(
// 						self.builder,
// 						llvm::LLVMIntPredicate::LLVMIntEQ,
// 						lhs,
// 						rhs,
// 						str_to_cstring(""),
// 					),
// 					None,
// 				),
// 				_ => (
// 					ptr::null_mut(),
// 					Some(self.error(
// 						format!(
// 							"cannot compare the two values: `{:?}` and `{:?}`",
// 							binop.x, binop.y
// 						),
// 						FluxErrorCode::CodegenCouldNotCmpValsOfType,
// 						vec![],
// 					)),
// 				),
// 			}
// 		}
// 	}

// 	fn float(&self, float: &FloatLit) -> PIValueResult {
// 		unsafe {
// 			let ty = match float.bits {
// 				64 => LLVMDoubleTypeInContext(self.ctx),
// 				32 => LLVMFloatTypeInContext(self.ctx),
// 				_ => LLVMFloatTypeInContext(self.ctx),
// 			};
// 			let val = match *float.signed {
// 				true => *float.val * -1.0,
// 				_ => *float.val,
// 			};
// 			(LLVMConstReal(ty, val), None)
// 		}
// 	}

// 	#[inline(always)]
// 	fn int(&self, int: &IntLit) -> PIValueResult {
// 		unsafe {
// 			(
// 				LLVMConstInt(LLVMIntType(int.bits as u32), *int.val, *int.signed as i32),
// 				None,
// 			)
// 		}
// 	}
// }
