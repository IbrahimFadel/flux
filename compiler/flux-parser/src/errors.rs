use std::fmt::{Debug, Display};

use ariadne::{Color, Label, Report, ReportKind};
use flux_error::{Error, FluxErrorCode};
use flux_lexer::TokenKind;
use flux_span::{Span, Spanned};

#[derive(Debug, Clone, PartialEq)]
pub enum ParseError {
	UnexpectedToken {
		expected: Vec<TokenKind>,
		got: Spanned<TokenKind>,
	},
	// More general which can allow for more specific messages and for expecting larger constructs like types or parameter
	Unxpected {
		expected: Spanned<String>,
	},
}

impl Error for ParseError {
	fn to_report(&self) -> Report<Span> {
		let report = match self {
			ParseError::UnexpectedToken { expected, got } => Report::build(
				ReportKind::Error,
				got.span.file_id.clone(),
				got.span.range.start().into(),
			)
			.with_code(FluxErrorCode::UnexpectedToken)
			.with_message(format!("unexpected token `{}`", got.inner))
			.with_label(
				Label::new(got.span.clone())
					.with_color(Color::Red)
					.with_message(format!("unexpected token `{}`", got.inner)),
			)
			.with_label(
				Label::new(got.span.clone())
					.with_color(Color::Blue)
					.with_message(format!(
						"expected {}{}",
						if expected.len() > 1 {
							format!("either ")
						} else {
							String::new()
						},
						comma_separated_end_with_or(&expected.iter().rev().collect::<Vec<_>>())
					)),
			),
			ParseError::Unxpected { expected } => todo!(),
		};
		report.finish()
	}
}

// eq pls clean
fn comma_separated_end_with_or<T: Debug + Display>(els: &[T]) -> String {
	let mut els: Vec<String> = els.iter().map(|el| format!("`{}`", el)).collect();
	let len = els.len();
	if len > 1 {
		els[len - 1] = format!("or {}", els[len - 1]);
	}
	els.join(", ")
}
