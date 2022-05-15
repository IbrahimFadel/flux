use std::{collections::HashMap, fmt};

use flux_error::FluxError;
use flux_hir::{Call, Expr, ExprIdx, HirModule, Stmt, VarDecl};
use flux_syntax::ast::Spanned;
use la_arena::Arena;
use smol_str::SmolStr;

#[derive(Debug, Clone)]
struct FnSignature {
	param_types: Vec<Spanned<TypeInfo>>,
	return_type: Spanned<TypeInfo>,
}

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

fn generate_function_signature(f: &flux_hir::FnDecl) -> Option<FnSignature> {
	if let Some(_) = f.name {
		let mut param_types = vec![];
		for p in &f.params {
			param_types.push(hir_type_to_type_info(&p.ty));
		}
		let return_type = hir_type_to_type_info(&f.return_type);

		Some(FnSignature {
			param_types,
			return_type,
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

fn type_info_to_hir_type(info: &TypeInfo) -> flux_hir::Type {
	use flux_hir::Type;
	match info {
		TypeInfo::F64 => Type::F64Type,
		TypeInfo::F32 => Type::F32Type,
		TypeInfo::Float => Type::F32Type,
		TypeInfo::SInt(x) => Type::INType(*x),
		TypeInfo::UInt(x) => Type::UNType(*x),
		TypeInfo::Int => Type::UNType(32),
		TypeInfo::Ident(name) => Type::IdentType(name.clone()),
		_ => unreachable!(),
	}
}

#[derive(Debug)]
struct TypeEnv<'a> {
	expr_arena: &'a Arena<Spanned<Expr>>,
	id_counter: u32,
	local_ids: &'a mut HashMap<String, TypeId>,
	local_types: &'a mut HashMap<TypeId, Spanned<TypeInfo>>,
	signatures: &'a HashMap<String, FnSignature>,
}

impl<'a> TypeEnv<'a> {
	fn new_typeid(&mut self, info: Spanned<TypeInfo>) -> TypeId {
		let id = self.id_counter;
		self.id_counter += 1;
		self.local_types.insert(id, info);
		id
	}

	pub fn infer_block(&mut self, block: &'a mut [Option<Spanned<Stmt>>]) -> Result<(), FluxError> {
		for stmt in block {
			if let Some(stmt) = stmt {
				self.infer_stmt(stmt)?;
			}
		}
		Ok(())
	}

	fn infer_stmt(&mut self, stmt: &'a Stmt) -> Result<(), FluxError> {
		match stmt {
			Stmt::VarDecl(var_decl) => self.infer_var_decl(var_decl),
			Stmt::Expr(expr) => {
				self.infer_expr(*expr)?;
				Ok(())
			}
			_ => Ok(()),
		}
	}

	fn infer_var_decl(&mut self, var_decl: &'a VarDecl) -> Result<(), FluxError> {
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
		match &self.expr_arena[expr_idx].node {
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

	fn infer_call(&mut self, call: &Call) -> Result<Spanned<TypeInfo>, FluxError> {
		let callee = &self.expr_arena[call.callee];
		if let Expr::Ident { val: fn_name } = &callee.node {
			let signature = self.signatures.get(fn_name.as_str()).unwrap();

			let args_len = call.args.len();
			let params_len = signature.param_types.len();
			if args_len != params_len {
				return Err(FluxError::default().with_msg(format!(
					"function `{}` expects {} arguments, but {} were provided",
					fn_name, params_len, args_len
				)));
			}

			for (i, arg) in call.args.iter().enumerate() {
				let arg_ty = self.infer_expr(*arg)?;
				let param_ty = signature.param_types[i].clone();
				let final_ty = self.unify(arg_ty, param_ty)?;
				self.propogate_local_ty(*arg, final_ty)?;
			}

			return Ok(signature.return_type.clone());
		}

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

pub fn typecheck_hir_module(hir_module: &mut HirModule) -> Result<(), FluxError> {
	let mut signatures = HashMap::new();
	for f in &hir_module.functions {
		if let Some(sig) = generate_function_signature(f) {
			let name = f.name.as_ref().unwrap();
			signatures.insert(name.clone(), sig);
		}
	}

	for f in &mut hir_module.functions {
		let mut local_ids = HashMap::new();
		let mut local_types = HashMap::new();
		let mut env = TypeEnv {
			expr_arena: &mut hir_module.db.exprs,
			id_counter: 0,
			local_ids: &mut local_ids,
			local_types: &mut local_types,
			signatures: &signatures,
		};

		for p in &f.params {
			if let Some(name) = &p.name {
				let id = env.new_typeid(hir_type_to_type_info(&p.ty));
				env.local_ids.insert(name.to_string(), id);
			}
		}

		env.infer_block(&mut f.block)?;
		println!("{:#?}", env);
	}

	Ok(())
}
