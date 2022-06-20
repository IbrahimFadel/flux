use std::collections::HashMap;

use flux_driver::{FunctionExportTable, FunctionSignature, TypeExportTable};
use flux_error::FluxError;
use flux_hir::{Expr, HirModule, Type};
use flux_syntax::ast::Spanned;
use la_arena::Arena;
use smol_str::SmolStr;

mod check;
use check::TypeCheck;
mod infer;
use infer::TypeEnv;

type FluxResult<T> = Result<T, FluxError>;

type TypeId = usize;

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
		let mut typeenv = TypeEnv::new(&signatures, &types, &hir_module.path, &use_paths);
		let mut check = TypeCheck::new(&mut typeenv, &mut hir_module.db.exprs);
		check.fn_decl(f)?;
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
