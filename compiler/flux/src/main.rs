use flux_driver::{FunctionExportTable, TypeExportTable};
use flux_error::FluxErrorReporting;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

const PROJECT_DIR: &str = "./examples";

fn main() {
	let subscriber = FmtSubscriber::builder()
		.with_max_level(Level::DEBUG)
		.finish();
	tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

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
		// std::fs::write("ast.txt", format!("{:#?}", module)).unwrap();
		flux_mir::lower_hir_module(&module);
	}
}
