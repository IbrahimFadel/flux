use flux_syntax::{ast, ast::AstNode};
use smol_str::SmolStr;
use std::collections::HashMap;

#[macro_export]
#[cfg(test)]
macro_rules! test_typeinf {
	($name:ident, $src:literal) => {
		paste::paste! {
				#[test]
				fn [<test_parse_ $name>]() {
					let mut err_reporting = flux_error::FluxErrorReporting::default();
					let file_id = err_reporting
						.add_file(format!("main"), $src.to_string())
						.expect("could not add file");
					let cst = flux_parser::parse($src, file_id);
					assert!(cst.errors.is_empty());
					let root = ast::Root::cast(cst.syntax()).unwrap();
					let (mut hir_module, _) = flux_hir::lower(vec![SmolStr::from("foo")], root, file_id);
					let function_exports = HashMap::new();
					let type_exports = HashMap::new();
					let res = crate::typecheck_hir_module(&mut hir_module, &function_exports, &type_exports);
					let mut s = String::new();

					if let Some(err) = res.err() {
						s = format!("{:#?}", err.to_diagnostic());
					} else {
						for f in &hir_module.functions {
							if let Some(name) = &f.name {
								s += &format!("{}\n--------------------\n", name.node);

								for stmt in &f.block {
									if let Some(stmt) = stmt {
										if let flux_hir::Stmt::VarDecl(var) = &stmt.node {
											s += &format!("{} -> {:?}\n", var.name, var.ty.node);
										}
									}
								}
							}
						}
					}

					let mut settings = insta::Settings::clone_current();
					settings.set_snapshot_path("./snapshots");
					settings.bind(|| {
						insta::assert_snapshot!(s);
					});
				}
		}
	};
}

test_typeinf!(
	int_defaults_u32,
	r#"fn main() {
  let x = 0;
}"#
);

test_typeinf!(
	basic_dependency,
	r#"fn main() {
  i16 x = 0;
	let y = x;
}"#
);

test_typeinf!(
	basic_backwards_dependency,
	r#"fn main() {
  let x = 0;
	i16 y = x;
}"#
);

test_typeinf!(
	function_call_type_propogation,
	r#"fn foo(i13 x) {

}

fn main() {
	let x = 0;
	let y = x;
	foo(y);
}"#
);

test_typeinf!(
	function_call_incorrect_num_args,
	r#"fn foo(i13 x) {

}

fn main() {
	foo();
}"#
);

test_typeinf!(
	function_call_incorrect_arg_types,
	r#"fn foo(i13 x) {

}

fn main() {
	i32 y = 0;
	foo(y);
}"#
);
