use codespan_reporting::term::termcolor::WriteColor;
use flux_syntax::{ast, ast::AstNode};
use smol_str::SmolStr;
use std::{collections::HashMap, str};

struct ErrorStream {
	s: String,
}

impl WriteColor for ErrorStream {
	fn supports_color(&self) -> bool {
		false
	}

	fn is_synchronous(&self) -> bool {
		false
	}

	fn reset(&mut self) -> std::io::Result<()> {
		Ok(())
	}

	fn set_color(
		&mut self,
		_: &codespan_reporting::term::termcolor::ColorSpec,
	) -> std::io::Result<()> {
		Ok(())
	}
}

impl std::io::Write for ErrorStream {
	fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
		let s = match str::from_utf8(buf) {
			Ok(v) => v,
			Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
		};
		self.s += s;
		Ok(buf.len())
	}

	fn flush(&mut self) -> std::io::Result<()> {
		Ok(())
	}
}

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
						let mut writer = ErrorStream {
							s: String::new()
						};
						let _ = codespan_reporting::term::emit(&mut writer, &codespan_reporting::term::Config::default(), &err_reporting.files, &err.to_diagnostic());
						s = writer.s;
					} else {
						for f in &hir_module.functions {
							if let Some(name) = &f.name {
								s += &format!("{}\n--------------------\n", name.node);
								for stmt in &f.block.0 {
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
	function_call_type_propagation,
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

test_typeinf!(
	function_call_type_propagation_to_addition,
	r#"fn foo(i13 x) {

}

fn main() {
	let x = 0;
	let y = x;
	i32 a = 1;
	foo(y);
	let z = y + a;
}"#
);

test_typeinf!(
	function_call_type_propagation_throughout_scopes,
	r#"fn foo(i13 x) {
	
}

fn main() {
	let x = 0;
	let y = x;
	foo(y);

	if x == 0 {
		let z = y;
		i32 y = 10;
		if z == y {

		}
	}
}"#
);
