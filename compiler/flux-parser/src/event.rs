use flux_syntax::syntax_kind::SyntaxKind;

use crate::errors::ParseError;

#[derive(Debug, Clone, PartialEq)]
pub enum Event {
	StartNode {
		kind: SyntaxKind,
		forward_parent: Option<usize>,
	},
	AddToken,
	FinishNode,
	Error(ParseError),
	Placeholder,
}
