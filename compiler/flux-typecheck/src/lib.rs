use std::collections::HashMap;

use flux_driver::{FunctionExportTable, FunctionSignature, TypeExportTable};
use flux_error::{FluxError, Span};
use flux_hir::{Call, Expr, FnDecl, HirModule, Stmt, Type, VarDecl};
use flux_syntax::ast::Spanned;
use la_arena::{Arena, Idx};
use smol_str::SmolStr;
use text_size::TextRange;

#[cfg(test)]
mod tests;

type TypeId = usize;

fn generate_function_signature(f: &flux_hir::FnDecl) -> Option<FunctionSignature> {
	if let Some(_) = f.name {
		let mut param_types = vec![];
		for p in &f.params.node {
			param_types.push(p.ty.clone());
		}

		Some(FunctionSignature {
			param_types: Spanned::new(param_types, f.public.span.clone()),
			return_type: f.return_type.clone(),
		})
	} else {
		None
	}
}

#[derive(Debug)]
struct TypeEnv<'a> {
	expr_arena: &'a Arena<Spanned<Expr>>,
	id_counter: usize,
	locals: HashMap<TypeId, Spanned<Type>>,
	local_ids: HashMap<SmolStr, TypeId>,
	signatures: &'a HashMap<Vec<SmolStr>, FunctionSignature>,
	module_path: &'a [SmolStr],
	use_paths: &'a [Vec<SmolStr>],
}

impl<'a> TypeEnv<'a> {
	pub fn insert(&mut self, info: Spanned<Type>) -> TypeId {
		let id = self.id_counter;
		self.id_counter += 1;
		self.locals.insert(id, info);
		id
	}

	pub fn infer_function(&mut self, f: &FnDecl) -> Result<(), FluxError> {
		for stmt in &f.block {
			if let Some(stmt) = stmt {
				self.infer_stmt(stmt)?;
			}
		}
		Ok(())
	}

	fn infer_stmt(&mut self, stmt: &Stmt) -> Result<(), FluxError> {
		match stmt {
			Stmt::VarDecl(var) => self.infer_var_decl(var),
			Stmt::Expr(expr) => {
				self.infer_expr(*expr)?;
				Ok(())
			}
			_ => Ok(()),
		}
	}

	fn infer_var_decl(&mut self, var: &VarDecl) -> Result<(), FluxError> {
		if var.ty.node == flux_hir::Type::Unknown {
			let ty = self.infer_expr(var.value)?;
			let id = self.insert(ty.clone());
			self.local_ids.insert(var.name.clone(), id);
			self.locals.insert(id, ty);
		} else {
			let var_ty_id = self.insert(var.ty.clone());
			let expr_ty = self.infer_expr(var.value)?;
			let expr_ty_id = self.insert(expr_ty);
			let final_ty = self.unify(
				var_ty_id,
				expr_ty_id,
				Span::new(
					TextRange::new(
						var.ty.span.range.start(),
						self.expr_arena[var.value].span.range.end(),
					),
					var.ty.span.file_id,
				),
			)?;
			let final_ty_id = self.insert(final_ty.clone());
			self.local_ids.insert(var.name.clone(), final_ty_id);
			self.locals.insert(final_ty_id, final_ty);
		}
		Ok(())
	}

	fn infer_expr(&mut self, idx: Idx<Spanned<Expr>>) -> Result<Spanned<Type>, FluxError> {
		let res = match &self.expr_arena[idx].node {
			Expr::Int(_) => Spanned::new(Type::Int, self.expr_arena[idx].span.clone()),
			Expr::Float { .. } => Spanned::new(Type::Float, self.expr_arena[idx].span.clone()),
			Expr::Prefix { expr, .. } => self.infer_expr(*expr)?,
			Expr::Binary { lhs, rhs, .. } => {
				let lhs_ty = self.infer_expr(*lhs)?;
				let lhs_id = self.insert(lhs_ty);
				let rhs_ty = self.infer_expr(*rhs)?;
				let rhs_id = self.insert(rhs_ty);
				let combined_range = TextRange::new(
					self.expr_arena[*lhs].span.range.start(),
					self.expr_arena[*rhs].span.range.end(),
				);
				let mut result = self.unify(
					lhs_id,
					rhs_id,
					Span::new(combined_range, self.expr_arena[*lhs].span.file_id),
				);
				if let Ok(result) = &mut result {
					result.span.range = combined_range;
				}
				return result;
			}
			Expr::Path(path) => Spanned::new(
				Type::Ref(*self.local_ids.get(&path_to_string(path)).unwrap()),
				self.expr_arena[idx].span.clone(),
			),
			Expr::Call(call) => self.infer_call(call)?,
			_ => Spanned::new(Type::Unknown, self.expr_arena[idx].span.clone()),
		};
		Ok(res)
	}

