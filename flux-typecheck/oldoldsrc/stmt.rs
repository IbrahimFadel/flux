// use flux_ast::{Return, VarDecl};

// use super::*;

// impl<'a> TypecheckCtx<'a> {
// 	pub fn check_stmt(&mut self, stmt: &'a mut Stmt) -> Option<FluxError> {
// 		match stmt {
// 			Stmt::VarDecl(var) => self.check_var(var),
// 			Stmt::Return(ret) => self.check_ret(ret),
// 			Stmt::ExprStmt(expr) => self.check_expr(expr),
// 			_ => None,
// 		}
// 	}

// 	fn check_ret(&mut self, ret: &'a mut Return) -> Option<FluxError> {
// 		if let Some(x) = &mut ret.val {
// 			if let Some(err) = self.check_expr(x) {
// 				return Some(err);
// 			}
// 		}
// 		return None;
// 	}

// 	fn check_var(&mut self, var: &'a mut VarDecl) -> Option<FluxError> {
// 		self.expecting_ty = Some(&var.type_);
// 		for name in &var.names {
// 			self.var_types.insert(name.to_string(), &var.type_);
// 		}

// 		for val in &mut var.values {
// 			if let Some(err) = self.check_expr(val) {
// 				return Some(err);
// 			}
// 		}

// 		return None;
// 	}
// }
