use flux_diagnostics::{Diagnostic, DiagnosticCode, ToDiagnostic};
use flux_span::{FileSpanned, InFile, Span, Spanned};

pub(crate) enum LoweringDiagnostic {
    Missing { msg: FileSpanned<String> },
    CouldNotParseInt { span: InFile<Span>, msg: String },
}

impl ToDiagnostic for LoweringDiagnostic {
    fn to_diagnostic(&self) -> flux_diagnostics::Diagnostic {
        match self {
            Self::Missing { msg } => Diagnostic::error(
                msg.map_ref(|msg| msg.span.range.start().into()),
                DiagnosticCode::HirMissing,
                msg.to_string(),
                vec![],
            ),
            Self::CouldNotParseInt { span, msg } => Diagnostic::error(
                span.map_ref(|span| span.range.start().into()),
                DiagnosticCode::CouldNotParseInt,
                "invalid integerr".to_string(),
                vec![FileSpanned::new(
                    Spanned::new(msg.clone(), span.inner),
                    span.file_id,
                )],
            ),
        }
    }
}
