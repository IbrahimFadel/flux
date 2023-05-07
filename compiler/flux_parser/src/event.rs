use flux_syntax::SyntaxKind;

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Event {
    StartNode {
        kind: SyntaxKind,
        forward_parent: Option<usize>,
    },
    AddToken,
    FinishNode,
    Error(String),
    Placeholder,
}
