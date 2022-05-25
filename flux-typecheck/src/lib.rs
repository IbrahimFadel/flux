use std::{collections::HashMap, fmt};

use flux_depend::DependencyGraph;
use flux_driver::{FunctionExportTable, FunctionSignature, TypeExportTable};
use flux_error::{FluxError, Span};
use flux_hir::{BinaryOp, Call, Expr, ExprIdx, HirModule, Stmt, VarDecl};
use flux_syntax::ast::Spanned;
use la_arena::{Arena, Idx};
use smol_str::SmolStr;
use text_size::TextRange;

#[cfg(test)]
mod tests;

// #[derive(Debug, Clone)]
// struct FnSignature {
// 	param_types: Vec<Spanned<TypeInfo>>,
// 	return_type: Spanned<TypeInfo>,
// }

type TypeId = u32;

type BitSize = u32;

#[derive(Debug, Clone)]
enum TypeInfo {
	Unknown,
	SInt(BitSize),
	UInt(BitSize),
	Int,
	F64,
	F32,
	Float,
	Ident(SmolStr),
	Ref(TypeId),
	Void,
}

impl fmt::Display for TypeInfo {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match &self {
			TypeInfo::SInt(x) => write!(f, "i{}", x),
			TypeInfo::UInt(x) => write!(f, "u{}", x),
			_ => write!(f, "{:?}", self),
		}
	}
}

fn generate_function_signature(f: &flux_hir::FnDecl) -> Option<FunctionSignature> {
	if let Some(_) = f.name {
		let mut param_types = vec![];
		for p in &f.params {
			param_types.push(p.ty.clone());
		}

		Some(FunctionSignature {
			param_types,
			return_type: f.return_type.clone(),
		})
	} else {
		None
	}
}

fn hir_type_to_type_info(ty: &Spanned<flux_hir::Type>) -> Spanned<TypeInfo> {
	use flux_hir::Type;
	match &ty.node {
		Type::F64Type => Spanned::new(TypeInfo::F64, ty.span.clone()),
		Type::F32Type => Spanned::new(TypeInfo::F32, ty.span.clone()),
		Type::INType(x) => Spanned::new(TypeInfo::SInt(*x), ty.span.clone()),
		Type::UNType(x) => Spanned::new(TypeInfo::UInt(*x), ty.span.clone()),
		Type::IdentType(ident) => Spanned::new(TypeInfo::Ident(ident.clone()), ty.span.clone()),
		Type::VoidType => Spanned::new(TypeInfo::Void, ty.span.clone()),
		_ => panic!("missing type"),
	}
}

#[derive(Debug)]
struct TypeEnv<'a> {
	expr_arena: Arena<Spanned<Expr>>,
	id_counter: u32,
	local_ids: HashMap<String, TypeId>,
	local_types: HashMap<TypeId, Spanned<TypeInfo>>,
	signatures: &'a HashMap<SmolStr, FunctionSignature>,
	type_decls: &'a HashMap<SmolStr, Spanned<TypeInfo>>,
	function_exports: &'a FunctionExportTable,
	type_exports: &'a TypeExportTable,
}

impl<'a> TypeEnv<'a> {
	fn type_info_to_hir_type(&self, info: &TypeInfo) -> flux_hir::Type {
		use flux_hir::Type;
		match info {
			TypeInfo::F64 => Type::F64Type,
			TypeInfo::F32 => Type::F32Type,
			TypeInfo::Float => Type::F32Type,
			TypeInfo::SInt(x) => Type::INType(*x),
			TypeInfo::UInt(x) => Type::UNType(*x),
			TypeInfo::Int => Type::UNType(32),
			TypeInfo::Ident(name) => Type::IdentType(name.clone()),
			TypeInfo::Ref(id) => self.type_info_to_hir_type(self.local_types.get(id).unwrap()),
			_ => unreachable!(format!("cannot convert type info `{:?}` to hir type", info)),
		}
	}

	fn new_typeid(&mut self, info: Spanned<TypeInfo>) -> TypeId {
		let id = self.id_counter;
		self.id_counter += 1;
		self.local_types.insert(id, info);
		id
	}

	pub fn infer_block(&mut self, block: &[Option<Spanned<Stmt>>]) -> Result<(), FluxError> {
		for stmt in block {
			if let Some(stmt) = stmt {
				self.infer_stmt(stmt)?;
			}
		}
		Ok(())
	}

