use flux_syntax::ast::{CallExpr, FloatExpr, IntExpr, IntrinsicExpr, PathExpr};
use flux_typesystem::r#type::{ConcreteKind, TypeId};
use itertools::Itertools;

use super::*;

type ExprResult = Result<(Expr, TypeId), LowerError>;

impl<'a> LoweringCtx<'a> {
	pub(super) fn lower_expr(
		&mut self,
		expr: Option<ast::Expr>,
	) -> Result<(Idx<Spanned<Expr>>, TypeId), LowerError> {
		let expr = if let Some(expr) = expr {
			expr
		} else {
			todo!("{:#?}", expr)
		};
		let range = expr.range();
		let idx = if let ast::Expr::ParenExpr(paren_e) = expr {
			let e = self.lower_expr(paren_e.expr())?;
			self.exprs[e.0].span.range = TextRange::from(paren_e.range()); // update range to include the parens
			e
		} else {
			let (expr, ty_info) = match expr {
				ast::Expr::BinExpr(bin_expr) => self.lower_binary(bin_expr)?,
				ast::Expr::IntExpr(int_expr) => self.lower_int(int_expr)?,
				ast::Expr::FloatExpr(float_expr) => self.lower_float(float_expr)?,
				ast::Expr::PrefixExpr(prefix_expr) => self.lower_prefix(prefix_expr)?,
				ast::Expr::CallExpr(call_expr) => self.lower_call(call_expr)?,
				ast::Expr::PathExpr(path_expr) => self.lower_path(path_expr)?,
				ast::Expr::StructExpr(struct_expr) => self.lower_struct_expr(struct_expr)?,
				ast::Expr::IfExpr(if_expr) => self.lower_if_expr(if_expr)?,
				ast::Expr::BlockExpr(block_expr) => self.lower_block_expr(block_expr)?,
				ast::Expr::TupleExpr(tuple_expr) => self.lower_tuple_expr(tuple_expr)?,
				ast::Expr::IntrinsicExpr(intrinsic_expr) => self.lower_intrinsic_expr(intrinsic_expr)?,
				ast::Expr::AddressExpr(address_expr) => self.lower_address_expr(address_expr)?,
				ast::Expr::IndexMemoryExpr(idx_mem_expr) => self.lower_idx_mem_expr(idx_mem_expr)?,
				_ => todo!(),
			};
			(
				self
					.exprs
					.alloc(Spanned::new(expr, Span::new(range, self.file_id.clone()))),
				ty_info,
			)
		};
		Ok(idx)
	}

	fn lower_binary(&mut self, bin_expr: ast::BinExpr) -> ExprResult {
		let op = match bin_expr.op().unwrap().kind() {
			SyntaxKind::Plus => BinaryOp::Add,
			SyntaxKind::Minus => BinaryOp::Sub,
			SyntaxKind::Star => BinaryOp::Mul,
			SyntaxKind::Slash => BinaryOp::Div,
			SyntaxKind::CmpEq => BinaryOp::CmpEq,
			SyntaxKind::DoubleColon => BinaryOp::DoubleColon,
			SyntaxKind::Period => return self.lower_binary_access(bin_expr),
			SyntaxKind::Eq => return self.lower_binary_assign(bin_expr),
			SyntaxKind::CmpNeq => BinaryOp::CmpNeq,
			_ => unreachable!(),
		};

		let binary_ty = self.tchecker.tenv.insert(Spanned::new(
			TypeKind::Unknown,
			Span::new(bin_expr.range(), self.file_id.clone()),
		));
		let (lhs, lhs_id) = self.lower_expr(bin_expr.lhs())?;
		let lhs_ty = self.tchecker.tenv.get_type(lhs_id);
		let lhs_id = self.tchecker.tenv.insert(lhs_ty.clone());
		let (rhs, rhs_id) = self.lower_expr(bin_expr.rhs())?;
		let rhs_ty = self.tchecker.tenv.get_type(rhs_id);
		let rhs_id = self.tchecker.tenv.insert(rhs_ty.clone());
		self
			.tchecker
			.unify(
				lhs_id,
				rhs_id,
				Span::combine(&self.exprs[lhs].span, &self.exprs[rhs].span),
			)
			.map_err(LowerError::TypeError)?;
		self
			.tchecker
			.unify(
				binary_ty,
				lhs_id,
				Span::new(bin_expr.range(), self.file_id.clone()),
			)
			.map_err(LowerError::TypeError)?;
		Ok((Expr::Binary(Binary { op, lhs, rhs }), binary_ty))
	}

