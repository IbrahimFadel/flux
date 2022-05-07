type PIResult = Result<(), PIError>;

pub fn typecheck_ast<'a>(
	file_ast_map: &mut IndexMap<FileId, AST>,
	_: &'a PIErrorReporting,
) -> PIResult {
	// for (id, ast) in file_ast_map.iter_mut() {
	// 	let mut ftx = FnCtx {
	// 		file_id: id.clone(),
	// 		expecting_ty: None,
	// 		type_decls: HashMap::new(),
	// 		var_types: HashMap::new(),
	// 		struct_methods: HashMap::new(),
	// 		struct_implementations: HashMap::new(),
	// 	};

	// 	ftx.check(ast)?;
	// }
	Ok(())
}

// use std::collections::HashMap;

// use indexmap::IndexMap;
// use flux_ast::{ApplyBlock, Expr, FnDecl, Ident, Return, Spanned, Stmt, TypeDecl, VarDecl, AST};
// use flux_error::{filesystem::FileId, PIError, PIErrorCode, PIErrorReporting, Span};

// mod apply;
// mod expr;

// type PIResult = Result<(), PIError>;

// struct FnCtx<'ctx> {
// 	file_id: FileId,
// 	expecting_ty: Option<&'ctx Expr>,
// 	type_decls: HashMap<String, &'ctx TypeDecl>,
// 	var_types: HashMap<String, &'ctx Expr>,
// 	struct_methods: HashMap<String, HashMap<String, &'ctx FnDecl>>,
// 	struct_implementations: HashMap<String, Vec<TypeDecl>>,
// }

// impl<'ctx> FnCtx<'ctx> {
// 	pub fn error(&self, msg: String, code: PIErrorCode, labels: Vec<(String, Span)>) -> PIError {
// 		PIError::new(msg, code, labels)
// 	}

// 	fn get_type_of_var_in_cur_block(&self, name: &Spanned<Ident>) -> Result<&'ctx Expr, PIError> {
// 		let res = self.var_types.get(&name.to_string());
// 		if let Some(ty) = res {
// 			return Ok(ty);
// 		}
// 		Err(self.error(
// 			format!("could not get type of variable `{}`", name.to_string()),
// 			PIErrorCode::TypecheckCouldNotGetTypeOfVar,
// 			vec![(
// 				format!("could not get type of variable `{}`", name.to_string()),
// 				name.span.clone(),
// 			)],
// 		))
// 	}

// 	pub fn check(&mut self, ast: &'ctx mut AST) -> PIResult {
// 		for ty in &ast.types {
// 			self.type_decls.insert(ty.name.to_string(), &ty);
// 		}

// 		for apply in &mut ast.apply_blocks {
// 			self.check_apply(apply)?;
// 		}

// 		for f in &mut ast.functions {
// 			self.check_fn(f)?;
// 		}

// 		Ok(())
// 	}

// 	fn check_fn(&mut self, f: &'ctx mut FnDecl) -> PIResult {
// 		for param in &*f.params {
// 			self.var_types.insert(param.name.to_string(), &param.type_);
// 		}
// 		for stmt in &mut f.block {
// 			self.expecting_ty = Some(&f.ret_ty);
// 			self.check_stmt(stmt)?;
// 		}
// 		self.var_types.clear();
// 		Ok(())
// 	}

// 	fn check_stmt(&mut self, stmt: &'ctx mut Stmt) -> PIResult {
// 		match stmt {
// 			Stmt::VarDecl(var) => self.check_var(var),
// 			Stmt::Return(ret) => self.check_ret(ret),
// 			Stmt::ExprStmt(expr) => self.check_expr(expr),
// 			_ => Ok(()),
// 		}
// 	}

// 	fn check_ret(&mut self, ret: &'ctx mut Return) -> PIResult {
// 		if let Some(x) = &mut ret.val {
// 			self.check_expr(x)?;
// 		}
// 		Ok(())
// 	}

// 	fn check_var(&mut self, var: &'ctx mut VarDecl) -> PIResult {
// 		self.expecting_ty = Some(&var.type_);
// 		for name in &var.names {
// 			self.var_types.insert(name.to_string(), &var.type_);
// 		}

// 		for val in &mut var.values {
// 			self.check_expr(val)?;
// 		}

// 		Ok(())
// 	}
// }

// pub fn typecheck_ast<'a>(
// 	file_ast_map: &mut IndexMap<FileId, AST>,
// 	_: &'a PIErrorReporting,
// ) -> PIResult {
// 	for (id, ast) in file_ast_map.iter_mut() {
// 		let mut ftx = FnCtx {
// 			file_id: id.clone(),
// 			expecting_ty: None,
// 			type_decls: HashMap::new(),
// 			var_types: HashMap::new(),
// 			struct_methods: HashMap::new(),
// 			struct_implementations: HashMap::new(),
// 		};

// 		ftx.check(ast)?;
// 	}
// 	Ok(())
// }

use indexmap::IndexMap;
use flux_ast::AST;
use flux_error::{filesystem::FileId, PIError, PIErrorReporting};
