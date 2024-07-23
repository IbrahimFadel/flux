use flux_diagnostics::{fmt::NthSuffix, Diagnostic, DiagnosticCode};
use flux_span::{FileId, FileSpan, Interner, Span, WithSpan};

use crate::hir::Path;

#[derive(Debug)]
pub(crate) enum ResolutionError {
    EmptyPath { path: Path },
    UnresolvedPath { path: Path, segment: usize },
    PrivateModule { path: Path, segment: usize },
    ExpectedTrait { path: Path, got: String },
}

impl ResolutionError {
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
            ResolutionError::ExpectedTrait { path, got } => (
                DiagnosticCode::ExpectedTrait,
                format!("expected trait"),
                vec![
                    format!("expected `{}` to be trait", path.to_string(interner))
                        .file_span(file_id, span),
                    format!("instead got {got}").file_span(file_id, span),
                ],
            ),
        };
        Diagnostic::error(FileSpan::new(file_id, span), code, msg, labels)
    }
}
