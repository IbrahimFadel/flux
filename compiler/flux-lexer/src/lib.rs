use logos::Logos;
use std::ops::Range as StdRange;
mod token_kind;
use text_size::{TextRange, TextSize};
pub use token_kind::TokenKind;

pub struct Lexer<'a> {
	inner: logos::Lexer<'a, TokenKind>,
}

impl<'a> Lexer<'a> {
	pub fn new(input: &'a str) -> Self {
		Self {
			inner: TokenKind::lexer(input),
		}
	}
}

impl<'a> Iterator for Lexer<'a> {
	type Item = Token<'a>;

	fn next(&mut self) -> Option<Self::Item> {
		let kind = self.inner.next()?;
		let text = self.inner.slice();

		let range = {
			let StdRange { start, end } = self.inner.span();
			let start = TextSize::try_from(start).unwrap();
			let end = TextSize::try_from(end).unwrap();

			TextRange::new(start, end)
		};

		Some(Self::Item { kind, text, range })
	}
}

#[derive(Debug, PartialEq)]
pub struct Token<'a> {
	pub kind: TokenKind,
	pub text: &'a str,
	pub range: TextRange,
}

extern "C" {
	pub fn asm_lex(src: *const u8, len: u64) -> u64;
}

#[cfg(test)]
mod tests {
	use paste;

	#[macro_export]
	#[cfg(test)]
	macro_rules! lex_str {
		($name:ident, $src:literal) => {
			paste::paste! {
					#[test]
					fn [<test_lex_ $name>]() {
						let tokens: Vec<_> = crate::Lexer::new($src).collect();
						let mut settings = insta::Settings::clone_current();
						let s = format!("{:#?}", tokens);
						settings.set_snapshot_path("./snapshots");
						settings.bind(|| {
							insta::assert_snapshot!(s);
						});
					}
			}
		};
	}

	lex_str!(prim_ty_in, "i54");
	lex_str!(prim_ty_un, "u54");
	lex_str!(float_sep, "1.0_1");
	lex_str!(float_addition, "1.02+2.40");
	lex_str!(
		all_toks,
		"mod use pub fn type apply to where is mut if else struct trait let return f64 f32 bool,+-*/->=>:(){}[]= => != < > <= >= :: ; & ."
	);
}
