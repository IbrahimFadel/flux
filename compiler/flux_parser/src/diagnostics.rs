use flux_diagnostics::{Diagnostic, DiagnosticCode, ToDiagnostic};
use flux_span::FileSpanned;

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum ParserDiagnostic {
    Unxpected { expected: FileSpanned<String> },
}

impl ToDiagnostic for ParserDiagnostic {
    fn to_diagnostic(&self) -> flux_diagnostics::Diagnostic {
        match self {
            Self::Unxpected { expected } => Diagnostic::error(
                expected.cloned_map(|msg| msg.span.range.start().into()),
                DiagnosticCode::ParserExpected,
                expected.inner.inner.to_string(),
                vec![expected.clone()],
            ),
        }
    }
}
