use ariadne::Report;
use flux_lexer::TokenKind;
use flux_span::{Span, Spanned};

#[derive(Debug, Clone, PartialEq)]
pub enum ParseError {
	ExpectedToken {
		expected: Vec<TokenKind>,
		got: Spanned<TokenKind>,
	},
	// More general which can allow for more specific messages and for expecting larger constructs like types or parameter
	Expected {
		expected: Spanned<String>,
	},
}

impl<'a> Into<&'a Report<Span>> for &'a ParseError {
	fn into(self) -> &'a Report<Span> {
		todo!()
	}
}
