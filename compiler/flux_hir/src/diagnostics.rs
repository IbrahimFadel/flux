use flux_diagnostics::{Diagnostic, DiagnosticCode, ToDiagnostic};
use flux_span::FileSpanned;

pub(crate) enum LoweringDiagnostic {
    Missing { msg: FileSpanned<String> },
}

impl ToDiagnostic for LoweringDiagnostic {
    fn to_diagnostic(&self) -> flux_diagnostics::Diagnostic {
        match self {
            Self::Missing { msg } => Diagnostic::error(
                msg.cloned_map(|msg| msg.span.range.start().into()),
                DiagnosticCode::HirMissing,
                msg.to_string(),
                vec![],
            ),
        }
    }
}
