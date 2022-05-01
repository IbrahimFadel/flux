use logos::Logos;
use pi_error::{filesystem::FileId, Span};
use std::ops::Range as StdRange;

mod token_kind;
use text_size::{TextRange, TextSize};
pub use token_kind::TokenKind;

pub struct Lexer<'a> {
	inner: logos::Lexer<'a, TokenKind>,
	file_id: FileId,
}

impl<'a> Lexer<'a> {
	pub fn new(input: &'a str, file_id: FileId) -> Self {
		Self {
			inner: TokenKind::lexer(input),
			file_id,
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
