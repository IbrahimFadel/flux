use flux_error::FluxError;
use flux_syntax::syntax_kind::SyntaxKind;

#[derive(Debug, Clone, PartialEq)]
pub(super) enum Event {
	StartNode {
		kind: SyntaxKind,
		forward_parent: Option<usize>,
	},
	AddToken,
	FinishNode,
	Error(FluxError),
	Placeholder,
}
