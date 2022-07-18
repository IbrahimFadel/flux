use std::io::Write;

use ariadne::{sources, Report};
use smol_str::SmolStr;

use flux_error::Error;
use flux_span::{FileId, Span};
use flux_syntax::{ast, ast::AstNode};

use crate::HirModule;

use super::error::LowerError;

struct Buf(pub String);

impl Write for &mut Buf {
	fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
		let s = String::from_utf8_lossy(buf);
		self.0 += s.into_owned().as_str();
		Ok(buf.len())
	}

	fn flush(&mut self) -> std::io::Result<()> {
		Ok(())
	}
}

fn lower_str(src: &str) -> (HirModule, Vec<LowerError>) {
	let file_id = flux_span::FileId("main.flx".into());
	let cst = flux_parser::parse(src, file_id.clone());
	let root = ast::Root::cast(cst.syntax()).unwrap();
	crate::lower(vec![SmolStr::from("main.flx")], root, file_id)
}

fn render_report(report: &Report<Span>, files: Vec<(FileId, String)>) -> String {
	let mut s = Buf(String::new());
	report.write(sources(files), &mut s).unwrap();
	s.0
}

#[macro_export]
#[cfg(test)]
macro_rules! unsuccessful {
	($name:ident, $src:literal) => {
		paste::paste! {
				#[test]
				fn [<test_lower_ $name>]() {
										let (_, errors) = lower_str($src);
										let s = errors.iter().map(|err| render_report(&err.to_report(), vec![(flux_span::FileId("main.flx".into()), String::from($src))])).collect::<Vec<_>>().join("\n");
                                        insta::assert_snapshot!(s);

				}
		}
	};
}

unsuccessful!(
	type_mismatch,
	r#"fn main() -> i32 => {
}"#
);
