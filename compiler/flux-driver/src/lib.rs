use std::{collections::HashMap, fs, path::Path, process::exit};

use flux_error::{FileId, FluxError, FluxErrorCode, FluxErrorReporting, Span};
use flux_hir::{hir::Spanned, HirModule};
use flux_parser::parse;
use flux_syntax::{ast, ast::AstNode};
use smol_str::SmolStr;
use text_size::{TextRange, TextSize};

#[derive(Debug, Clone)]
pub struct FunctionSignature {
	pub return_type: Spanned<flux_hir::hir::Type>,
	pub param_types: Spanned<Vec<Spanned<flux_hir::hir::Type>>>,
}
pub type FunctionExportTable = HashMap<Vec<SmolStr>, FunctionSignature>;
pub type TypeExportTable = HashMap<Vec<SmolStr>, Spanned<flux_hir::hir::Type>>;

/// Given a mod name `foo`, search for the corresponding source file:
/// `./foo.flx` || `./foo/foo.flx`
/// OK((path it found, file content))
/// Err(FileNotFound)
fn find_path_with_mod_name(
	parent_dir: &str,
	name: &Spanned<SmolStr>,
) -> Result<(String, String), FluxError> {
	let src = fs::read_to_string(&format!("{}/{}.flx", parent_dir, name.node));
	if let Some(_) = src.as_ref().err() {
		let src = fs::read_to_string(&format!("{}/{}/{}.flx", parent_dir, name.node, name.node));
		if let Some(_) = src.as_ref().err() {
			Err(
				FluxError::build(
					format!("could not find module `{}`", name.node),
					name.span.clone(),
					FluxErrorCode::CouldNotFindModule,
					(
						format!("could not find module `{}`", name.node),
						name.span.clone(),
					),
				)
				.with_label(
					format!(
						"no such file `{}/{}.flx` or `{}/{}/{}.flx`",
						parent_dir, name.node, parent_dir, name.node, name.node
					),
					name.span.clone(),
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
	module_path: Vec<SmolStr>,
	function_exports: &mut FunctionExportTable,
	type_exports: &mut TypeExportTable,
) {
	for f in &module.functions {
		if f.public.node {
			let mut path = module_path.clone();
			path.push(f.name.node.clone());
			function_exports.insert(path, generate_function_signature(f));
		}
	}
	for ty in &module.types {
		if ty.public.node {
			let mut path = module_path.clone();
			path.push(ty.name.node.clone());
			type_exports.insert(path, ty.ty.clone());
		}
	}
}

fn parse_file_and_submodules<'a>(
	parent_dir: &str,
	module_path: Vec<SmolStr>,
	name: &Spanned<SmolStr>,
	err_reporting: &mut FluxErrorReporting,
	function_exports: &mut FunctionExportTable,
	type_exports: &mut TypeExportTable,
	hir_modules: &mut Vec<HirModule>,
) {
	let src = find_path_with_mod_name(parent_dir, name);
	if let Some(err) = src.as_ref().err() {
		err_reporting.report(err);
		return;
	}
	let (path, src) = src.unwrap();

	let file_id = err_reporting.add_file(
		format!("{}/{}.flx", parent_dir, name.node).into(),
		src.clone(),
	);
	let cst = parse(src.as_str(), file_id.clone());
	// println!("{}", cst.debug_tree());
	cst.errors.iter().for_each(|err| err_reporting.report(err));
	let root = ast::Root::cast(cst.syntax()).unwrap();
	let (hir_module, errors) = flux_hir::lower(module_path.clone(), root, file_id);
	errors.iter().for_each(|err| err_reporting.report(err));
	if errors.len() + cst.errors.len() > 0 {
		exit(1);
	}

	populate_export_table(
		&hir_module,
		module_path.clone(),
		function_exports,
		type_exports,
	);

	report_ambiguous_uses(&hir_module.uses, err_reporting);

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

fn generate_function_signature(f: &flux_hir::hir::FnDecl) -> FunctionSignature {
	FunctionSignature {
		return_type: f.return_type.clone(),
		param_types: Spanned::new(
			f.params.iter().map(|p| p.ty.clone()).collect(),
			f.params.span.clone(),
		),
	}
}

fn report_ambiguous_uses(uses: &[flux_hir::hir::UseDecl], err_reporting: &mut FluxErrorReporting) {
	let mut errors = vec![];
	let mut unique_uses: Vec<Spanned<SmolStr>> = vec![]; // hash set
	for u in uses {
		let last = u.path.last().unwrap();
		if let Some(idx) = unique_uses.iter().position(|u| u.node == last.as_str()) {
			errors.push(
				FluxError::build(
					format!("ambiguous `use` for `{}`", last.to_string()),
					last.span.clone(),
					FluxErrorCode::AmbiguousUse,
					(
						format!("ambiguous `use` for `{}`", last.to_string()),
						last.span.clone(),
					),
				)
				.with_label(format!("one here"), unique_uses[idx].span.clone())
				.with_label(format!("another here"), last.span.clone())
				.with_note(format!(
					"(hint) consider doing `use {} as foo;` to disambiguate",
					u.path
						.iter()
						.map(|s| s.to_string())
						.collect::<Vec<String>>()
						.join("::")
				)),
			);
		} else {
			unique_uses.push(last.clone());
		}
	}
	errors.iter().for_each(|err| err_reporting.report(err));
}

pub fn parse_main_with_dependencies(
	directory: &str,
	function_exports: &mut FunctionExportTable,
	type_exports: &mut TypeExportTable,
	err_reporting: &mut FluxErrorReporting,
) -> Vec<HirModule> {
	let mut hir_modules = vec![];
	parse_file_and_submodules(
		directory,
		vec![SmolStr::from("pkg")],
		&Spanned::new(
			SmolStr::from("main"),
			Span::new(
				TextRange::new(TextSize::from(0), TextSize::from(0)),
				FileId("main.flx".into()), // this might be problematic... but like, meh it's just error reporting who cares
			),
		),
		err_reporting,
		function_exports,
		type_exports,
		&mut hir_modules,
	);
	hir_modules
}
