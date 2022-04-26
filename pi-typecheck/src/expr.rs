use pi_ast::{
	BinOp, CallExpr, EnumExpr, EnumType, Field, FloatLit, Ident, IntLit, OpKind, PrimitiveType,
	StructExpr, StructType, Unary,
};
use pi_error::{PIErrorCode, Span};

use super::*;

impl<'ctx> FnCtx<'ctx> {
	pub fn check_expr(&mut self, expr: &'ctx mut Expr) -> PIResult {
		match expr {
			Expr::IntLit(int) => self.check_int_lit(int),
			Expr::FloatLit(float) => self.check_float_lit(float),
			Expr::BinOp(binop) => self.check_binop(binop),
			Expr::CallExpr(call) => {
				let res = self.check_call(call)?;
				if let Some(enum_expr) = res {
					*expr = enum_expr;
				}
				Ok(())
			}
			Expr::StructExpr(struct_expr) => self.check_struct_expr(struct_expr),
			_ => Ok(()),
		}
	}

	fn check_struct_expr(&self, struct_expr: &mut StructExpr) -> PIResult {
		for (name, val) in &mut struct_expr.fields.iter_mut() {
			if val.is_none() {
				*val = Some(Box::from(Spanned::new(
					Expr::Ident((**name).clone()),
					Span::new(0..0, self.file_id),
				)));
			}
		}

		if let Some(struct_ty_decl) = self.type_decls.get(&struct_expr.name.to_string()) {
			if let Expr::StructType(struct_ty) = &*struct_ty_decl.type_ {
				self.compare_struct_expr_fields_to_struct_ty(
					struct_expr,
					struct_ty,
					&struct_ty_decl.name.to_string(),
				)?;
			}
		}

		return Ok(());
	}

	fn compare_struct_expr_fields_to_struct_ty(
		&self,
		struct_expr: &mut StructExpr,
		struct_ty: &IndexMap<Spanned<Ident>, Spanned<Field>>,
		struct_ty_name: &String,
	) -> PIResult {
		if struct_expr.fields.len() != struct_ty.len() {
			return Err(self.error(
				"struct expression does not have the same number of fields as the type it is constructing".to_owned(),
				PIErrorCode::TypecheckStructExprDiffNumberFieldsAsStructTy,
				vec![
					("incorrect number of fields in struct expression".to_owned(), struct_expr.fields.span.clone())
				],
			));
		}

		for (name, field) in struct_ty {
			if let Some(struct_expr_val_opt) = struct_expr.fields.get_mut(name) {
				let struct_expr_val = struct_expr_val_opt
					.as_mut()
					.expect("internal compiler error");
				let res = match &mut ***struct_expr_val {
					Expr::IntLit(int) => match &*field.type_ {
						Expr::PrimitiveType(prim) => match prim {
							PrimitiveType::I64
							| PrimitiveType::I32
							| PrimitiveType::I16
							| PrimitiveType::I8
							| PrimitiveType::U64
							| PrimitiveType::U32
							| PrimitiveType::U16
							| PrimitiveType::U8 => {
								int.bits = primitive_kind_to_bits(&prim);
								Ok(())
							}
							_ => Err(self.error(
								format!(
									"expected struct expression's field value to be of type `{}`",
									*field.type_
								),
								PIErrorCode::CodegenUnknownIdentType,
								vec![
									(
										format!("expected value to be type `{}`", *field.type_),
										int.val.span.clone(),
									),
									(
										format!("field defined with type `{}` here", *field.type_),
										field.span.clone(),
									),
								],
							)),
						},
						_ => Err(self.error(
							format!(
								"expected struct expression's field value to be of type `{}`",
								*field.type_
							),
							PIErrorCode::CodegenUnknownIdentType,
							vec![
								(
									format!("expected value to be type `{}`", *field.type_),
									int.val.span.clone(),
								),
								(
									format!("field defined with type `{}` here", *field.type_),
									field.span.clone(),
								),
							],
						)),
					},
					Expr::FloatLit(float) => match &*field.type_ {
						Expr::PrimitiveType(prim) => match prim {
							PrimitiveType::F64 | PrimitiveType::F32 => {
								float.bits = primitive_kind_to_bits(&prim);
								Ok(())
							}
							_ => {
								println!("{:?}", field);
								Err(self.error(
									format!(
										"expected struct expression's field value to be of type `{}`",
										*field.type_
									),
									PIErrorCode::CodegenUnknownIdentType,
									vec![
										(
											format!("expected value to be type `{}`", *field.type_),
											float.val.span.clone(),
										),
										(
											format!("field defined with type `{}` here", *field.type_),
											field.span.clone(),
										),
									],
								))
							}
						},
						_ => Err(self.error(
							format!(
								"expected struct expression's field value to be of type `{}`",
								*field.type_
							),
							PIErrorCode::CodegenUnknownIdentType,
							vec![
								(
									format!("expected value to be type `{}`", *field.type_),
									float.val.span.clone(),
								),
								(
									format!("field defined with type `{}` here", *field.type_),
									field.span.clone(),
								),
							],
						)),
					},
					Expr::StructExpr(sub_struct_expr) => self.check_struct_expr(sub_struct_expr),
					_ => Ok(()),
				};
				if let Some(err) = res.err() {
					return Err(err);
				}
			} else {
				return Err(self.error(
					format!(
						"could not find field `{}` in struct expression",
						name.to_string()
					),
					PIErrorCode::TypecheckCouldNotFindFieldInStructExpr,
					vec![
						(
							format!(
								"expected field `{}` in `{}` struct expression",
								name.to_string(),
								struct_ty_name,
							),
							name.span.clone(),
						),
						(
							"instead got these fields".to_owned(),
							struct_expr.fields.span.clone(),
						),
					],
				));
			}
		}

		return Ok(());
	}

