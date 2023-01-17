use flux_diagnostics::{Diagnostic, DiagnosticCode, ToDiagnostic};
use flux_span::{FileSpanned, InFile, Span, Spanned};

pub(crate) enum TypeError {
    TypeMismatch {
        a: FileSpanned<String>,
        b: FileSpanned<String>,
        span: InFile<Span>,
    },
}

impl ToDiagnostic for TypeError {
    fn to_diagnostic(&self) -> flux_diagnostics::Diagnostic {
        match self {
            Self::TypeMismatch { a, b, span } => Diagnostic::error(
                span.map_ref(|span| span.range.start().into()),
                DiagnosticCode::TypeMismatch,
                "type mismatch".to_string(),
                vec![
                    a.clone(),
                    b.clone(),
                    InFile::new(
                        Spanned::new(
                            format!(
                                "type mismatch between `{}` and `{}`",
                                a.inner.inner, b.inner.inner
                            ),
                            span.inner,
                        ),
                        span.file_id,
                    ),
                ],
            ),
        }
    }
}
