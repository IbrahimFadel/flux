#[macro_export]
#[cfg(test)]
macro_rules! test_typeinf_success {
	($name:ident, $src:literal) => {
		paste::paste! {
				#[test]
				fn [<test_parse_ $name>]() {
					let mut err_reporting = FluxErrorReporting::default();
					let file_id = err_reporting
						.add_file(format!("main"), $src.to_string())
						.expect("could not add file");
					let cst = parse($src, file_id);
					assert!(cst.errors.is_empty());
					let root = ast::Root::cast(cst.syntax()).unwrap();
					let (mut hir_module, _) = lower(String::from("foo"), root, file_id);
					let function_exports = HashMap::new();
					let type_exports = HashMap::new();
					let res = crate::typecheck_hir_module(&mut hir_module, &function_exports, &type_exports);
					assert!(res.is_ok());
					let s = format!("{:#?}", hir_module);
					let mut settings = insta::Settings::clone_current();
					settings.set_snapshot_path("./snapshots");
					settings.bind(|| {
						insta::assert_snapshot!(s);
					});
				}
		}
	};
}

// test_typeinf_success!(
// 	basic,
// 	r#"fn main() {
//   let x = 0;
// }"#
// );

// test_typeinf_success!(
// 	basic_dependency,
// 	r#"fn main() {
//   let x = 0;
// 	i7 y = x;
// }"#
// );

// test_typeinf_success!(
// 	function_call_change_var_types,
// 	r#"type Foo i32

// fn foo(u13 a) -> Foo {
//   Foo x = 0;
// }

// fn main() {
//   let x = 1 + 4;
//   let y = x;
//   let z = foo(y);
// }"#
// );