	/// Returns Some(Expr) if a call is really just an enum expression.
	/// Some(Expr) is the enum expression that the call expression should be replaced with
	fn check_call(&self, call: &mut CallExpr) -> Result<Option<Expr>, PIError> {
		match &**call.callee {
			Expr::BinOp(binop) => {
				if let Some(ty_name) = self.lhs_of_binop_is_type_name(binop) {
					if let Expr::Ident(rhs) = &**binop.y {
						if call.args.len() == 0 {
							return Err(self.error(
								format!("missing initializer in enum expression"),
								PIErrorCode::TypecheckMissingInitializerInEnumExpr,
								vec![(
									format!("missing initialzer in enum expression"),
									call.args.span.clone(),
								)],
							));
						} else if call.args.len() > 1 {
							let mut labels = vec![(
								format!("too many initialzers in enum expression"),
								call.args.span.clone(),
							)];
							for (i, arg) in call.args.iter().enumerate() {
								if i == 0 {
									continue;
								}
								labels.push((format!("unexpected initialzer"), arg.span.clone()));
							}
							return Err(self.error(
								format!("too many initializers in enum expression"),
								PIErrorCode::TypecheckTooManyInitializersInEnumExpr,
								labels,
							));
						}

						let spanned_rhs = Spanned::new(rhs.clone(), binop.y.span.clone());
						let enum_expr = EnumExpr::new(ty_name.clone(), spanned_rhs, call.args[0].clone());
						return Ok(Some(Expr::EnumExpr(enum_expr)));
					} else {
						return Err(self.error(
							format!("expected rhs of enum expression to be an identifier"),
							PIErrorCode::TypecheckExpectedRHSOfEnumExprToBeIdent,
							vec![],
						));
					}
				}

				let expr =
					self.check_binop_call(&Spanned::new((*binop).clone(), call.callee.span.clone()))?;
				if let Some(expr) = expr {
					call.args.splice(..0, [expr]);
				}
				Ok(None)
			}
			_ => Ok(None),
		}
	}

	fn lhs_of_binop_is_type_name(&self, binop: &BinOp) -> Option<Spanned<Ident>> {
		if binop.op != OpKind::Period {
			return None;
		}
		if let Expr::Ident(lhs) = &**binop.x {
			if let Some(ty_name) = self.type_decls.get(&lhs.to_string()) {
				return Some(ty_name.name.clone());
			}
		}
		return None;
	}

	/// If it's a method call, it will return an expr for a pointer to the struct to prepend to the call args
	fn check_binop_call(
		&self,
		binop: &Spanned<BinOp>,
	) -> Result<Option<Box<Spanned<Expr>>>, PIError> {
		match &**binop.x {
			Expr::Ident(var_name) => match binop.op {
				OpKind::Period => {
					if let Some(ty_name) = self.type_decls.get(&var_name.to_string()) {
						return Ok(None);
					}
					self.check_binop_struct_access_call(
						binop,
						&Spanned::new((*var_name).clone(), binop.x.span.clone()),
					)
				}
				OpKind::Doublecolon => {
					self.check_binop_double_colon_call(binop)?;
					Ok(None)
				}
				_ => Ok(None),
			},
			_ => Ok(None),
		}
	}

	fn check_binop_double_colon_call(&self, binop: &BinOp) -> PIResult {
		if let Expr::Ident(_) = &**binop.x {
			if let Expr::Ident(_) = &**binop.y {}
		}
		Ok(())
	}

