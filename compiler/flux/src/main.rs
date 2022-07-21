use flux_driver::{FunctionExportTable, TypeExportTable};
use flux_error::FluxErrorReporting;

const PROJECT_DIR: &str = ".";

fn main() {
	let mut err_reporting = FluxErrorReporting { files: vec![] };
	let mut function_exports = FunctionExportTable::default();
	let mut type_exports = TypeExportTable::default();

	let _ = flux_driver::parse_main_with_dependencies(
		PROJECT_DIR,
		&mut function_exports,
		&mut type_exports,
		&mut err_reporting,
	);

	// for _ in &modules {
	// println!("{:#?}", module);
	// std::fs::write("ast.txt", format!("{:#?}", module)).unwrap();
	// flux_mir::lower::lower_module(module);
	// }
}