	fn infer_stmt(&mut self, stmt: &Stmt) -> Result<(), FluxError> {
		match stmt {
			Stmt::VarDecl(var_decl) => self.infer_var_decl(var_decl),
			Stmt::Expr(expr) => {
				self.infer_expr(*expr)?;
				Ok(())
			}
			_ => Ok(()),
		}
	}

	fn infer_var_decl(&mut self, var_decl: &VarDecl) -> Result<(), FluxError> {
		if var_decl.ty.node == flux_hir::Type::Missing {
			let ty = self.infer_expr(var_decl.value)?;
			let id = self.new_typeid(ty);
			self.local_ids.insert(var_decl.name.to_string(), id);
		} else {
			let ty = hir_type_to_type_info(&var_decl.ty);
			let inferred_ty = self.infer_expr(var_decl.value)?;
			let final_ty = self.unify(ty, inferred_ty)?;
			let id = self.new_typeid(final_ty);
			self.local_ids.insert(var_decl.name.to_string(), id);
		}
		Ok(())
	}

	fn infer_expr(&mut self, expr_idx: ExprIdx) -> Result<Spanned<TypeInfo>, FluxError> {
		match &self.expr_arena[expr_idx].node.clone() {
			Expr::Int(_) => Ok(Spanned::new(
				TypeInfo::Int,
				self.expr_arena[expr_idx].span.clone(),
			)),
			Expr::Binary { lhs, rhs, .. } => {
				let lhs_ty = self.infer_expr(lhs.clone())?;
				let rhs_ty = self.infer_expr(rhs.clone())?;
				self.unify(lhs_ty, rhs_ty)
			}
			Expr::Ident { val } => Ok(Spanned::new(
				TypeInfo::Ref(*self.local_ids.get(val.as_str()).unwrap()),
				self.expr_arena[expr_idx].span.clone(),
			)),
			Expr::Call(call) => self.infer_call(call),
			_ => Ok(Spanned::new(
				TypeInfo::Unknown,
				self.expr_arena[expr_idx].span.clone(),
			)),
		}
	}

	fn infer_call_with_signature(
		&mut self,
		call: &Call,
		fn_name: &SmolStr,
		signature: &FunctionSignature,
	) -> Result<Spanned<TypeInfo>, FluxError> {
		let args_len = call.args.len();
		let params_len = signature.param_types.len();
		if args_len != params_len {
			return Err(
				FluxError::default()
					.with_msg(format!(
						"function `{}` expects {} arguments, but {} were provided",
						fn_name, params_len, args_len
					))
					.with_label(
						format!(
							"function `{}` expects {} arguments, but {} were provided",
							fn_name, params_len, args_len
						),
						Some(self.expr_arena[call.callee].span.clone()),
					),
			);
		}

		for (i, arg) in call.args.iter().enumerate() {
			let arg_ty = self.infer_expr(*arg)?;
			let param_ty = hir_type_to_type_info(&signature.param_types[i]);
			let final_ty = self.unify(arg_ty, param_ty)?;
			self.propogate_local_ty(*arg, final_ty)?;
		}

		return Ok(hir_type_to_type_info(&signature.return_type));
	}

	fn infer_ident_call(
		&mut self,
		call: &Call,
		fn_name: &SmolStr,
	) -> Result<Spanned<TypeInfo>, FluxError> {
		let signature = self.signatures.get(fn_name.as_str()).unwrap();
		self.infer_call_with_signature(call, fn_name, signature)
	}