	fn expect_signature(&self, path: &[Spanned<SmolStr>]) -> Result<FunctionSignature, FluxError> {
		let path_span = if path.len() == 1 {
			path[0].span.clone()
		} else {
			Span::new(
				TextRange::new(
					path[0].span.range.start(),
					path.last().unwrap().span.range.end(),
				),
				path[0].span.file_id,
			)
		};
		let path: Vec<SmolStr> = path.iter().map(|s| s.node.clone()).collect();
		for (sig_path, signature) in self.signatures {
			if &path == sig_path {
				return Ok(signature.clone());
			} else if &[self.module_path, &path].concat() == sig_path {
				return Ok(signature.clone());
			}
			for u in self.use_paths {
				let absolute_path = &[self.module_path, u].concat();
				if u.last().unwrap() == path.last().unwrap() {
					if sig_path == u {
						return Ok(signature.clone());
					}
					if sig_path == absolute_path {
						return Ok(signature.clone());
					}
				}
				if absolute_path.last().unwrap() == &path[0] {
					if sig_path == &[absolute_path, &path[1..]].concat() {
						return Ok(signature.clone());
					}
				}
			}
		}

		Err(
			FluxError::default()
				.with_msg(format!(
					"could not find function signature for `{}`",
					path.join("::")
				))
				.with_primary(
					format!(
						"could not find function signature for `{}`",
						path.join("::")
					),
					Some(path_span),
				),
		)
	}

	fn verify_call_args(
		&mut self,
		fn_name: &str,
		call: &Call,
		signature: &FunctionSignature,
	) -> Result<Spanned<Type>, FluxError> {
		let args_len = call.args.len();
		let params_len = signature.param_types.len();
		if args_len != params_len {
			return Err(
				FluxError::default()
					.with_msg(format!(
						"function `{}` expects {} arguments, but {} were provided",
						fn_name, params_len, args_len
					))
					.with_primary(
						format!(
							"function `{}` expects {} arguments, but {} were provided",
							fn_name, params_len, args_len
						),
						Some(self.expr_arena[call.callee].span.clone()),
					)
					.with_label(
						format!("{} arguments supplied", args_len),
						Some(call.args.span.clone()),
					)
					.with_label(
						format!("`{}` defined with {} parameters", fn_name, params_len),
						Some(signature.param_types.span.clone()),
					),
			);
		}

		for (i, arg) in call.args.iter().enumerate() {
			let arg_ty = self.infer_expr(*arg)?;
			let arg_id = self.insert(arg_ty);
			let param_id = self.insert(signature.param_types[i].clone());
			let final_ty = self.unify(arg_id, param_id, self.expr_arena[*arg].span.clone())?;
			let final_id = self.insert(final_ty);
			if let Expr::Path(name) = &self.expr_arena[*arg].node {
				self.local_ids.insert(path_to_string(name), final_id);
			}
		}

		return Ok(signature.return_type.clone());
	}

	fn infer_call(&mut self, call: &Call) -> Result<Spanned<Type>, FluxError> {
		let callee = &self.expr_arena[call.callee];

		match &callee.node {
			Expr::Path(path) => {
				let signature = self.expect_signature(path)?;
				return self.verify_call_args(&path.last().unwrap().node, call, &signature);
			}
			_ => todo!(),
		};
	}

