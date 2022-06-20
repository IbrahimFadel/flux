use flux_syntax::ast::{CallExpr, FloatExpr, IntExpr, PathExpr};
use flux_typesystem::{TypeId, TypeKind};

use super::*;

type ExprResult = Result<(Expr, TypeId), FluxError>;

impl LoweringCtx {
	pub(super) fn lower_expr(
		&mut self,
		expr: Option<ast::Expr>,
	) -> Result<(Idx<Spanned<Expr>>, TypeId), FluxError> {
		let expr = if let Some(expr) = expr {
			expr
		} else {
			return Err(FluxError::default().with_msg(format!("could not lower expression: missing")));
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
				// ast::Expr::StructExpr(struct_expr) => self.lower_struct_expr(struct_expr),
				ast::Expr::IfExpr(if_expr) => self.lower_if_expr(if_expr)?,
				ast::Expr::BlockExpr(block_expr) => self.lower_block_expr(block_expr)?,
				_ => unreachable!(),
			};
			(
				self
					.exprs
					.alloc(Spanned::new(expr, Span::new(range, self.file_id))),
				ty_info,
			)
		};
		Ok(idx)
	}

	fn lower_binary(&mut self, bin_expr: ast::BinExpr) -> ExprResult {
		let op = if let Some(op) = bin_expr.op() {
			match op.kind() {
				SyntaxKind::Plus => BinaryOp::Add,
				SyntaxKind::Minus => BinaryOp::Sub,
				SyntaxKind::Star => BinaryOp::Mul,
				SyntaxKind::Slash => BinaryOp::Div,
				SyntaxKind::CmpEq => BinaryOp::CmpEq,
				SyntaxKind::DoubleColon => BinaryOp::DoubleColon,
				_ => unreachable!(),
			}
		} else {
			return Err(FluxError::default().with_msg(format!(
				"could not lower binary expression: missing operator"
			)));
		};

		let binary_ty = self.tenv.insert(TypeKind::Unknown); // let's figure out what type it has
		let (lhs, lhs_id) = self.lower_expr(bin_expr.lhs())?;
		let lhs_ty = self.tenv.get_type(lhs_id);
		let lhs_id = self.tenv.insert(lhs_ty);
		let (rhs, rhs_id) = self.lower_expr(bin_expr.rhs())?;
		let rhs_ty = self.tenv.get_type(rhs_id);
		let rhs_id = self.tenv.insert(rhs_ty);
		self.tenv.unify(lhs_id, rhs_id)?;
		self.tenv.unify(binary_ty, lhs_id)?; // Now the bin_expr type is dependent on the terms
		Ok((Expr::Binary(Binary { op, lhs, rhs }), binary_ty))
	}

	fn lower_int(&mut self, int_expr: IntExpr) -> ExprResult {
		if int_expr.tok().is_none() {
			return Err(
				FluxError::default().with_msg(format!("could not lower int expression: missing value")),
			);
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
		if let Some(err) = n.as_ref().err() {
			return Err(
				FluxError::default()
					.with_msg(format!(
						"could not lower int expression: {}",
						err.to_string()
					))
					.with_primary(
						format!("could not lower int expression: {}", err.to_string()),
						Some(Span::new(int_expr.range(), self.file_id)),
					)
					.with_code(FluxErrorCode::HirParseIntString)
					.with_label(
						format!("could not lower int expression"),
						Some(Span::new(int_expr.syntax().text_range(), self.file_id)),
					),
			);
		} else {
			return Ok((
				Expr::Int(Int {
					n: n.unwrap(),
					ty: Type::UInt(32),
				}),
				self.tenv.insert(TypeKind::Int(None)),
			));
		}
	}

	fn lower_float(&mut self, float_expr: FloatExpr) -> ExprResult {
		if float_expr.tok().is_none() {
			return Err(
				FluxError::default().with_msg(format!("could not lower float expression: missing value")),
			);
		}

		let n = float_expr.tok().unwrap().text().parse::<f64>();
		if let Some(err) = n.as_ref().err() {
			return Err(
				FluxError::default()
					.with_msg(format!(
						"could not lower float expression: {}",
						err.to_string()
					))
					.with_primary(
						format!("could not lower float expression: {}", err.to_string()),
						Some(Span::new(float_expr.range(), self.file_id)),
					)
					.with_code(FluxErrorCode::HirParseIntString)
					.with_label(
						format!("could not lower float expression"),
						Some(Span::new(float_expr.syntax().text_range(), self.file_id)),
					),
			);
		} else {
			return Ok((
				Expr::Float(Float {
					n: n.unwrap(),
					ty: Type::F32,
				}),
				self.tenv.insert(TypeKind::Float(None)),
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
			return Err(FluxError::default().with_msg(format!(
				"could not lower prefix expression: missing operator"
			)));
		};

		let (expr, expr_id) = self.lower_expr(prefix_expr.expr())?;
		let prefix_id = self.tenv.insert(TypeKind::Unknown);
		self.tenv.unify(prefix_id, expr_id)?;
		Ok((Expr::Prefix { op, expr }, prefix_id))
	}

	fn lower_call(&mut self, call_expr: CallExpr) -> ExprResult {
		let (callee, callee_id) = self.lower_expr(call_expr.callee())?;
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
		let args = Spanned::new(args, Span::new(args_range, self.file_id));

		// TODO: Fix
		// let i = self.tenv.insert(TypeKind::Unknown);
		// let o = self.tenv.insert(TypeKind::Unknown); // TODO: this shouldn't be unknown, we should find the signature
		// let call_ty = TypeInfo::Func(Box::new(x), ())

		Ok((
			Expr::Call(Call { callee, args }),
			self.tenv.insert(TypeKind::Concrete(Type::Func(
				Box::new(Type::Unknown),
				Box::new(Type::Unknown),
			))),
		))
	}

	fn lower_path(&mut self, path_expr: PathExpr) -> ExprResult {
		let mut spanned_path = vec![];
		let mut path = vec![];

		path_expr.names().for_each(|name| {
			spanned_path.push(Spanned::new(
				SmolStr::from(name.text()),
				Span::new(name.text_range(), self.file_id),
			));
			path.push(SmolStr::from(name.text()));
		});

		Ok((
			Expr::Path(spanned_path),
			self
				.tenv
				.insert(TypeKind::Ref(self.tenv.get_path_id(&path))),
		))
	}

	// fn lower_struct_expr(&mut self, struct_expr: ast::StructExpr) -> Expr {
	// 	let name = if let Some(name) = struct_expr.name() {
	// 		let syntax = &name.names().collect::<Vec<_>>()[0];
	// 		Spanned::new(
	// 			SmolStr::from(syntax.text()),
	// 			Span::new(syntax.text_range(), self.file_id),
	// 		)
	// 	} else {
	// 		self.errors.push(
	// 			FluxError::default().with_msg(format!("could not lower struct expressions: missing name")),
	// 		);
	// 		return Expr::Missing;
	// 	};
	// 	let mut fields = vec![];
	// 	for field in struct_expr.fields() {
	// 		if let Some(name) = field.name() {
	// 			let syntax = &name.names().collect::<Vec<_>>()[0];
	// 			fields.push((
	// 				Spanned::new(
	// 					SmolStr::from(syntax.text()),
	// 					Span::new(syntax.text_range(), self.file_id),
	// 				),
	// 				self.lower_expr(field.value()),
	// 			));
	// 		} else {
	// 			self.errors.push(FluxError::default().with_msg(format!(
	// 				"could not lower struct expressions: field missing name"
	// 			)));
	// 			return Expr::Missing;
	// 		}
	// 	}
	// 	let fields_range = match (struct_expr.lparen(), struct_expr.rparen()) {
	// 		(Some(lparen), Some(rparen)) => {
	// 			TextRange::new(lparen.text_range().start(), rparen.text_range().end())
	// 		}
	// 		_ => struct_expr.range(),
	// 	};
	// 	let fields = Spanned::new(fields, Span::new(fields_range, self.file_id));

	// 	Expr::Struct(Struct { name, fields })
	// }

	fn lower_if_expr(&mut self, if_expr: ast::IfExpr) -> ExprResult {
		let (condition, condition_id) = self.lower_expr(if_expr.condition())?; // TODO: verify condition_id is a boolean?
		let (then, then_id) = if let Some(then) = if_expr.then() {
			let range = then.range();
			let (block, block_id) = self.lower_block_expr(then)?;
			(
				Spanned::new(block, Span::new(range, self.file_id)),
				block_id,
			)
		} else {
			return Err(
				FluxError::default().with_msg(format!("could not lower if expression: missing then block")),
			);
		};
		let (else_, else_id) = self.lower_expr(if_expr.else_())?;
		self.tenv.unify(then_id, else_id)?;
		let if_id = self.tenv.insert(TypeKind::Unknown);
		self.tenv.unify(if_id, then_id)?;
		Ok((
			Expr::If(If::new(condition, self.exprs.alloc(then), else_)),
			if_id,
		))
	}

	fn lower_block_expr(&mut self, block_expr: ast::BlockExpr) -> ExprResult {
		let mut block = vec![];
		let mut block_ids = vec![];
		let stmts = block_expr.stmts();
		for stmt in stmts {
			let (stmt, stmt_id) = self.lower_stmt(stmt)?;
			block.push(stmt);
			block_ids.push(stmt_id);
		}
		let type_id = if let Some(id) = block_ids.last() {
			*id
		} else {
			self.tenv.insert(TypeKind::Concrete(Type::Unit))
		};
		Ok((Expr::Block(Block(block)), type_id))
	}
}
