// use pi_ast::{BinOp, CallExpr, Field, FloatLit, OpKind, StructExpr, Unary};

// use super::*;

// impl<'a> TypecheckCtx<'a> {
// 	pub fn check_expr(&mut self, expr: &'a mut Expr) -> Option<PIError> {
// 		match expr {
// 			Expr::IntLit(int) => self.check_int_lit(int),
// 			Expr::FloatLit(float) => self.check_float_lit(float),
// 			Expr::BinOp(binop) => self.check_binop(binop),
// 			Expr::CallExpr(call) => self.check_call(call),
// 			Expr::StructExpr(struct_expr) => self.check_struct_expr(struct_expr),
// 			_ => None,
// 		}
// 	}

// 	fn check_struct_expr(&self, struct_expr: &mut StructExpr) -> Option<PIError> {
// 		for (name, val) in &mut struct_expr.fields.iter_mut() {
// 			if val.is_none() {
// 				*val = Some(Box::from(Spanned::new(Expr::Ident((**name).clone()), 0..0)));
// 			}
// 		}

// 		if let Some(struct_ty_decl) = self.types.get(&struct_expr.name.to_string()) {
// 			if let Expr::StructType(struct_ty) = &*struct_ty_decl.type_ {
// 				if let Some(err) = self.compare_struct_expr_fields_to_struct_ty(
// 					struct_expr,
// 					struct_ty,
// 					&struct_ty_decl.name.to_string(),
// 				) {
// 					return Some(err);
// 				}
// 			}
// 		}

// 		return None;
// 	}

// 	fn compare_struct_expr_fields_to_struct_ty(
// 		&self,
// 		struct_expr: &mut StructExpr,
// 		struct_ty: &IndexMap<Spanned<Ident>, Spanned<Field>>,
// 		struct_ty_name: &String,
// 	) -> Option<PIError> {
// 		if struct_expr.fields.len() != struct_ty.len() {
// 			return Some(self.error(
// 				"struct expression does not have the same number of fields as the type it is constructing".to_owned(),
// 				PIErrorCode::TypecheckStructExprDiffNumberFieldsAsStructTy,
// 				vec![
// 					("incorrect number of fields in struct expression".to_owned(), struct_expr.fields.span.clone())
// 				],
// 			));
// 		}

// 		for (name, field) in struct_ty {
// 			if let Some(struct_expr_val_opt) = struct_expr.fields.get_mut(name) {
// 				let struct_expr_val = struct_expr_val_opt
// 					.as_mut()
// 					.expect("internal compiler error");
// 				let res = match &mut ***struct_expr_val {
// 					Expr::IntLit(int) => match &*field.type_ {
// 						Expr::PrimitiveType(prim) => match prim.kind {
// 							PrimitiveKind::I64 | PrimitiveKind::U64 => {
// 								int.bits = 64;
// 								None
// 							}
// 							PrimitiveKind::I32 | PrimitiveKind::U32 => {
// 								int.bits = 32;
// 								None
// 							}
// 							PrimitiveKind::I16 | PrimitiveKind::U16 => {
// 								int.bits = 16;
// 								None
// 							}
// 							PrimitiveKind::I8 | PrimitiveKind::U8 => {
// 								int.bits = 8;
// 								None
// 							}
// 							_ => Some(self.error(
// 								format!(
// 									"expected struct expression's field value to be of type `{:?}`",
// 									prim.kind
// 								),
// 								PIErrorCode::CodegenUnknownIdentType,
// 								vec![],
// 							)),
// 						},
// 						_ => Some(self.error(
// 							format!(
// 								"expected struct expression's field value to be of type `{}`",
// 								*field.type_
// 							),
// 							PIErrorCode::CodegenUnknownIdentType,
// 							vec![],
// 						)),
// 					},
// 					Expr::FloatLit(float) => match &*field.type_ {
// 						Expr::PrimitiveType(prim) => match prim.kind {
// 							PrimitiveKind::F64 => {
// 								float.bits = 64;
// 								None
// 							}
// 							PrimitiveKind::F32 => {
// 								float.bits = 32;
// 								None
// 							}
// 							_ => Some(self.error(
// 								format!(
// 									"expected struct expression's field value to be of type `{:?}`",
// 									prim.kind
// 								),
// 								PIErrorCode::CodegenUnknownIdentType,
// 								vec![],
// 							)),
// 						},
// 						_ => Some(self.error(
// 							format!(
// 								"expected struct expression's field value to be of type `{}`",
// 								*field.type_
// 							),
// 							PIErrorCode::CodegenUnknownIdentType,
// 							vec![],
// 						)),
// 					},
// 					Expr::StructExpr(sub_struct_expr) => self.check_struct_expr(sub_struct_expr),
// 					_ => None,
// 				};
// 				if let Some(err) = res {
// 					return Some(err);
// 				}
// 			} else {
// 				return Some(self.error(
// 					format!(
// 						"could not find field `{}` in struct expression",
// 						name.to_string()
// 					),
// 					PIErrorCode::TypecheckCouldNotFindFieldInStructExpr,
// 					vec![
// 						(
// 							format!(
// 								"expected field `{}` in `{}` struct expression",
// 								name.to_string(),
// 								struct_ty_name,
// 							),
// 							name.span.clone(),
// 						),
// 						(
// 							"instead got these fields".to_owned(),
// 							struct_expr.fields.span.clone(),
// 						),
// 					],
// 				));
// 			}
// 		}