	fn get_field_access_type(&mut self, access: &hir::Access) -> Result<TypeId, LowerError> {
		let struct_expr = match self.exprs[access.lhs].inner.clone() {
			Expr::Path(path) => {
				let id = self
					.tchecker
					.tenv
					.get_path_id(&path)
					.map_err(LowerError::TypeError)?;
				self.to_ty(&self.tchecker.tenv.get_type(id).clone())
			}
			Expr::Access(access) => {
				let id = self.get_field_access_type(&access)?;
				self.to_ty(&self.tchecker.tenv.get_type(id).clone())
			}
			_ => unreachable!("{:#?}", self.exprs[access.lhs].inner),
		};

		let struct_name = match &struct_expr.inner {
			Type::Ident((name, _)) => name,
			_ => unreachable!(),
		};
		let struct_type_decl = self.type_decls.get(struct_name).unwrap();
		let struct_ty = &struct_type_decl.ty.inner;
		let struct_ty = match struct_ty {
			Type::Struct(struct_ty) => struct_ty,
			_ => unreachable!(),
		};

		let ty_decl = struct_ty
			.fields
			.iter()
			.find(|(name, _)| **name == access.field.inner);
		let ty_id = match ty_decl {
			Some((_, field_ty)) => {
				let ty = self.to_ty_kind(&field_ty.ty);
				self.tchecker.tenv.insert(ty)
			}
			None => match self.method_signatures.get(&self.fmt_ty(&struct_expr.inner)) {
				Some(methods) => match methods.get(&access.field.inner) {
					Some(signature) => *signature,
					_ => todo!(),
				},
				_ => todo!(),
			},
		};

		Ok(ty_id)
	}

	fn lower_binary_access(&mut self, access_expr: ast::BinExpr) -> ExprResult {
		let (lhs, _) = self.lower_expr(access_expr.lhs())?;

		let field = match access_expr.rhs().unwrap() {
			ast::Expr::PathExpr(path) => {
				let names = path.names().collect::<Vec<_>>();
				let name = names.first().unwrap();
				Spanned::new(
					SmolStr::from(name.text()),
					Span::new(name.text_range(), self.file_id.clone()),
				)
			}
			_ => unreachable!(),
		};
		let access = Access { lhs, field };

		let access_ty_id = self.get_field_access_type(&access)?;

		Ok((Expr::Access(access), access_ty_id))
	}

	fn lower_binary_assign(&mut self, assign_expr: ast::BinExpr) -> ExprResult {
		let (lhs, lhs_id) = self.lower_expr(assign_expr.lhs())?;
		let (rhs, rhs_id) = self.lower_expr(assign_expr.rhs())?;

		self
			.tchecker
			.unify(lhs_id, rhs_id, self.span(&assign_expr))
			.map_err(LowerError::TypeError)?;

		Ok((
			Expr::Binary(Binary {
				lhs,
				op: BinaryOp::Assign,
				rhs,
			}),
			lhs_id,
		))
	}

	fn lower_int(&mut self, int_expr: IntExpr) -> ExprResult {
		if int_expr.tok().is_none() {
			todo!()
			// return Err(FluxError::build(
			// 	format!("could not lower int expression: missing value"),
			// 	self.default_span(),
			// 	FluxErrorCode::CouldNotLowerNode,
			// 	(
			// 		format!("could not lower int expression: missing value"),
			// 		self.default_span(),
			// 	),
			// ));
		}
		let int = int_expr.tok().unwrap().text().replace("_", "");
		let radix: u32 = if let Some(prefix) = int.get(0..1) {
			match prefix {
				"0x" => 16,
				"0b" => 2,
				_ => 10,
			}
		} else {
			10
		};

		let n = u64::from_str_radix(int.as_str(), radix);
		if let Some(_) = n.as_ref().err() {
			todo!()
		// return Err(
		// 	FluxError::build(
		// 		format!("could not lower int expression: {}", err.to_string()),
		// 		self.span(&int_expr),
		// 		FluxErrorCode::HirParseIntString,
		// 		(
		// 			format!("could not lower int expression: {}", err.to_string()),
		// 			self.span(&int_expr),
		// 		),
		// 	)
		// 	.with_label(
		// 		format!("could not lower int expression"),
		// 		self.span(&int_expr),
		// 	),
		// );
		} else {
			return Ok((
				Expr::Int(Int {
					n: n.unwrap(),
					ty: Type::UInt(32),
				}),
				self.tchecker.tenv.insert(Spanned::new(
					TypeKind::Int(None),
					Span::new(int_expr.range(), self.file_id.clone()),
				)),
			));
		}
	}

