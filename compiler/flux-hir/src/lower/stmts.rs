use flux_typesystem::TypeId;

use super::*;

type StmtResult = Result<((Spanned<Stmt>, TypeId), bool), FluxError>;

impl<'a> LoweringCtx<'a> {
	pub(super) fn lower_stmt(&mut self, ast: ast::Stmt) -> StmtResult {
		match &ast {
			ast::Stmt::VarDecl(ast) => self.lower_var_decl(ast),
			ast::Stmt::ExprStmt(ast) => {
				let (expr, expr_id) = self.lower_expr(ast.expr())?;
				Ok((
					(
						Spanned::new(
							Stmt::Expr(expr),
							Span::new(ast.range(), self.file_id.clone()),
						),
						expr_id,
					),
					ast.semicolon().is_some(),
				))
			}
			ast::Stmt::ReturnStmt(ast) => self.lower_return_stmt(ast),
		}
	}

	fn lower_var_decl(&mut self, var_decl: &ast::VarDecl) -> StmtResult {
		if let Some(name) = var_decl.name() {
			let var_ty = self.lower_type(var_decl.ty())?;
			let var_ty_id = self.tchecker.tenv.insert(var_ty.clone());
			let (expr, expr_id) = self.lower_expr(var_decl.value())?;
			self.tchecker.unify(
				var_ty_id,
				expr_id,
				Span::combine(&var_ty.span, &self.exprs[expr].span),
			)?;
			self
				.tchecker
				.tenv
				.set_path_id(&[name.text().into()], var_ty_id);
			Ok((
				(
					Spanned::new(
						Stmt::VarDecl(VarDecl {
							ty: Spanned::new(Type::Unknown, var_ty.span.clone()),
							name: name.text().into(),
							value: expr,
						}),
						Span::new(var_decl.range(), self.file_id.clone()),
					),
					self.tchecker.tenv.insert(Spanned::new(
						Type::Tuple(vec![]),
						Span::new(var_decl.range(), self.file_id.clone()),
					)),
				),
				true,
			))
		} else {
			Err(FluxError::build(
				format!("could not lower variable declaration: missing name"),
				self.default_span(),
				FluxErrorCode::CouldNotLowerNode,
				(
					format!("could not lower variable declaration: missing name"),
					self.default_span(),
				),
			))
		}
	}

	fn lower_return_stmt(&mut self, return_stmt: &ast::ReturnStmt) -> StmtResult {
		let (value, value_id) = self.lower_expr(return_stmt.value())?;
		self.tchecker.unify(
			self.tchecker.tenv.return_type_id,
			value_id,
			self.exprs[value].span.clone(),
		)?;
		Ok((
			(
				Spanned::new(
					Stmt::Return(Return { value }),
					Span::new(return_stmt.range(), self.file_id.clone()),
				),
				self.tchecker.tenv.insert(Spanned::new(
					Type::Tuple(vec![]),
					Span::new(return_stmt.range(), self.file_id.clone()),
				)),
			),
			true,
		))
	}
}
