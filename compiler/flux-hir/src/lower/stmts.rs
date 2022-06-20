use flux_typesystem::TypeId;

use super::*;

type StmtResult = Result<(Spanned<Stmt>, TypeId), FluxError>;

impl LoweringCtx {
	pub(super) fn lower_stmt(&mut self, ast: ast::Stmt) -> StmtResult {
		match ast {
			ast::Stmt::VarDecl(ref ast) => self.lower_var_decl(ast),
			ast::Stmt::ExprStmt(ref ast) => {
				let (expr, expr_id) = self.lower_expr(ast.expr())?;
				Ok((
					Spanned::new(Stmt::Expr(expr), Span::new(ast.range(), self.file_id)),
					expr_id,
				))
			}
			ast::Stmt::ReturnStmt(ref ast) => self.lower_return_stmt(ast),
		}
	}

	fn lower_var_decl(&mut self, var_decl: &ast::VarDecl) -> StmtResult {
		if let Some(name) = var_decl.name() {
			let (ty, var_ty_id) = self.lower_type(var_decl.ty())?;
			let (expr, expr_id) = self.lower_expr(var_decl.value())?;
			self.tenv.unify(var_ty_id, expr_id)?;
			self.tenv.set_path_id(&[name.text().into()], var_ty_id);
			Ok((
				Spanned::new(
					Stmt::VarDecl(VarDecl {
						ty: Spanned::new(Type::Unknown, ty.span.clone()),
						name: name.text().into(),
						value: expr,
					}),
					Span::new(var_decl.range(), self.file_id),
				),
				self.tenv.insert(TypeKind::Concrete(Type::Unit)),
			))
		} else {
			Err(FluxError::default().with_msg(format!(
				"could not lower variable declaration: missing name"
			)))
		}
	}

	fn lower_return_stmt(&mut self, ast: &ast::ReturnStmt) -> StmtResult {
		let (value, _) = self.lower_expr(ast.value())?;
		Ok((
			Spanned::new(
				Stmt::Return(Return { value }),
				Span::new(ast.range(), self.file_id),
			),
			self.tenv.insert(TypeKind::Concrete(Type::Unit)),
		))
	}
}