	fn lower_float(&mut self, float_expr: FloatExpr) -> ExprResult {
		if float_expr.tok().is_none() {
			todo!()
			// return Err(FluxError::build(
			// 	format!("could not lower float expression: missing value"),
			// 	self.default_span(),
			// 	FluxErrorCode::CouldNotLowerNode,
			// 	(
			// 		format!("could not lower float expression: missing value"),
			// 		self.default_span(),
			// 	),
			// ));
		}

		let n = float_expr.tok().unwrap().text().parse::<f64>();
		if let Some(_) = n.as_ref().err() {
			todo!()
		// return Err(
		// 	FluxError::build(
		// 		format!("could not lower float expression: {}", err.to_string()),
		// 		self.span(&float_expr),
		// 		FluxErrorCode::CouldNotLowerNode,
		// 		(
		// 			format!("could not lower float expression: {}", err.to_string()),
		// 			self.span(&float_expr),
		// 		),
		// 	)
		// 	.with_label(
		// 		format!("could not lower float expression"),
		// 		self.span(&float_expr),
		// 	),
		// );
		} else {
			return Ok((
				Expr::Float(Float {
					n: n.unwrap(),
					ty: Type::F32,
				}),
				self.tchecker.tenv.insert(Spanned::new(
					TypeKind::Float(None),
					Span::new(float_expr.range(), self.file_id.clone()),
				)),
			));
		}
	}

	fn lower_prefix(&mut self, prefix_expr: ast::PrefixExpr) -> ExprResult {
		let op = if let Some(op) = prefix_expr.op() {
			match op.kind() {
				SyntaxKind::Minus => PrefixOp::Neg,
				_ => unreachable!(),
			}
		} else {
			todo!()
			// return Err(FluxError::build(
			// 	format!("could not lower prefix expression: missing operator"),
			// 	self.default_span(),
			// 	FluxErrorCode::CouldNotLowerNode,
			// 	(
			// 		format!("could not lower prefix expression: missing operator"),
			// 		self.default_span(),
			// 	),
			// ));
		};

		let (expr, expr_id) = self.lower_expr(prefix_expr.expr())?;
		let prefix_id = self.tchecker.tenv.insert(Spanned::new(
			TypeKind::Unknown,
			Span::new(prefix_expr.range(), self.file_id.clone()),
		));
		self
			.tchecker
			.unify(prefix_id, expr_id, self.exprs[expr].span.clone())
			.map_err(LowerError::TypeError)?;
		Ok((Expr::Prefix { op, expr }, prefix_id))
	}

	fn lower_call(&mut self, call_expr: CallExpr) -> ExprResult {
		if let ast::Expr::IntrinsicExpr(intrinsic) = call_expr.callee().unwrap() {
			let intrinsic = intrinsic.name().unwrap();
			let intrinsic = Spanned::new(
				intrinsic.text().into(),
				Span::new(intrinsic.text_range(), self.file_id.clone()),
			);
			return self.lower_intrinsic_call(intrinsic, call_expr.args());
		}

		let (callee, _) = self.lower_expr(call_expr.callee())?;

		let mut args = vec![];
		let mut arg_ids = vec![];
		for arg in call_expr.args() {
			let (arg, arg_id) = self.lower_expr(Some(arg))?;
			args.push(arg);
			arg_ids.push(arg_id);
		}

		let args_range = match (call_expr.lparen(), call_expr.rparen()) {
			(Some(lparen), Some(rparen)) => {
				TextRange::new(lparen.text_range().start(), rparen.text_range().end())
			}
			(Some(lparen), _) => {
				if !args.is_empty() {
					TextRange::new(
						lparen.text_range().start(),
						self.exprs[args.last().unwrap().clone()].span.range.end(),
					)
				} else {
					TextRange::new(lparen.text_range().start(), lparen.text_range().end())
				}
			}
			(_, Some(rparen)) => {
				if !args.is_empty() {
					TextRange::new(
						self.exprs[args[0]].span.range.end(),
						rparen.text_range().end(),
					)
				} else {
					TextRange::new(
						self.exprs[callee].span.range.end(),
						rparen.text_range().end(),
					)
				}
			}
			_ => call_expr.range(),
		};
		let args = Spanned::new(args, Span::new(args_range, self.file_id.clone()));

		Ok((
			Expr::Call(Call { callee, args }),
			self
				.tchecker
				.tenv
				.insert(Spanned::new(TypeKind::Unknown, self.span(&call_expr))),
		))
	}

