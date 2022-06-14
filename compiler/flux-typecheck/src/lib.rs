use std::collections::HashMap;

use flux_driver::{FunctionExportTable, FunctionSignature, TypeExportTable};
use flux_error::{FluxError, Span};
use flux_hir::{Call, Expr, FnDecl, HirModule, If, Stmt, Struct, Type, VarDecl};
use flux_syntax::ast::Spanned;
use la_arena::{Arena, Idx};
use smol_str::SmolStr;
use text_size::TextRange;

#[cfg(test)]
mod tests;
mod unification;

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
struct SymbolTable {
	locals: HashMap<TypeId, Spanned<Type>>,
	local_ids: HashMap<SmolStr, TypeId>,
}

impl Default for SymbolTable {
	fn default() -> Self {
		Self {
			locals: HashMap::new(),
			local_ids: HashMap::new(),
		}
	}
}

#[derive(Debug)]
struct TypeEnv<'a> {
	expr_arena: &'a Arena<Spanned<Expr>>,
	id_counter: usize,
	scopes: Vec<SymbolTable>,
	signatures: &'a HashMap<Vec<SmolStr>, FunctionSignature>,
	module_path: &'a [SmolStr],
	use_paths: &'a [Vec<SmolStr>],
	types: &'a HashMap<Vec<SmolStr>, &'a Spanned<Type>>,
}

impl<'a> TypeEnv<'a> {
	pub fn new(
		expr_arena: &'a Arena<Spanned<Expr>>,
		types: &'a HashMap<Vec<SmolStr>, &'a Spanned<Type>>,
		signatures: &'a HashMap<Vec<SmolStr>, FunctionSignature>,
		module_path: &'a [SmolStr],
		use_paths: &'a [Vec<SmolStr>],
	) -> Self {
		Self {
			expr_arena,
			types,
			id_counter: 0,
			scopes: vec![],
			signatures,
			module_path,
			use_paths,
		}
	}

	fn add_var_to_scope(&mut self, name: SmolStr, ty: Spanned<Type>) {
		let id = self.insert(ty.clone());
		self.scopes.last_mut().unwrap().local_ids.insert(name, id);
		self.scopes.last_mut().unwrap().locals.insert(id, ty);
	}

	fn get_var_id(&self, name: &str) -> Option<&TypeId> {
		for scope in self.scopes.iter().rev() {
			if let Some(id) = scope.local_ids.get(name) {
				return Some(id);
			}
		}
		None
	}

	fn get_type_with_id(&self, id: TypeId) -> &Spanned<Type> {
		for scope in self.scopes.iter().rev() {
			if let Some(ty) = scope.locals.get(&id) {
				return ty;
			}
		}
		panic!(
			"internal compiler error: could not find type with id `{}`",
			id
		);
	}

	fn set_id_type(&mut self, id: TypeId, ty: Spanned<Type>) {
		self.scopes.last_mut().unwrap().locals.insert(id, ty);
	}

	pub fn insert(&mut self, info: Spanned<Type>) -> TypeId {
		let id = self.id_counter;
		self.id_counter += 1;
		self.scopes.last_mut().unwrap().locals.insert(id, info);
		id
	}

	pub fn infer_function(&mut self, f: &mut FnDecl) -> Result<(), FluxError> {
		self.infer_block(&mut f.block.0)?;
		Ok(())
	}

	fn infer_block(&mut self, block: &mut Vec<Option<Spanned<Stmt>>>) -> Result<(), FluxError> {
		self.scopes.push(SymbolTable::default());
		for stmt in block.iter_mut() {
			if let Some(stmt) = stmt {
				self.infer_stmt(&mut stmt.node)?;
			}
		}

		for stmt in block {
			if let Some(stmt) = stmt {
				if let Stmt::VarDecl(var) = &mut stmt.node {
					if let Some(id) = self.get_var_id(&var.name) {
						let mut ty = self.get_type_with_id(*id).node.clone();
						while let Type::Ref(id) = &ty {
							ty = self.get_type_with_id(*id).node.clone();
						}
						if ty == Type::Int {
							ty = Type::UInt(32);
						} else if ty == Type::Float {
							ty = Type::F32;
						}
						var.ty.node = ty;
					}
				}
			}
		}

		self.scopes.pop();
		Ok(())
	}

	fn infer_stmt(&mut self, stmt: &mut Stmt) -> Result<(), FluxError> {
		match stmt {
			Stmt::VarDecl(var) => self.infer_var_decl(var),
			Stmt::If(if_) => self.infer_if(if_),
			Stmt::Expr(expr) => {
				self.infer_expr(*expr)?;
				Ok(())
			}
			_ => Ok(()),
		}
	}

