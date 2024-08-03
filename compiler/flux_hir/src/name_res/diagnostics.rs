use flux_diagnostics::{fmt::NthSuffix, Diagnostic, DiagnosticCode};
use flux_util::{FileId, FileSpan, Interner, Path, Span, WithSpan, Word};

#[derive(Debug, Clone)]
pub enum ResolutionError<A> {
    EmptyPath {
        path: Path<Word, A>,
    },
    UnresolvedPath {
        path: Path<Word, A>,
        segment: usize,
    },
    PrivateModule {
        path: Path<Word, A>,
        segment: usize,
    },
    UnexpectedItem {
        path: Path<Word, A>,
        expected: String,
        got: String,
    },
}

impl<A> ResolutionError<A> {
    pub(crate) fn to_diagnostic(
        &self,
        file_id: FileId,
        span: Span,
        interner: &'static Interner,
    ) -> Diagnostic {
        let (code, msg, labels) = match self {
            ResolutionError::EmptyPath { .. } => (
                DiagnosticCode::CannotResolveEmptyPath,
                format!("could not resolve empty path"),
                vec![format!("empty path").file_span(file_id, span)],
            ),
            ResolutionError::UnresolvedPath { path, segment } => (
                DiagnosticCode::UnresolvedPath,
                format!("could not resolve path"),
                vec![
                    format!("could not resolve path {}", path.to_string(interner))
                        .file_span(file_id, span),
                    format!(
                        "{}{} segment could not be resolved",
                        segment + 1,
                        (segment + 1).nth_suffix()
                    )
                    .file_span(file_id, span),
                ],
            ),
            ResolutionError::PrivateModule { path, segment } => (
                DiagnosticCode::PrivateModule,
                format!("could not resolve private module"),
                vec![
                    format!(
                        "encountered private module while resolving {}",
                        path.to_string(interner)
                    )
                    .file_span(file_id, span),
                    format!(
                        "{}{} segment was private",
                        segment + 1,
                        (segment + 1).nth_suffix()
                    )
                    .file_span(file_id, span),
                ],
            ),
            ResolutionError::UnexpectedItem {
                path,
                expected,
                got,
            } => (
                DiagnosticCode::UnexpectedItem,
                format!("expected {}", expected),
                vec![
                    format!("expected `{}` to be {}", path.to_string(interner), expected)
                        .file_span(file_id, span),
                    format!("instead got {got}").file_span(file_id, span),
                ],
            ),
        };
        Diagnostic::error(FileSpan::new(file_id, span), code, msg, labels)
    }
}
