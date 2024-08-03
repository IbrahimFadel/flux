use flux_diagnostics::{Diagnostic, DiagnosticCode, ToDiagnostic};
use flux_util::FileSpanned;

#[derive(Debug, Clone)]
pub(crate) enum ParserDiagnostic {
    Unxpected { expected: FileSpanned<String> },
}

impl ToDiagnostic for ParserDiagnostic {
    fn to_diagnostic(&self) -> flux_diagnostics::Diagnostic {
        match self {
            Self::Unxpected { expected } => Diagnostic::error(
                expected.as_ref().map(|msg| msg.span).to_file_span(),
                DiagnosticCode::ParserExpected,
                "expected syntax not found".to_string(),
                vec![expected.clone()],
            ),
        }
    }
}