// 		return None;
// 	}

// 	fn check_call(&self, call: &mut CallExpr) -> Option<PIError> {
// 		match &**call.callee {
// 			Expr::BinOp(binop) => {
// 				let (expr, err) =
// 					self.check_binop_call(&Spanned::new((*binop).clone(), call.callee.span.clone()));
// 				if let Some(err) = err {
// 					return Some(err);
// 				}
// 				if let Some(expr) = expr {
// 					call.args.splice(..0, [expr]);
// 				}
// 				None
// 			}
// 			_ => None,
// 		}
// 	}

// 	fn check_binop_call(
// 		&self,
// 		binop: &Spanned<BinOp>,
// 	) -> (Option<Box<Spanned<Expr>>>, Option<PIError>) {
// 		match &**binop.x {
// 			Expr::Ident(var_name) => match binop.op {
// 				OpKind::Period => self.check_binop_struct_access_call(
// 					binop,
// 					&Spanned::new((*var_name).clone(), binop.x.span.clone()),
// 				),
// 				OpKind::Doublecolon => {
// 					if let Some(err) = self.check_binop_double_colon_call(binop) {
// 						return (None, Some(err));
// 					}
// 					(None, None)
// 				}
// 				_ => (None, None),
// 			},
// 			_ => (None, None),
// 		}
// 	}

// 	fn check_binop_double_colon_call(&self, binop: &BinOp) -> Option<PIError> {
// 		if let Expr::Ident(mod_name) = &**binop.x {
// 			if let Expr::Ident(rhs) = &**binop.y {}
// 		}
// 		None
// 	}

// 	fn check_binop_struct_access_call(
// 		&self,
// 		binop: &BinOp,
// 		var_name: &Spanned<Ident>,
// 	) -> (Option<Box<Spanned<Expr>>>, Option<PIError>) {
// 		let (ty, err) = self.get_type_of_var_in_cur_block(var_name);
// 		if let Some(err) = err {
// 			return (None, Some(err));
// 		}
// 		if let Expr::Ident(struct_name) = ty {
// 			let methods = self.struct_methods.get(&struct_name.to_string()).unwrap();

// 			if let Expr::Ident(rhs) = &**binop.y {
// 				let method = methods.get(&rhs.to_string()).unwrap();
// 				if method.params.len() > 0 {
// 					if *method.params[0].name == "this" {
// 						return (
// 							Some(Box::from(Spanned::new(
// 								Expr::Unary(Unary::new(OpKind::Ampersand, binop.x.clone())),
// 								0..0,
// 							))),
// 							None,
// 						);
// 					}
// 				}
// 			}
// 		}
// 		(None, None)
// 	}

// 	fn check_binop(&mut self, binop: &'a mut BinOp) -> Option<PIError> {
// 		match binop.op {
// 			OpKind::Eq => self.check_binop_eq(binop),
// 			OpKind::Doublecolon => self.check_binop_double_colon(binop),
// 			_ => {
// 				if let Some(err) = self.check_expr(&mut *binop.x) {
// 					return Some(err);
// 				}
// 				if let Some(err) = self.check_expr(&mut *binop.y) {
// 					return Some(err);
// 				}
// 				None
// 			}
// 		}
// 	}

// 	fn check_binop_double_colon(&self, binop: &'a BinOp) -> Option<PIError> {
// 		None
// 	}

// 	fn check_binop_eq(&mut self, binop: &'a BinOp) -> Option<PIError> {
// 		match &**binop.x {
// 			Expr::BinOp(b) => {
// 				let (expr, err) = self.get_struct_access_type(b);
// 				if let Some(err) = err {
// 					return Some(err);
// 				}
// 				self.expecting_ty = Some(expr);
// 				None
// 			}
// 			_ => None,
// 		}
// 	}

// 	fn get_struct_access_type(&mut self, binop: &'a BinOp) -> (&'a Expr, Option<PIError>) {
// 		let mut b = binop;
// 		let mut field_names = vec![];
// 		if let Expr::Ident(rhs) = &**b.y {
// 			field_names.push(Spanned::new((*rhs).clone(), binop.y.span.clone()));
// 		}
// 		while let Expr::BinOp(sub_binop) = &**b.x {
// 			if sub_binop.op != OpKind::Period {
// 				return (
// 					&Expr::Error,
// 					Some(self.error(
// 						"expected `.` operator in chained struct field access".to_owned(),
// 						PIErrorCode::TypecheckExpectedPeriodOpInChainedStructFieldAccess,
// 						vec![],
// 					)),
// 				);
// 			}
// 			if let Expr::Ident(rhs) = &**sub_binop.y {
// 				field_names.push(Spanned::new((*rhs).clone(), sub_binop.y.span.clone()));
// 			} else {
// 				return (
// 					&Expr::Error,
// 					Some(self.error(
// 						"expected rhs of `.` operator to be identifier".to_owned(),
// 						PIErrorCode::TypecheckExpectedRHSOfPeriodToBeIdent,
// 						vec![],
// 					)),
// 				);
// 			}
// 			b = sub_binop;
// 		}
// 		if let Expr::Ident(rhs) = &**b.x {
// 			field_names.push(Spanned::new((*rhs).clone(), b.x.span.clone()));
// 		}