	fn infer_binop_doublecolon_call(
		&mut self,
		call: &Call,
		lhs: Idx<Spanned<Expr>>,
		rhs: Idx<Spanned<Expr>>,
	) -> Result<Spanned<TypeInfo>, FluxError> {
		// { { { pkg, foo }, test}, do_test }
		let mut absolute_path: Vec<SmolStr> = vec![];

		let lhs_span_start = self.expr_arena[lhs].span.range.start();
		let rhs_span_end = self.expr_arena[rhs].span.range.end();
		let callee_span = Span::new(
			TextRange::new(lhs_span_start, rhs_span_end),
			self.expr_arena[lhs].span.file_id,
		);

		let mut lhs = lhs;
		while let Expr::Binary { op, lhs: l, rhs: r } = &self.expr_arena[lhs].node {
			let rhs_name = match &self.expr_arena[*r].node {
				Expr::Ident { val } => val,
				_ => {
					return Err(
						FluxError::default()
							.with_msg(format!("expected identifier on lhs of `::`"))
							.with_primary(
								format!("expected identifier on lhs of `::`"),
								Some(self.expr_arena[*r].span.clone()),
							),
					);
				}
			};
			absolute_path.push(rhs_name.clone());
			match &self.expr_arena[*l].node {
				Expr::Ident { val } => absolute_path.push(val.clone()),
				Expr::Binary { op, .. } => {
					if *op != BinaryOp::DoubleColon {
						return Err(
							FluxError::default()
								.with_msg(format!("expected `::` to be operator in function call")),
						);
					}
				}
				_ => panic!(""),
			}
			lhs = *l;
		}

		let fn_name = match self.expr_arena[rhs].node.clone() {
			Expr::Ident { val } => val,
			_ => {
				return Err(
					FluxError::default()
						.with_msg(format!("expected identifier on rhs of `::`"))
						.with_primary(
							format!("expected identifier on rhs of `::`"),
							Some(self.expr_arena[rhs].span.clone()),
						),
					// TODO: search module for public functions and suggest replacements
					// possibly only suggest replacements with the same number of args
				);
			}
		};
		absolute_path = absolute_path.into_iter().rev().collect();
		absolute_path.push(fn_name.clone());

		match self.function_exports.get(&absolute_path) {
			Some(signature) => self.infer_call_with_signature(call, &fn_name, signature),
			_ => Err(
				FluxError::default()
					.with_msg(format!(
						"could not find function signature for `{}::{}`",
						absolute_path.join("::"),
						fn_name
					))
					.with_primary(
						format!(
							"could not find function signature for `{}::{}`",
							absolute_path.join("::"),
							fn_name
						),
						Some(callee_span),
					)
					.with_note(format!(
						"(hint) you might need to mark `{}` as public",
						fn_name
					)),
			),
		}
	}

	fn infer_binop_call(
		&mut self,
		call: &Call,
		op: BinaryOp,
		lhs: Idx<Spanned<Expr>>,
		rhs: Idx<Spanned<Expr>>,
	) -> Result<Spanned<TypeInfo>, FluxError> {
		match op {
			BinaryOp::DoubleColon => self.infer_binop_doublecolon_call(call, lhs, rhs),
			_ => panic!(
				"internal compiler error: typecheck unimplemented for binop call: {:?}",
				op
			),
		}
	}

	fn infer_call(&mut self, call: &Call) -> Result<Spanned<TypeInfo>, FluxError> {
		let callee = &self.expr_arena[call.callee].clone();
		match &callee.node {
			Expr::Ident { val: fn_name } => self.infer_ident_call(call, fn_name)?,
			Expr::Binary { op, lhs, rhs } => self.infer_binop_call(call, op.clone(), *lhs, *rhs)?,
			_ => panic!(
				"internal compiler error: unhandled callee type: {:#?}",
				callee.node
			),
		};
		Ok(Spanned::new(
			TypeInfo::Unknown,
			self.expr_arena[call.callee].span.clone(),
		))
	}

	fn propogate_local_ty(&mut self, expr: ExprIdx, ty: Spanned<TypeInfo>) -> Result<(), FluxError> {
		if let Expr::Ident { val: local_name } = &self.expr_arena[expr].node {
			let mut id = self.local_ids.get(local_name.as_str()).unwrap().clone();
			let mut info = self.local_types.get(&id).unwrap();
			while let TypeInfo::Ref(_id) = info.node {
				id = _id;
				info = self.local_types.get(&id).unwrap();
			}
			self.local_types.insert(id, ty.clone());
		}
		Ok(())
	}

