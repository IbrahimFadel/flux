use std::collections::HashMap;

use flux_error::{filesystem::FileId, FluxError, Span};
use flux_hir::{Expr, HirModule, Stmt, VarDecl};
use la_arena::{Arena, ArenaMap, Idx};
use smol_str::SmolStr;
use text_size::{TextRange, TextSize};

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
}

#[derive(Debug)]
struct TypeEnv<'a> {
	ident_types: HashMap<&'a str, &'a flux_hir::Type>,
	var_ids: HashMap<&'a str, TypeId>,
	var_types: HashMap<TypeId, TypeInfo>,
	id_counter: u32,

	type_id_exprs: HashMap<TypeId, Idx<Expr>>,
	currently_unifying_ranges: Option<(TextRange, TextRange)>,

	expr_ranges: &'a ArenaMap<Idx<Expr>, TextRange>,
	expr_arena: &'a Arena<Expr>,
}

// impl<'a> Default for TypeEnv<'a> {
// 	fn default() -> Self {
// 		Self {
// 			var_ids: HashMap::new(),
// 			var_types: HashMap::new(),
// 			id_counter: 0,
// 		}
// 	}
// }

impl<'a> TypeEnv<'a> {
	pub fn new(expr_ranges: &'a ArenaMap<Idx<Expr>, TextRange>, expr_arena: &'a Arena<Expr>) -> Self {
		Self {
			ident_types: HashMap::new(),
			var_ids: HashMap::new(),
			var_types: HashMap::new(),
			id_counter: 0,
			type_id_exprs: HashMap::new(),
			expr_ranges,
			currently_unifying_ranges: None,
			expr_arena,
		}
	}

	fn get_expr(&self, idx: Idx<Expr>) -> Expr {
		self.expr_arena[idx].clone()
	}

	fn new_typeid(&mut self, info: TypeInfo) -> TypeId {
		let id = self.id_counter;
		self.id_counter += 1;
		self.var_types.insert(id, info);
		id
	}

	fn get_type(&self, name: &str) -> TypeInfo {
		let id = self.var_ids.get(name).unwrap();
		self.var_types.get(id).unwrap().clone()
	}

	fn hir_type_to_type_info(&self, ty: &flux_hir::Type) -> TypeInfo {
		use flux_hir::Type;
		match ty {
			Type::F64Type => TypeInfo::F64,
			Type::F32Type => TypeInfo::F32,
			Type::INType(int) => TypeInfo::SInt(int.bits),
			Type::UNType(int) => TypeInfo::UInt(int.bits),
			Type::IdentType(ident) => {
				TypeInfo::Ident(ident.clone())
				// self.hir_type_to_type_info(&self.ident_types.get(ident.as_str()).unwrap())
			}
			_ => unreachable!(),
		}
	}

	pub fn test(&mut self, var: &'a VarDecl) -> Result<(), FluxError> {
		let initial_type_info = if var.ty == flux_hir::Type::Missing {
			TypeInfo::Unknown
		} else {
			self.hir_type_to_type_info(&var.ty)
		};
		let var_id = self.new_typeid(initial_type_info.clone());
		self.type_id_exprs.insert(var_id, var.value);
		self.var_ids.insert(&var.name, var_id);
		let value = self.get_expr(var.value);
		let expr_type = self.analyze(&value)?;
		// self.currently_unifying_ranges = Some((var.));
		let final_ty = self.unify(initial_type_info, expr_type)?;
		self.var_types.insert(var_id, final_ty);
		Ok(())
	}

	fn analyze(&mut self, expr: &Expr) -> Result<TypeInfo, FluxError> {
		match expr {
			Expr::Int { .. } => Ok(TypeInfo::Int),
			Expr::Float { .. } => Ok(TypeInfo::Float),
			Expr::Ident { val } => {
				// let info = self.get_type(val);
				// return Ok(info);
				Ok(TypeInfo::Ident(val.clone()))
			}
			Expr::Binary { lhs, rhs, .. } => {
				let lhs_ty = self.analyze(&self.expr_arena[*lhs])?;
				let rhs_ty = self.analyze(&self.expr_arena[*rhs])?;
				// self.currently_unifying_ranges = Some((self.expr_ranges[*lhs], self.expr_ranges[*rhs]));
				self.unify(lhs_ty, rhs_ty)
			}
			Expr::Prefix { expr, .. } => {
				let ty = self.analyze(&self.expr_arena[*expr])?;
				Ok(ty)
			}
			Expr::Missing => {
				Err(FluxError::default().with_msg(format!("cannot run type inference on missing data")))
			}
		}
	}

	fn unify(&self, a: TypeInfo, b: TypeInfo) -> Result<TypeInfo, FluxError> {
		use TypeInfo::*;
		match (&a, &b) {
			(Float, Float) => Ok(Float),
			(F32, Float) | (Unknown, F32) | (F32, F32) => Ok(F32),
			(F64, Float) | (Unknown, F64) | (F64, F64) | (Float, F64) => Ok(F64),
			(Unknown, Float) => Ok(Float),

			(Unknown, Int) => Ok(Int),
			(Unknown, UInt(x)) => Ok(UInt(*x)),
			(UInt(x), Int) => Ok(UInt(*x)),
			(Int, UInt(x)) => Ok(UInt(*x)),
			(SInt(x), Int) => Ok(SInt(*x)),

			(Ident(name), _) => self.unify(
				self.hir_type_to_type_info(&self.ident_types.get(name.as_str()).unwrap()),
				b,
			),

			_ => {
				Err(FluxError::default().with_msg(format!("could not unify {:?} and {:?}", a, b)))
				// let (a_pos, b_pos) = self.currently_unifying_ranges.unwrap();
				// Err(
				// 	FluxError::default()
				// 		.with_msg(format!("could not unify {:?} and {:?}", a, b))
				// 		.with_primary(
				// 			format!("could not unify {:?} and {:?}", a, b),
				// 			Some(Span::new(a_pos, FileId(0))),
				// 		)
				// 		.with_label(format!("{:?}", a), Some(Span::new(a_pos, FileId(0))))
				// 		.with_label(format!("{:?}", b), Some(Span::new(b_pos, FileId(0)))),
				// )
			}
		}
	}
}

pub fn typecheck_hir_module(hir_module: &HirModule) -> Result<(), FluxError> {
	let mut env = TypeEnv::new(&hir_module.db.expr_ranges, &hir_module.db.exprs);

	for ty in &hir_module.types {
		env.ident_types.insert(&ty.name, &ty.ty);
	}

	for f in &hir_module.functions {
		for stmt in &f.block {
			if let Some(stmt) = stmt {
				if let Stmt::VarDecl(var) = stmt {
					env.test(var)?;
				}
			}
		}
	}

	println!("{:#?}", env);
	Ok(())
}