	fn unify(&mut self, a: TypeId, b: TypeId, span: Span) -> Result<Spanned<Type>, FluxError> {
		use Type::*;
		let ty = match (self.locals[&a].node.clone(), self.locals[&b].node.clone()) {
			(Float, Float) => Float,
			(Int, Int) => Int,
			(Int, x) => {
				let span = self.locals[&a].span.clone();
				self.locals.insert(a, Spanned::new(Ref(b), span));
				x
			}
			(x, Int) => {
				let span = self.locals[&b].span.clone();
				self.locals.insert(b, Spanned::new(Ref(a), span));
				x
			}
			(F32, Float) | (Float, F32) | (F32, F32) => F32,
			(F64, Float) | (Float, F64) | (F64, F64) => F64,
			(Ref(a), Ref(b)) => {
				// this pattern isn't necessary, but it saves one function call if both sides are references. Since comparing variables is quite common, i think it makes sense to have
				return self.unify(a, b, span);
			}
			(Ref(a), _) => return self.unify(a, b, span),
			(_, Ref(b)) => return self.unify(a, b, span),
			(SInt(n1), SInt(n2)) => {
				if n1 == n2 {
					SInt(n1)
				} else {
					return Err(self.unification_err(a, b, span));
				}
			}
			(UInt(n1), UInt(n2)) => {
				if n1 == n2 {
					UInt(n1)
				} else {
					return Err(self.unification_err(a, b, span));
				}
			}
			_ => return Err(self.unification_err(a, b, span)),
		};
		Ok(Spanned::new(ty, self.locals[&a].span.clone()))
	}

	fn unification_err(&self, a: TypeId, b: TypeId, span: Span) -> FluxError {
		let mut a_info = self.locals[&a].clone();
		let mut a_i = 0;
		while let Type::Ref(id) = &a_info.node {
			a_info = self.locals.get(id).unwrap().clone();
			a_i += 1;
		}
		let mut b_info = self.locals[&b].clone();
		let mut b_i = 0;
		while let Type::Ref(id) = &b_info.node {
			b_info = self.locals.get(id).unwrap().clone();
			b_i += 1;
		}
		let mut err = FluxError::default()
			.with_msg(format!(
				"could not unify `{}` and `{}`",
				a_info.node, b_info.node
			))
			.with_primary(
				format!("could not unify `{}` and `{}`", a_info.node, b_info.node),
				Some(span),
			)
			.with_label(
				format!("`{}` type", a_info.node),
				Some(self.locals[&a].span.clone()),
			)
			.with_label(
				format!("`{}` type", b_info.node),
				Some(self.locals[&b].span.clone()),
			);

		if a_i > 0 {
			err = err.with_label(
				format!("type `{}` inferred from here", a_info.node),
				Some(a_info.span),
			);
		}
		if b_i > 0 {
			err = err.with_label(
				format!("type `{}` inferred from here", b_info.node),
				Some(b_info.span),
			);
		}
		err
	}
}

fn path_to_string(path: &[Spanned<SmolStr>]) -> SmolStr {
	SmolStr::from(
		path
			.iter()
			.map(|s| s.node.clone())
			.collect::<Vec<SmolStr>>()
			.join("::"),
	)
}

fn typecheck_hir_module(
	hir_module: &mut HirModule,
	function_exports: &FunctionExportTable,
	type_exports: &TypeExportTable,
) -> Result<(), FluxError> {
	let mut signatures = HashMap::new();
	for (path, signature) in function_exports {
		signatures.insert(path.clone(), signature.clone());
	}
	for f in &hir_module.functions {
		if let Some(signature) = generate_function_signature(f) {
			let name = f.name.as_ref().unwrap();
			let path = [hir_module.path.as_slice(), &[name.node.clone()]].concat();
			signatures.insert(path, signature);
		}
	}

	let use_paths: Vec<Vec<SmolStr>> = hir_module
		.uses
		.iter()
		.map(|u| {
			u.path
				.iter()
				.map(|s| s.node.clone())
				.collect::<Vec<SmolStr>>()
		})
		.collect();

	for f in &mut hir_module.functions {
		let mut env = TypeEnv {
			expr_arena: &hir_module.db.exprs,
			id_counter: 0,
			locals: HashMap::new(),
			local_ids: HashMap::new(),
			signatures: &signatures,
			module_path: &hir_module.path,
			use_paths: &use_paths,
		};

		env.infer_function(f)?;

		for stmt in &mut f.block {
			if let Some(stmt) = stmt {
				if let Stmt::VarDecl(var) = &mut stmt.node {
					if let Some(id) = env.local_ids.get(&var.name) {
						var.ty.node = env.locals[id].node.clone();
					}
				}
			}
		}

		println!("{:#?}", f);
	}

	Ok(())
}

pub fn typecheck_hir_modules(
	hir_modules: &mut [HirModule],
	function_exports: &FunctionExportTable,
	type_exports: &TypeExportTable,
) -> Result<(), FluxError> {
	for hir_module in hir_modules {
		typecheck_hir_module(hir_module, function_exports, type_exports)?;
	}
	Ok(())
}