	fn lower_intrinsic_call(
		&mut self,
		intrinsic: Spanned<SmolStr>,
		args: impl Iterator<Item = ast::Expr>,
	) -> ExprResult {
		let instrinsic_name = intrinsic.inner.split(".").last().unwrap();
		match instrinsic_name {
			"malloc" => self.lower_malloc_intrinsic_expr(args, intrinsic.span),
			"free" => self.lower_free_intrinsic_expr(args, intrinsic.span),
			"memcpy" => self.lower_memcpy_intrinsic_expr(args, intrinsic.span),
			_ => return Err(LowerError::UnknownIntrinsic { intrinsic }),
		}
	}

	fn lower_malloc_intrinsic_expr(
		&mut self,
		args: impl Iterator<Item = ast::Expr>,
		span: Span,
	) -> ExprResult {
		let args: Result<Vec<_>, _> = args.map(|arg| self.lower_expr(Some(arg))).collect();
		let args = args?;
		if args.len() != 1 {
			return Err(LowerError::IncorrectNumberOfArgsInCall {});
		}
		let (arg, arg_id) = args.first().unwrap();

		let u64_id = self.tchecker.tenv.insert(Spanned::new(
			TypeKind::Concrete(ConcreteKind::UInt(64)),
			span.clone(),
		));

		self
			.tchecker
			.unify(*arg_id, u64_id, self.exprs[*arg].span.clone())
			.map_err(LowerError::TypeError)?;

		let u8_id = self.tchecker.tenv.insert(Spanned::new(
			TypeKind::Concrete(ConcreteKind::UInt(8)),
			span.clone(),
		));
		let u8_ptr_ty = TypeKind::Concrete(ConcreteKind::Ptr(u8_id));
		let u8_ptr_id = self.tchecker.tenv.insert(Spanned::new(
			u8_ptr_ty,
			Span::combine(&span, &self.exprs[*arg].span),
		));

		let malloc = Intrinsic::Malloc(*arg);
		Ok((Expr::Intrinsic(malloc), u8_ptr_id))
	}

	fn lower_free_intrinsic_expr(
		&mut self,
		args: impl Iterator<Item = ast::Expr>,
		span: Span,
	) -> ExprResult {
		let args: Result<Vec<_>, _> = args.map(|arg| self.lower_expr(Some(arg))).collect();
		let args = args?;
		if args.len() != 1 {
			return Err(LowerError::IncorrectNumberOfArgsInCall {});
		}
		let (arg, arg_id) = args.first().unwrap();

		let u8_id = self.tchecker.tenv.insert(Spanned::new(
			TypeKind::Concrete(ConcreteKind::UInt(8)),
			span.clone(),
		));
		let u8_ptr_ty = TypeKind::Concrete(ConcreteKind::Ptr(u8_id));
		let u8_ptr_id = self.tchecker.tenv.insert(Spanned::new(
			u8_ptr_ty,
			Span::combine(&span, &self.exprs[*arg].span),
		));

		self
			.tchecker
			.unify(*arg_id, u8_ptr_id, self.exprs[*arg].span.clone())
			.map_err(LowerError::TypeError)?;

		let unit_id = self.tchecker.tenv.insert(Spanned::new(
			TypeKind::Concrete(ConcreteKind::Tuple(vec![])),
			span.clone(),
		));

		let free = Intrinsic::Free(*arg);
		Ok((Expr::Intrinsic(free), unit_id))
	}