	fn check_binop_struct_access_call(
		&self,
		binop: &BinOp,
		var_name: &Spanned<Ident>,
	) -> Result<Option<Box<Spanned<Expr>>>, PIError> {
		let ty = self.get_type_of_var_in_cur_block(var_name)?;
		if let Expr::Ident(struct_name) = ty {
			let methods = self.struct_methods.get(&struct_name.to_string()).unwrap();

			if let Expr::Ident(rhs) = &**binop.y {
				let method = methods.get(&rhs.to_string()).unwrap();
				if method.params.len() > 0 {
					if *method.params[0].name == "this" {
						return Ok(Some(Box::from(Spanned::new(
							Expr::Unary(Unary::new(OpKind::Ampersand, binop.x.clone())),
							Span::new(0..0, self.file_id),
						))));
					}
				}
			}
		}
		Ok(None)
	}

	fn check_binop(&mut self, binop: &'ctx mut BinOp) -> PIResult {
		match binop.op {
			OpKind::Eq => self.check_binop_eq(binop),
			OpKind::Doublecolon => self.check_binop_double_colon(binop),
			_ => {
				self.check_expr(&mut *binop.x)?;
				self.check_expr(&mut *binop.y)?;
				Ok(())
			}
		}
	}

	fn check_binop_period(&self, binop: &'ctx mut BinOp) -> PIResult {
		// let rhs = match &**binop.y {
		// 	Expr::Ident(name) => name,
		// 	_ => panic!(),
		// };
		// if let Expr::Ident(name) = &**binop.x {
		// 	if let Some(ty) = self.type_decls.get(&name.to_string()) {
		// 		if let Expr::EnumType(enum_ty) = &*ty.type_ {
		// 			return self.get_enum_access_type(enum_ty, rhs);
		// 		}
		// 	}
		// }
		Ok(())
	}

	// fn get_enum_access_type(&self, enum_ty: &EnumType, ty_name: &Spanned<Ident>) -> PIResult {
	// 	if let Some(x) = enum_ty.get(ty_name) {
	// 		return Ok(x);
	// 	} else {
	// 		Err(())
	// 	}
	// }

	fn check_binop_double_colon(&self, _: &'ctx BinOp) -> PIResult {
		Ok(())
	}

	fn check_binop_eq(&mut self, binop: &'ctx BinOp) -> PIResult {
		match &**binop.x {
			Expr::BinOp(b) => {
				let expr = self.get_struct_access_type(b)?;
				self.expecting_ty = Some(expr);
				Ok(())
			}
			_ => Ok(()),
		}
	}

	fn get_struct_access_type(&mut self, binop: &'ctx BinOp) -> Result<&'ctx Expr, PIError> {
		let mut b = binop;
		let mut field_names = vec![];
		if let Expr::Ident(rhs) = &**b.y {
			field_names.push(Spanned::new((*rhs).clone(), binop.y.span.clone()));
		}
		while let Expr::BinOp(sub_binop) = &**b.x {
			if sub_binop.op != OpKind::Period {
				return Err(self.error(
					"expected `.` operator in chained struct field access".to_owned(),
					PIErrorCode::TypecheckExpectedPeriodOpInChainedStructFieldAccess,
					vec![],
				));
			}
			if let Expr::Ident(rhs) = &**sub_binop.y {
				field_names.push(Spanned::new((*rhs).clone(), sub_binop.y.span.clone()));
			} else {
				return Err(self.error(
					"expected rhs of `.` operator to be identifier".to_owned(),
					PIErrorCode::TypecheckExpectedRHSOfPeriodToBeIdent,
					vec![],
				));
			}
			b = sub_binop;
		}
		if let Expr::Ident(rhs) = &**b.x {
			field_names.push(Spanned::new((*rhs).clone(), b.x.span.clone()));
		}

		let struct_var_name = field_names.last_mut().cloned().unwrap();
		let mut struct_var_type_name = self.get_type_of_var_in_cur_block(&struct_var_name)?;
		field_names.pop();
		while let Expr::PtrType(ptr) = struct_var_type_name {
			struct_var_type_name = &**ptr;
		}
		if let Expr::Ident(name) = struct_var_type_name {
			let struct_var_type = &self
				.type_decls
				.get(&name.to_string())
				.as_ref()
				.unwrap()
				.type_;
			if let Expr::StructType(_) = &**struct_var_type {
				let (expr, err) = self.find_rightmost_field_type(&mut field_names, &struct_var_type);
				if let Err(err) = err {
					return Err(err);
				} else {
					return Ok(expr);
				}
			} else {
				return Err(self.error(
					"expected lhs of `.` operator to be a struct".to_owned(),
					PIErrorCode::TypecheckExpectedLHSOfPeriodToBeStruct,
					vec![],
				));
			}
		}
		panic!("this should be fatal");
	}

	fn find_rightmost_field_type(
		&self,
		field_names: &mut Vec<Spanned<Ident>>,
		struct_ty_expr: &'ctx Spanned<Expr>,
	) -> (&'ctx Expr, PIResult) {
		if let Expr::StructType(struct_ty) = &**struct_ty_expr {
			if field_names.len() == 0 {
				return (struct_ty_expr, Ok(()));
			}
			let field_name = field_names.pop().unwrap();

			if let Some(field_ty) = self.get_struct_field_type(struct_ty, &field_name) {
				if let Expr::Ident(struct_type_name) = &field_ty {
					let res = &self
						.type_decls
						.get(&struct_type_name.to_string())
						.unwrap()
						.type_;
					return match &**res {
						Expr::StructType(_) => self.find_rightmost_field_type(field_names, &res),
						_ => (&res, Ok(())),
					};
				} else {
					return (field_ty, Ok(()));
				}
			}
		}
		panic!("cant thin of msg");
	}

	fn get_struct_field_type(
		&self,
		struct_ty: &'ctx StructType,
		field_name: &Spanned<Ident>,
	) -> Option<&'ctx Expr> {
		if let Some(field) = struct_ty.get(field_name) {
			return Some(&field.type_);
		}
		None
	}

