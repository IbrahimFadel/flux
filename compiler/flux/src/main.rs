use flux_driver::{FunctionExportTable, TypeExportTable};
use flux_error::FluxErrorReporting;

const PROJECT_DIR: &str = ".";

trait ToI32 {}

struct Foo;

struct Bar<T: ToI32> {
	pub x: T,
}

fn main() {
	let mut err_reporting = FluxErrorReporting { files: vec![] };
	let mut function_exports = FunctionExportTable::default();
	let mut type_exports = TypeExportTable::default();

	let modules = flux_driver::parse_main_with_dependencies(
		PROJECT_DIR,
		&mut function_exports,
		&mut type_exports,
		&mut err_reporting,
	);

	for module in &modules {
		// println!("{:#?}", module);
		// fs::write("ast.txt", format!("{:#?}", module));
		// flux_mir::lower::lower_module(module);
	}
}
