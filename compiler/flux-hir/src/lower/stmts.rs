use flux_typesystem::r#type::ConcreteKind;
use indexmap::IndexMap;

use super::*;

type StmtResult = Result<((Spanned<Stmt>, TypeId), bool), LowerError>;

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
		let name = self.unwrap_ident(
			var_decl.name(),
			var_decl.range(),
			format!("variable declaration name"),
		)?;
		let var_ty = self.lower_type(var_decl.ty(), &IndexMap::new())?;
		let var_ty_id = self.tchecker.tenv.insert(self.to_ty_kind(&var_ty));
		let (expr, expr_id) = self.lower_expr(var_decl.value())?;

		self
			.tchecker
			.unify(
				var_ty_id,
				expr_id,
				Span::combine(&var_ty.span, &self.exprs[expr].span),
			)
			.map_err(LowerError::TypeError)?;
		self
			.tchecker
			.tenv
			.set_path_id(&[name.inner.clone()], var_ty_id);
		Ok((
			(
				Spanned::new(
					Stmt::VarDecl(VarDecl {
						ty: Spanned::new(Type::Unknown, var_ty.span.clone()),
						name: name,
						value: expr,
					}),
					Span::new(var_decl.range(), self.file_id.clone()),
				),
				self.tchecker.tenv.insert(Spanned::new(
					TypeKind::Concrete(ConcreteKind::Tuple(vec![])),
					Span::new(var_decl.range(), self.file_id.clone()),
				)),
			),
			true,
		))
	}

	fn lower_return_stmt(&mut self, return_stmt: &ast::ReturnStmt) -> StmtResult {
		let (value, value_id) = match return_stmt.value() {
			Some(val) => self.lower_expr(Some(val))?,
			_ => {
				let span = self.span(return_stmt);
				let ty = self.tchecker.tenv.insert(Spanned::new(
					TypeKind::Concrete(ConcreteKind::Tuple(vec![])),
					span.clone(),
				));
				(
					self
						.exprs
						.alloc(Spanned::new(Expr::Tuple(Tuple(vec![])), span)),
					ty,
				)
			}
		};
		self
			.tchecker
			.unify(
				self.tchecker.tenv.return_type_id,
				value_id,
				self.exprs[value].span.clone(),
			)
			.map_err(LowerError::TypeError)?;
		Ok((
			(
				Spanned::new(
					Stmt::Return(Return { value }),
					Span::new(return_stmt.range(), self.file_id.clone()),
				),
				self.tchecker.tenv.insert(Spanned::new(
					TypeKind::Concrete(ConcreteKind::Tuple(vec![])),
					Span::new(return_stmt.range(), self.file_id.clone()),
				)),
			),
			true,
		))
	}
}