	fn check_float_lit(&mut self, float: &mut FloatLit) -> PIResult {
		if let Some(Expr::PrimitiveType(prim)) = &self.expecting_ty {
			match prim {
				PrimitiveType::I64
				| PrimitiveType::I32
				| PrimitiveType::I16
				| PrimitiveType::I8
				| PrimitiveType::U64
				| PrimitiveType::U32
				| PrimitiveType::U16
				| PrimitiveType::U8 => Err(self.error(
					format!("expected int but got float instead"),
					PIErrorCode::TypecheckExpectedIntGotFloat,
					vec![(
						"expected int but got float".to_owned(),
						float.val.span.clone(),
					)],
				)),
				_ => {
					let expected_bits = primitive_kind_to_bits(&prim);
					if float.bits != expected_bits {
						float.bits = expected_bits;
					}
					Ok(())
				}
			}
		} else {
			Ok(())
		}
	}

	fn check_int_lit(&mut self, int: &mut IntLit) -> PIResult {
		if let Some(Expr::PrimitiveType(prim)) = &self.expecting_ty {
			self.reassign_int_lit_bits(int, prim)?;
		} else if let Some(Expr::Ident(ident)) = &self.expecting_ty {
			let ty = self
				.type_decls
				.get(&ident.to_string())
				.expect("expected type");
			if let Expr::PrimitiveType(prim) = &*ty.type_ {
				self.reassign_int_lit_bits(int, prim)?;
			}
		}
		Ok(())
	}

	fn reassign_int_lit_bits(&self, int: &mut IntLit, prim: &PrimitiveType) -> PIResult {
		let expected_bits = primitive_kind_to_bits(&prim);
		let expected_signed = primitive_kind_to_signedness(&prim);
		if int.bits != expected_bits {
			int.bits = expected_bits;
		}
		if expected_signed == false && *int.negative == true {
			let mut labels = vec![("expected unsigned integer".to_owned(), int.val.span.clone())];
			if expected_signed == false {
				labels.push((format!("unexpected `-`"), int.negative.span.clone()))
			}
			return Err(self.error(
				format!("expected unsigned integer but got signed integer",),
				PIErrorCode::TypecheckUnexpectedSignednessInIntLit,
				labels,
			));
		}
		Ok(())
	}
}

fn primitive_kind_to_bits(prim: &PrimitiveType) -> u8 {
	match prim {
		PrimitiveType::U64 | PrimitiveType::I64 | PrimitiveType::F64 => 64,
		PrimitiveType::U32 | PrimitiveType::I32 | PrimitiveType::F32 => 32,
		PrimitiveType::U16 | PrimitiveType::I16 => 16,
		PrimitiveType::U8 | PrimitiveType::I8 => 8,
		_ => 32,
	}
}

fn primitive_kind_to_signedness(prim: &PrimitiveType) -> bool {
	match prim {
		PrimitiveType::U64 | PrimitiveType::U32 | PrimitiveType::U16 | PrimitiveType::U8 => false,
		_ => true,
	}
}
