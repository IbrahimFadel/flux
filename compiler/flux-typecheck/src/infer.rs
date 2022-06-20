use flux_error::Span;
use flux_hir::{Binary, ExprIdx};
use text_size::TextRange;

use super::*;

mod unification;

#[derive(Debug)]
pub(super) struct TypeEnv<'a> {
	id_counter: usize,
	scopes: Vec<SymbolTable>,
	functions: &'a HashMap<Vec<SmolStr>, FunctionSignature>,
	types: &'a HashMap<Vec<SmolStr>, &'a Spanned<Type>>,
	module_path: &'a [SmolStr],
	use_paths: &'a [Vec<SmolStr>],
}

impl<'a> TypeEnv<'a> {
	pub fn new(
		functions: &'a HashMap<Vec<SmolStr>, FunctionSignature>,
		types: &'a HashMap<Vec<SmolStr>, &'a Spanned<Type>>,
		module_path: &'a [SmolStr],
		use_paths: &'a [Vec<SmolStr>],
	) -> Self {
		Self {
			id_counter: 0,
			scopes: vec![],
			functions,
			types,
			module_path,
			use_paths,
		}
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

	fn insert(&mut self, info: Spanned<Type>) -> TypeId {
		let id = self.id_counter;
		self.id_counter += 1;
		self.scopes.last_mut().unwrap().locals.insert(id, info);
		id
	}
}

pub(crate) fn infer_expr(
	env: &mut TypeEnv,
	exprs: &mut Arena<Spanned<Expr>>,
	idx: ExprIdx,
) -> FluxResult<Spanned<Type>> {
	let res = match &exprs[idx].node {
		Expr::Int(_) => Spanned::new(Type::Int, exprs[idx].span.clone()),
		Expr::Float(_) => Spanned::new(Type::Float, exprs[idx].span.clone()),
		Expr::Prefix { expr, .. } => infer_expr(env, exprs, *expr)?,
		Expr::Binary(binary) => {
			let lhs_ty = infer_expr(env, exprs, binary.lhs)?;
			let lhs_id = env.insert(lhs_ty);
			let rhs_ty = infer_expr(env, exprs, binary.rhs)?;
			let rhs_id = env.insert(rhs_ty);
			let combined_range = TextRange::new(
				exprs[binary.lhs].span.range.start(),
				exprs[binary.rhs].span.range.end(),
			);
			let mut result = env.unify(
				lhs_id,
				rhs_id,
				Span::new(combined_range, exprs[binary.lhs].span.file_id),
			);
			if let Ok(result) = &mut result {
				result.span.range = combined_range;
			}
			return result;
		}
		_ => return Err(FluxError::default()),
	};
	Ok(res)
}

fn infer_binary_expr(
	env: &mut TypeEnv,
	exprs: &mut Arena<Spanned<Expr>>,
	binary: &mut Binary,
) -> FluxResult<Spanned<Type>> {
	// let lhs_ty = infer_expr(env, exprs, binary.lhs)?;
	// 		let lhs_id = env.insert(lhs_ty);
	// 		let rhs_ty = infer_expr(env, exprs, binary.rhs)?;
	// 		let rhs_id = env.insert(rhs_ty);
	// 		let combined_range = TextRange::new(
	// 			exprs[binary.lhs].span.range.start(),
	// 			exprs[binary.rhs].span.range.end(),
	// 		);
	// 		let mut result = env.unify(
	// 			lhs_id,
	// 			rhs_id,
	// 			Span::new(combined_range, exprs[binary.lhs].span.file_id),
	// 		);
	// 		if let Ok(result) = &mut result {
	// 			result.span.range = combined_range;
	// 		}
	// 		return result;
	Err(FluxError::default())
}
