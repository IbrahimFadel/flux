use std::{collections::HashMap, fs, path::Path, process::exit};

use flux_error::{FluxError, FluxErrorReporting};
use flux_hir::HirModule;
use flux_parser::parse;
use flux_syntax::{
	ast,
	ast::{AstNode, Spanned},
};

#[derive(Debug, Clone)]
pub struct FunctionSignature {
	pub return_type: Spanned<flux_hir::Type>,
	pub param_types: Vec<Spanned<flux_hir::Type>>,
}
pub type FunctionExportTable = HashMap<Vec<String>, FunctionSignature>;
pub type TypeExportTable = HashMap<Vec<String>, Spanned<flux_hir::Type>>;

/// Given a mod name `foo`, search for the corresponding source file:
/// `./foo.flx` || `./foo/foo.flx`
/// OK((path it found, file content))
/// Err(FileNotFound)
fn find_path_with_mod_name(
	parent_dir: &str,
	name: &Spanned<String>,
) -> Result<(String, String), FluxError> {
	let src = fs::read_to_string(&format!("{}/{}.flx", parent_dir, name.node));
	if let Some(_) = src.as_ref().err() {
		let src = fs::read_to_string(&format!("{}/{}/{}.flx", parent_dir, name.node, name.node));
		if let Some(_) = src.as_ref().err() {
			Err(
				FluxError::default()
					.with_msg(format!("could not find module `{}`", name.node))
					.with_primary(
						format!("could not find module `{}`", name.node),
						Some(name.span.clone()),
					)
					.with_label(
						format!(
							"no such file `{}/{}.flx` or `{}/{}/{}.flx`",
							parent_dir, name.node, parent_dir, name.node, name.node
						),
						Some(name.span.clone()),
					),
			)
		} else {
			Ok((
				format!("{}/{}/{}.flx", parent_dir, name.node, name.node),
				src.unwrap(),
			))
		}
	} else {
		Ok((format!("{}/{}.flx", parent_dir, name.node), src.unwrap()))
	}
}

fn populate_export_table(
	module: &HirModule,
	module_path: Vec<String>,
	function_exports: &mut FunctionExportTable,
	type_exports: &mut TypeExportTable,
) {
	for f in &module.functions {
		if f.public.node {
			if let Some(name) = &f.name {
				let mut path = module_path.clone();
				path.push(name.to_string());
				function_exports.insert(path, generate_function_signature(f));
			}
		}
	}
	for ty in &module.types {
		if ty.public.node {
			let mut path = module_path.clone();
			path.push(ty.name.to_string());
			type_exports.insert(path, ty.ty.clone());
		}
	}
}

fn parse_file_and_submodules<'a>(
	parent_dir: &str,
	module_path: Vec<String>,
	name: &Spanned<String>,
	err_reporting: &mut FluxErrorReporting,
	function_exports: &mut FunctionExportTable,
	type_exports: &mut TypeExportTable,
	hir_modules: &mut Vec<HirModule>,
) {
	let src = find_path_with_mod_name(parent_dir, name);
	if let Some(err) = src.as_ref().err() {
		err_reporting.report(&[err.clone()]);
		return;
	}
	let (path, src) = src.unwrap();

	let file_id = err_reporting
		.add_file(format!("{}/{}.flx", parent_dir, name.node), src.clone())
		.expect("could not add file");
	let cst = parse(src.as_str(), file_id);
	err_reporting.report(&cst.errors);
	let root = ast::Root::cast(cst.syntax()).unwrap();
	let (hir_module, errors) = flux_hir::lower(name.node.clone(), root, file_id);
	err_reporting.report(&errors);
	err_reporting.report(&hir_module.db.errors);
	if errors.len() + cst.errors.len() + hir_module.db.errors.len() > 0 {
		exit(1);
	}

	// let child = dependency_graph.add_node(hir_module.clone());
	// dependency_graph.add_edge(parent, child, DependencyType::SubModule);
	populate_export_table(
		&hir_module,
		module_path.clone(),
		function_exports,
		type_exports,
	);

	for m in &hir_module.mods {
		let parent_dir = Path::new(&path).parent().unwrap();
		let mut module_path = module_path.clone();
		module_path.push(m.name.node.clone());
		parse_file_and_submodules(
			parent_dir.to_str().unwrap(),
			module_path,
			&m.name,
			err_reporting,
			function_exports,
			type_exports,
			hir_modules,
		);
	}

	hir_modules.push(hir_module);
}

fn generate_function_signature(f: &flux_hir::FnDecl) -> FunctionSignature {
	FunctionSignature {
		return_type: f.return_type.clone(),
		param_types: f.params.iter().map(|p| p.ty.clone()).collect(),
	}
}

pub fn parse_main_with_dependencies(
	directory: &str,
	function_exports: &mut FunctionExportTable,
	type_exports: &mut TypeExportTable,
	err_reporting: &mut FluxErrorReporting,
) -> Vec<HirModule> {
	let mut hir_modules = vec![];
	let src = fs::read_to_string(&format!("{}/{}", directory, "main.flx")).unwrap();
	let entry_file_id = err_reporting
		.add_file(format!("{}/main.flx", directory), src.clone())
		.expect("could not add file");
	let entry_cst = parse(src.as_str(), entry_file_id);
	err_reporting.report(&entry_cst.errors);
	let root = ast::Root::cast(entry_cst.syntax()).unwrap();
	let (hir_module, errors) = flux_hir::lower(String::from("main"), root, entry_file_id);
	err_reporting.report(&errors);
	err_reporting.report(&hir_module.db.errors);
	if errors.len() + entry_cst.errors.len() + hir_module.db.errors.len() > 0 {
		exit(1);
	}

	populate_export_table(
		&hir_module,
		vec!["pkg".to_string()],
		function_exports,
		type_exports,
	);

	for m in &hir_module.mods {
		parse_file_and_submodules(
			directory,
			vec!["pkg".to_string(), m.name.node.clone()],
			&m.name,
			err_reporting,
			function_exports,
			type_exports,
			&mut hir_modules,
		);
	}
	hir_modules.push(hir_module);
	hir_modules
}
