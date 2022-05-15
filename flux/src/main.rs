use std::{fs, process::exit};
use flux_error::FluxErrorReporting;
// use flux_hir::HirModule;
use flux_parser::*;
use flux_syntax::{ast, ast::AstNode};
use flux_typecheck::*;

// struct FluxModule {
// 	name: String,
// 	cst: Parse,
// 	hir: HirModule,
// }

fn foo(x: u8) {}

fn main() {
	let src = fs::read_to_string("examples/main.flx").unwrap();
	let mut err_reporting = FluxErrorReporting::default();
	let file_id = err_reporting
		.add_file(format!("main"), src.clone())
		.expect("could not add file");
	let cst = parse(src.as_str(), file_id);
	err_reporting.report(&cst.errors);
	let root = ast::Root::cast(cst.syntax()).unwrap();
	let mut hir_module = flux_hir::lower(root, file_id);
	err_reporting.report(&hir_module.db.errors);
	if cst.errors.len() + hir_module.db.errors.len() > 0 {
		exit(1);
	}

	let res = typecheck_hir_module(&mut hir_module);
	if let Some(err) = res.err() {
		err_reporting.report(&[err]);
		exit(1);
	}
	// println!("{:#?}", hir_module);
}