	fn infer_if(&mut self, if_stmt: &mut If) -> Result<(), FluxError> {
		self.infer_expr(if_stmt.condition)?;
		self.infer_block(&mut if_stmt.then.0)?;
		self.infer_block(&mut if_stmt.else_ifs.0)?;
		self.infer_block(&mut if_stmt.else_.0)?;
		Ok(())
	}

	fn infer_var_decl(&mut self, var: &VarDecl) -> Result<(), FluxError> {
		if var.ty.node == flux_hir::Type::Unknown {
			let ty = self.infer_expr(var.value)?;
			self.add_var_to_scope(var.name.clone(), ty);
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
			self.add_var_to_scope(var.name.clone(), final_ty);
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
			Expr::Path(path) => {
				if let Some(var) = self.get_var_id(&path_to_string(path)) {
					Spanned::new(Type::Ref(*var), self.expr_arena[idx].span.clone())
				} else {
					let path_span = Span::new(
						TextRange::new(
							path[0].span.range.start(),
							path.last().unwrap().span.range.end(),
						),
						path[0].span.file_id,
					);
					return Err(
						FluxError::default()
							.with_msg(format!(
								"unknown variable referenced `{}`",
								path_to_string(path)
							))
							.with_primary(
								format!("unknown variable referenced `{}`", path_to_string(path)),
								Some(path_span),
							),
					);
				}
			}
			Expr::Call(call) => self.infer_call(call)?,
			Expr::Struct(struct_expr) => {
				self.infer_struct(struct_expr)?;
				Spanned::new(
					Type::Ident(struct_expr.name.as_ref().unwrap().node.clone()),
					self.expr_arena[idx].span.clone(),
				)
			}
			_ => unreachable!(),
		};
		Ok(res)
	}

	fn infer_struct(&mut self, struct_expr: &Struct) -> Result<(), FluxError> {
		if struct_expr.name.is_none() {
			return Err(FluxError::default());
		}
		let name = struct_expr.name.as_ref().unwrap();
		let ty = self.expect_type(name)?;
		let ty = match ty {
			Type::Struct(struct_ty) => struct_ty,
			_ => panic!("internal compiler error: tried to infer struct but got non-struct type"),
		};

		let ty_len = ty.0.len();
		let e_len = struct_expr.fields.len();
		if ty_len != e_len {
			return Err(
				FluxError::default()
					.with_msg(format!(
						"incorrect number of fields supplied in struct expression"
					))
					.with_primary(
						format!(
							"type `{}` expects {} fields, but {} were supplied",
							name.node, ty_len, e_len
						),
						Some(struct_expr.fields.span.clone()),
					)
					.with_label(
						format!("`{}` defined with {} fields", name.node, ty_len),
						Some(ty.0.span.clone()),
					),
			);
		}

		for (e_name, e_val) in &struct_expr.fields.node {
			if e_name.is_none() {
				return Err(FluxError::default().with_msg(format!("missing field name")));
			}
			let e_name = e_name.as_ref().unwrap();
			if let Some(struct_type_field) = ty.0.get(&e_name.node) {
				let e_ty = self.infer_expr(*e_val)?;
				let e_id = self.insert(e_ty);
				let ty_id = self.insert(struct_type_field.ty.clone());
				self.unify(e_id, ty_id, e_name.span.clone())?;
			} else {
				return Err(
					FluxError::default()
						.with_msg(format!(
							"type `{}` contains no field called `{}`",
							name.node, e_name.node
						))
						.with_primary(
							format!(
								"type `{}` contains no field called `{}`",
								name.node, e_name.node
							),
							Some(e_name.span.clone()),
						),
				);
			}
		}

		Ok(())
	}

	//TODO: resolve imports, uses, etc
	fn expect_type(&self, name: &Spanned<SmolStr>) -> Result<&'a Type, FluxError> {
		if let Some(ty) = self.types.get(&vec![name.node.clone()]) {
			return Ok(ty);
		}
		Err(FluxError::default().with_msg(format!("could not find type `{}`", name.node)))
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
			if let Expr::Path(name) = &self.expr_arena[*arg].node {
				self.add_var_to_scope(path_to_string(name), final_ty);
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
	let mut types = HashMap::new();
	for ty in &hir_module.types {
		types.insert(vec![ty.name.node.clone()], &ty.ty);
	}

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
		let mut env = TypeEnv::new(
			&hir_module.db.exprs,
			&types,
			&signatures,
			&hir_module.path,
			&use_paths,
		);

		env.infer_function(f)?;

		println!("{}", f);
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