	fn lower_memcpy_intrinsic_expr(
		&mut self,
		args: impl Iterator<Item = ast::Expr>,
		span: Span,
	) -> ExprResult {
		let args: Result<Vec<_>, _> = args.map(|arg| self.lower_expr(Some(arg))).collect();
		let args = args?;
		if args.len() != 3 {
			return Err(LowerError::IncorrectNumberOfArgsInCall {});
		}

		let unit_id = self.tchecker.tenv.insert(Spanned::new(
			TypeKind::Concrete(ConcreteKind::Tuple(vec![])),
			span.clone(),
		));

		let param_types = vec![
			TypeKind::Concrete(ConcreteKind::Ptr(unit_id)),
			TypeKind::Concrete(ConcreteKind::Ptr(unit_id)),
			TypeKind::Concrete(ConcreteKind::UInt(64)),
		];

		for (i, (_, arg_ty)) in args.iter().enumerate() {
			let param_ty = self
				.tchecker
				.tenv
				.insert(self.default_spanned(param_types[i].clone()));
			self
				.tchecker
				.unify(*arg_ty, param_ty, span.clone())
				.map_err(LowerError::TypeError)?;
		}

		Ok((
			Expr::Intrinsic(Intrinsic::Memcpy(Memcpy {
				dest: args[0].0,
				src: args[1].0,
				n: args[2].0,
			})),
			unit_id,
		))
	}

	fn lower_intrinsic_expr(&mut self, intrinsic_expr: IntrinsicExpr) -> ExprResult {
		let intrinsic = self.unwrap_ident(
			intrinsic_expr.name(),
			intrinsic_expr.range(),
			format!("intrinsic name"),
		)?;
		match intrinsic.inner.split(".").last().unwrap() {
			"nullptr" => self.lower_nullptr_intrinsic(intrinsic.span),
			_ => return Err(LowerError::UnknownIntrinsic { intrinsic }),
		}
	}

	fn lower_nullptr_intrinsic(&mut self, span: Span) -> ExprResult {
		let unit_id = self.tchecker.tenv.insert(Spanned::new(
			TypeKind::Concrete(ConcreteKind::Tuple(vec![])),
			span.clone(),
		));
		let unit_ptr_id = self.tchecker.tenv.insert(Spanned::new(
			TypeKind::Concrete(ConcreteKind::Ptr(unit_id)),
			span.clone(),
		));
		Ok((Expr::Intrinsic(Intrinsic::Nullptr), unit_ptr_id))
	}

	fn lower_path(&mut self, path_expr: PathExpr) -> ExprResult {
		let mut spanned_path = vec![];
		let mut path = vec![];

		path_expr.names().for_each(|name| {
			spanned_path.push(Spanned::new(
				SmolStr::from(name.text()),
				Span::new(name.text_range(), self.file_id.clone()),
			));
			path.push(SmolStr::from(name.text()));
		});

		let id = self
			.tchecker
			.tenv
			.get_path_id(&spanned_path)
			.map_err(LowerError::TypeError)?;
		Ok((
			Expr::Path(spanned_path),
			self
				.tchecker
				.tenv
				.insert(Spanned::new(TypeKind::Ref(id), self.span(&path_expr))),
		))
	}

