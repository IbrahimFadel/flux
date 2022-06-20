use flux_driver::{FunctionExportTable, TypeExportTable};
use flux_error::FluxErrorReporting;

const PROJECT_DIR: &str = ".";

fn main() {
	let mut err_reporting = FluxErrorReporting::default();
	let mut function_exports = FunctionExportTable::default();
	let mut type_exports = TypeExportTable::default();

	let modules = flux_driver::parse_main_with_dependencies(
		PROJECT_DIR,
		&mut function_exports,
		&mut type_exports,
		&mut err_reporting,
	);

	println!("{:#?}", modules);

	// let typecheck_result =
	// 	flux_typecheck::typecheck_hir_modules(&mut modules, &function_exports, &type_exports);
	// if let Some(err) = typecheck_result.err() {
	// 	err_reporting.report(&[err]);
	// 	exit(1);
	// }

	// for module in &modules {
	// 	// println!("{:#?}", module);
	// 	fs::write("ast.txt", format!("{:#?}", module));
	// 	flux_mir::lower::lower_module(module);
	// }
}