	fn unify(
		&self,
		a: Spanned<TypeInfo>,
		b: Spanned<TypeInfo>,
	) -> Result<Spanned<TypeInfo>, FluxError> {
		use TypeInfo::*;
		match (&a.node, &b.node) {
			(Float, Float) => Ok(Spanned::new(Float, a.span)),
			(F32, Float) | (Unknown, F32) | (F32, F32) => Ok(Spanned::new(F32, a.span)),
			(F64, Float) | (Unknown, F64) | (F64, F64) | (Float, F64) => Ok(Spanned::new(F64, a.span)),
			(Unknown, Float) => Ok(Spanned::new(Float, a.span)),

			(Ref(a), _) => {
				let type_info = self.local_types.get(a).unwrap();
				self.unify(type_info.clone(), b)
			}
			(_, Ref(b)) => {
				let type_info = self.local_types.get(b).unwrap();
				self.unify(a, type_info.clone())
			}
			(Int, Int) | (Unknown, Int) => Ok(Spanned::new(Int, a.span)),
			(Unknown, UInt(x)) => Ok(Spanned::new(UInt(*x), a.span)),
			(Unknown, SInt(x)) => Ok(Spanned::new(SInt(*x), a.span)),
			(UInt(x), Int) | (Int, UInt(x)) => Ok(Spanned::new(UInt(*x), a.span)),
			(SInt(x), Int) | (Int, SInt(x)) => Ok(Spanned::new(SInt(*x), a.span)),
			(UInt(x), UInt(y)) | (SInt(x), SInt(y)) => {
				if x == y {
					Ok(Spanned::new(UInt(*x), a.span))
				} else {
					Err(self.unification_err(&a, &b))
				}
			}

			(Ident(name), _) => self.unify(self.type_decls.get(name.as_str()).unwrap().clone(), b),
			_ => Err(self.unification_err(&a, &b)),
		}
	}

	fn unification_err(&self, a: &Spanned<TypeInfo>, b: &Spanned<TypeInfo>) -> FluxError {
		let mut a_info = a.clone();
		while let TypeInfo::Ref(id) = &a_info.node {
			a_info = self.local_types.get(id).unwrap().clone();
		}
		let mut b_info = b.clone();
		let mut i = 0;
		while let TypeInfo::Ref(id) = &b_info.node {
			b_info = self.local_types.get(id).unwrap().clone();
			i += 1;
		}
		let mut err = FluxError::default()
			.with_msg(format!(
				"could not unify `{}` and `{}`",
				a_info.node, b_info.node
			))
			.with_label(format!("`{}` type", a_info.node), Some(a.span.clone()))
			.with_label(format!("`{}` type", b_info.node), Some(b.span.clone()));
		if i > 0 {
			err = err.with_label(
				format!("type `{}` inferred from here", b_info.node),
				Some(b_info.span),
			);
		}
		err
	}
}

fn typecheck_hir_module(
	hir_module: &mut HirModule,
	function_exports: &FunctionExportTable,
	type_exports: &TypeExportTable,
) -> Result<(), FluxError> {
	let mut types = HashMap::new();
	let mut signatures: HashMap<SmolStr, FunctionSignature> = HashMap::new();
	for ty in &hir_module.types {
		types.insert(ty.name.node.clone(), hir_type_to_type_info(&ty.ty));
	}
	for u in &hir_module.uses {
		let path: Vec<SmolStr> = u.path.iter().map(|s| s.node.clone()).collect();
		if let Some(ty) = type_exports.get(&path) {
			types.insert(SmolStr::from(path.join("::")), hir_type_to_type_info(&ty));
		} else if let Some(f) = function_exports.get(&path) {
			// signatures.insert(path.join("::"), (*f).clone());
			signatures.insert(path.last().unwrap().clone(), (*f).clone());
		}
	}

	for f in &hir_module.functions {
		if let Some(sig) = generate_function_signature(f) {
			let name = f.name.as_ref().unwrap();
			signatures.insert(name.node.clone(), sig);
		}
	}

	for f in &mut hir_module.functions {
		let arena = hir_module.db.exprs.clone();
		let mut env = TypeEnv {
			expr_arena: arena,
			id_counter: 0,
			local_ids: HashMap::new(),
			local_types: HashMap::new(),
			signatures: &signatures,
			type_decls: &types,
			function_exports,
			type_exports,
		};

		for p in &f.params {
			if let Some(name) = &p.name {
				let id = env.new_typeid(hir_type_to_type_info(&p.ty));
				env.local_ids.insert(name.clone().to_string(), id);
			}
		}

		env.infer_block(&f.block)?;

		for stmt in &mut f.block {
			if let Some(stmt) = stmt {
				if let Stmt::VarDecl(var) = &mut stmt.node {
					let id = env.local_ids.get(var.name.as_str()).unwrap();
					let ty = env.local_types.get(id).unwrap();
					var.ty = Spanned::new(env.type_info_to_hir_type(ty), ty.span.clone());
				}
			}
		}
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