	fn lower_struct_expr(&mut self, struct_expr: ast::StructExpr) -> ExprResult {
		let struct_name = self.unwrap_path(
			struct_expr.name(),
			struct_expr.range(),
			format!("struct expression name"),
		)?;
		let struct_name_str = Spanned::new(
			SmolStr::from(struct_name.iter().map(|s| s.inner.clone()).join("::")),
			Spanned::vec_span(&struct_name).unwrap(),
		);
		let type_decl = match self.type_decls.get(&struct_name_str.inner) {
			Some(ty_decl) => ty_decl.clone(),
			None => {
				return Err(LowerError::UnknownStruct {
					name: Spanned::new(
						struct_name_str.inner,
						Spanned::vec_span(&struct_name).unwrap(),
					),
				})
			}
		};
		let struct_type = match &type_decl.ty.inner {
			Type::Struct(struct_ty) => struct_ty,
			_ => unreachable!(),
		};

		let num_generics = struct_type.generics.len();
		let mut type_params = Vec::with_capacity(num_generics); // these are the types that will be retured associated with this expression's type
		let type_params_uninit = type_params.spare_capacity_mut();
		let mut fields = vec![];
		let mut initialized_fields = HashSet::new();
		for field in struct_expr.fields() {
			let name = self.unwrap_ident(field.name(), field.range(), format!("struct field name"))?;
			let (val, val_id) = self.lower_expr(field.value())?;

			if let Some(field) = struct_type.fields.get(&name.inner) {
				let field_ty_kind = self.to_ty_kind(&field.ty);
				let field_ty_id = self.tchecker.tenv.insert(field_ty_kind.clone());

				self
					.tchecker
					.unify(val_id, field_ty_id, self.exprs[val].span.clone())
					.map_err(LowerError::TypeError)?;

				if let TypeKind::Generic((name, _)) = self.tchecker.tenv.inner_type(&field_ty_kind.inner) {
					let mut id = val_id;
					while let TypeKind::Concrete(ConcreteKind::Ptr(new_id)) =
						self.tchecker.tenv.get_type(id).inner
					{
						id = new_id;
					}
					let index = struct_type.generics.get_index_of(&name).unwrap();
					type_params_uninit[index].write(id);
				}

				initialized_fields.insert(name.inner.clone());
			} else {
				return Err(LowerError::NoSuchStructField {
					struct_name: struct_name_str,
					field_name: name,
				});
			}
			fields.push((
				Spanned::new(
					SmolStr::from(name.inner.clone()),
					Span::new(name.span.range, self.file_id.clone()),
				),
				val,
			));
		}

		unsafe {
			type_params.set_len(num_generics);
		}

		let uninitialized_fields: Vec<SmolStr> = struct_type
			.fields
			.iter()
			.filter_map(|(field_name, _)| match initialized_fields.get(field_name) {
				Some(_) => None,
				None => Some(field_name.clone()),
			})
			.collect();

		let fields_range = match (struct_expr.lparen(), struct_expr.rparen()) {
			(Some(lparen), Some(rparen)) => {
				TextRange::new(lparen.text_range().start(), rparen.text_range().end())
			}
			_ => struct_expr.range(),
		};
		let fields = Spanned::new(fields, Span::new(fields_range, self.file_id.clone()));

		if uninitialized_fields.len() > 0 {
			return Err(LowerError::UninitializedFieldsInStructExpr {
				struct_name: struct_name_str.inner,
				struct_type: self.fmt_ty(&type_decl.ty),
				uninitialized_fields,
				span: fields.span.clone(),
			});
		}

		let id = self.tchecker.tenv.insert(Spanned::new(
			TypeKind::Concrete(ConcreteKind::Ident((struct_name_str.inner, type_params))),
			Span::new(struct_expr.range(), self.file_id.clone()),
		));
		let expr = Expr::Struct(Struct {
			name: struct_name,
			fields,
		});
		Ok((expr, id))
	}

	fn lower_if_expr(&mut self, if_expr: ast::IfExpr) -> ExprResult {
		let (condition, _) = self.lower_expr(if_expr.condition())?; // TODO: verify condition_id is a boolean
		let (then, then_id) = if let Some(then) = if_expr.then() {
			let range = then.range();
			let (block, block_id) = self.lower_block_expr(then)?;
			(
				Spanned::new(block, Span::new(range, self.file_id.clone())),
				block_id,
			)
		} else {
			todo!()
			// return Err(FluxError::build(
			// 	format!("could not lower if expression: missing then block"),
			// 	Span::new(TextRange::default(), self.file_id.clone()),
			// 	FluxErrorCode::CouldNotLowerNode,
			// 	(
			// 		format!("could not lower if expression: missing then block"),
			// 		Span::new(TextRange::default(), self.file_id.clone()),
			// 	),
			// ));
		};
		let else_ = if if_expr.else_().is_some() {
			let (else_, else_id) = self.lower_expr(if_expr.else_())?;
			self
				.tchecker
				.unify(
					then_id,
					else_id,
					Span::combine(&then.span, &self.exprs[else_].span),
				)
				.map_err(LowerError::TypeError)?;
			else_
		} else {
			let else_id = self.tchecker.tenv.insert(Spanned::new(
				TypeKind::Concrete(ConcreteKind::Tuple(vec![])),
				self.span(&if_expr),
			));
			self
				.tchecker
				.unify(
					then_id,
					else_id,
					self.span(&if_expr), // Span::combine(&then.span, &self.exprs[else_].span),
				)
				.map_err(LowerError::TypeError)?;
			self
				.exprs
				.alloc(Spanned::new(Expr::Missing, self.span(&if_expr)))
		};
		// let (else_, else_id) = self.lower_expr(if_expr.else_()).map_err(LowerError::TypeError)?;
		// self.tchecker.unify(
		// 	then_id,
		// 	else_id,
		// 	Span::combine(&then.span, &self.exprs[else_].span),
		// ).map_err(LowerError::TypeError)?;
		let if_id = self.tchecker.tenv.insert(Spanned::new(
			TypeKind::Unknown,
			Span::new(if_expr.range(), self.file_id.clone()),
		));
		self
			.tchecker
			.unify(
				if_id,
				then_id,
				Span::new(if_expr.range(), self.file_id.clone()),
			)
			.map_err(LowerError::TypeError)?;
		Ok((
			Expr::If(If::new(condition, self.exprs.alloc(then), else_)),
			if_id,
		))
	}

