use flux_syntax::SyntaxKind;

use crate::diagnostics::ParserDiagnostic;

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Event {
    StartNode {
        kind: SyntaxKind,
        forward_parent: Option<usize>,
    },
    AddToken,
    FinishNode,
    Error(ParserDiagnostic),
    Placeholder,
}