// 		let struct_var_name = field_names.last_mut().cloned().unwrap();
// 		let (mut struct_var_type_name, err) = self.get_type_of_var_in_cur_block(&struct_var_name);
// 		if let Some(err) = err {
// 			return (&Expr::Error, Some(err));
// 		}
// 		field_names.pop();
// 		while let Expr::PtrType(ptr) = struct_var_type_name {
// 			struct_var_type_name = &**ptr;
// 		}
// 		if let Expr::Ident(name) = struct_var_type_name {
// 			let struct_var_type = &self.types.get(&name.to_string()).as_ref().unwrap().type_;
// 			// 	.clone();
// 			if let Expr::StructType(struct_ty) = &**struct_var_type {
// 				// let (expr, err) = self.find_rightmost_field_type(&mut field_names, &struct_ty);
// 				let (expr, err) = self.find_rightmost_field_type(&mut field_names, &struct_var_type);
// 				if let Some(err) = err {
// 					return (&Expr::Error, Some(err));
// 				} else {
// 					return (expr, None);
// 				}
// 			} else {
// 				return (
// 					&Expr::Error,
// 					Some(self.error(
// 						"expected lhs of `.` operator to be a struct".to_owned(),
// 						PIErrorCode::TypecheckExpectedLHSOfPeriodToBeStruct,
// 						vec![],
// 					)),
// 				);
// 			}
// 		}
// 		panic!("this should be fatal");
// 	}

// 	fn find_rightmost_field_type(
// 		&self,
// 		field_names: &mut Vec<Spanned<Ident>>,
// 		struct_ty_expr: &'a Spanned<Expr>,
// 	) -> (&'a Expr, Option<PIError>) {
// 		if let Expr::StructType(struct_ty) = &**struct_ty_expr {
// 			if field_names.len() == 0 {
// 				return (struct_ty_expr, None);
// 			}
// 			let field_name = field_names.pop().unwrap();

// 			if let Some(field_ty) = self.get_struct_field_type(struct_ty, &field_name) {
// 				if let Expr::Ident(struct_type_name) = &field_ty {
// 					let res = &self.types.get(&struct_type_name.to_string()).unwrap().type_;
// 					return match &**res {
// 						Expr::StructType(_) => self.find_rightmost_field_type(field_names, &res),
// 						_ => (&res, None),
// 					};
// 				} else {
// 					return (field_ty, None);
// 				}
// 			}
// 		}
// 		panic!("cant thin of msg");
// 	}

// 	fn check_float_lit(&mut self, float: &mut FloatLit) -> Option<PIError> {
// 		if let Some(Expr::PrimitiveType(prim)) = &self.expecting_ty {
// 			let expected_bits = primitive_kind_to_bits(&prim.kind);
// 			if float.bits != expected_bits {
// 				float.bits = expected_bits;
// 			}
// 		}
// 		return None;
// 	}

// 	fn check_int_lit(&mut self, int: &mut IntLit) -> Option<PIError> {
// 		if let Some(Expr::PrimitiveType(prim)) = &self.expecting_ty {
// 			self.reassign_int_lit_bits(int, prim);
// 		} else if let Some(Expr::Ident(ident)) = &self.expecting_ty {
// 			let ty = self.types.get(&ident.to_string()).expect("expected type");
// 			if let Expr::PrimitiveType(prim) = &*ty.type_ {
// 				self.reassign_int_lit_bits(int, prim);
// 			}
// 		}
// 		return None;
// 	}

// 	fn reassign_int_lit_bits(&self, int: &mut IntLit, prim: &PrimitiveType) -> Option<PIError> {
// 		let expected_bits = primitive_kind_to_bits(&prim.kind);
// 		let expected_signed = primitive_kind_to_signedness(&prim.kind);
// 		if int.bits != expected_bits {
// 			int.bits = expected_bits;
// 		}
// 		if expected_signed == false && *int.signed == true {
// 			let mut labels = vec![("expected unsigned integer".to_owned(), int.val.span.clone())];
// 			if expected_signed == false {
// 				labels.push((format!("unexpected `-`"), int.signed.span.clone()))
// 			}
// 			return Some(self.error(
// 				format!("expected unsigned integer but got signed integer",),
// 				PIErrorCode::TypecheckUnexpectedSignednessInIntLit,
// 				labels,
// 			));
// 		}
// 		return None;
// 	}
// }