	fn lower_block_expr(&mut self, block_expr: ast::BlockExpr) -> ExprResult {
		let mut block = vec![];
		let mut block_ids = vec![];
		let stmts = block_expr.stmts();
		let mut terminal_stmt: Option<(Spanned<Stmt>, usize)> = None;
		for stmt in stmts {
			if let Some((terminal_stmt, _)) = terminal_stmt {
				return Err(LowerError::StmtAfterTerminalStmt {
					terminal_stmt: terminal_stmt.span,
					stmt: Span::new(stmt.range(), self.file_id.clone()),
				});
				// return Err(
				// 		FluxError::build(
				// 			format!("cannot put statements after block value statement"),
				// 			self.span(&stmt),
				// 			FluxErrorCode::StmtAfterBlockValStmt,
				// 			(
				// 				format!("cannot put statements after block value statement"),
				// 				self.span(&stmt),
				// 			),
				// 		)
				// 		.with_label(format!("block value statement"), s.span),
				// 	);
			}
			let ((stmt, stmt_id), has_semi) = self.lower_stmt(stmt)?;
			block.push(stmt.clone());
			block_ids.push(stmt_id);
			if !has_semi {
				terminal_stmt = Some((stmt, stmt_id));
			}
		}
		let type_id = if let Some((_, id)) = terminal_stmt {
			id
		} else {
			self.tchecker.tenv.insert(Spanned::new(
				TypeKind::Concrete(ConcreteKind::Tuple(vec![])),
				self.span(&block_expr),
			))
		};
		Ok((Expr::Block(Block(block)), type_id))
	}

	fn lower_tuple_expr(&mut self, tuple_expr: ast::TupleExpr) -> ExprResult {
		let mut values = vec![];
		let mut value_types = vec![];
		for e in tuple_expr.values() {
			let (idx, ty_id) = self.lower_expr(Some(e))?;
			values.push(idx);
			value_types.push(ty_id);
		}
		let type_id = self.tchecker.tenv.insert(Spanned::new(
			TypeKind::Concrete(ConcreteKind::Tuple(value_types)),
			self.span(&tuple_expr),
		));
		Ok((Expr::Tuple(Tuple(values)), type_id))
	}

	fn lower_address_expr(&mut self, address_expr: ast::AddressExpr) -> ExprResult {
		let (expr, expr_id) = self.lower_expr(Some(address_expr.expr().unwrap()))?;
		let ty = self.tchecker.tenv.insert(Spanned::new(
			TypeKind::Concrete(ConcreteKind::Ptr(expr_id)),
			self.span(&address_expr),
		));
		Ok((Expr::Address(Address(expr)), ty))
	}

	fn lower_idx_mem_expr(&mut self, idx_mem_expr: ast::IndexMemoryExpr) -> ExprResult {
		let (val, val_id) = self.lower_expr(idx_mem_expr.expr())?;
		let (idx, _) = self.lower_expr(idx_mem_expr.expr())?;

		let ty = match &self.tchecker.tenv.get_type(val_id).inner {
			TypeKind::Concrete(ConcreteKind::Ptr(id)) => *id,
			_ => {
				return Err(LowerError::IndexMemAccessOnNonPtrExpr {
					span: self.span(&idx_mem_expr),
					ty: self
						.tchecker
						.tenv
						.fmt_ty(&self.tchecker.tenv.get_type(val_id)),
				})
			}
		};
		Ok((Expr::IdxMem(IdxMem { val, idx }), ty))
	}
}
