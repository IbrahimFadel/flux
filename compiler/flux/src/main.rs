use std::process::exit;

use flux_driver::{FunctionExportTable, TypeExportTable};
use flux_error::FluxErrorReporting;

const PROJECT_DIR: &str = ".";

fn main() {
	let mut err_reporting = FluxErrorReporting::default();
	let mut function_exports = FunctionExportTable::default();
	let mut type_exports = TypeExportTable::default();

	let mut modules = flux_driver::parse_main_with_dependencies(
		PROJECT_DIR,
		&mut function_exports,
		&mut type_exports,
		&mut err_reporting,
	);

	let typecheck_result =
		flux_typecheck::typecheck_hir_modules(&mut modules, &function_exports, &type_exports);
	if let Some(err) = typecheck_result.err() {
		err_reporting.report(&[err]);
		exit(1);
	}

	// println!("{:#?}", modules);
}
