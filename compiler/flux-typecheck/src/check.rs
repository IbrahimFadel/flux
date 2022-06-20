use flux_hir::FnDecl;

use crate::infer::infer_expr;

use super::*;

#[derive(Debug)]
pub(super) struct TypeCheck<'a> {
	typeenv: &'a mut TypeEnv<'a>,
	exprs: &'a mut Arena<Spanned<Expr>>,
}

impl<'a> TypeCheck<'a> {
	pub fn new(typeenv: &'a mut TypeEnv<'a>, exprs: &'a mut Arena<Spanned<Expr>>) -> Self {
		Self { typeenv, exprs }
	}

	pub fn fn_decl(&'a mut self, f: &'a FnDecl) -> FluxResult<()> {
		infer_expr(self.typeenv, self.exprs, f.block)?;
		Ok(())
	}
}
