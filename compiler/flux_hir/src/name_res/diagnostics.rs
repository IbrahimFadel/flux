use flux_diagnostics::{fmt::NthSuffix, Diagnostic, DiagnosticCode};
use flux_span::{FileId, FileSpan, Interner, Span, WithSpan};

use crate::hir::Path;

#[derive(Debug)]
pub(crate) enum ResolutionError {
    EmptyPath { path: Path },
    UnresolvedPath { path: Path, segment: usize },
    PrivateModule { path: Path, segment: usize },
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
                        segment,
                        segment.nth_suffix()
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
                    format!("{}{} segment was private", segment, segment.nth_suffix())
                        .file_span(file_id, span),
                ],
            ),
        };
        Diagnostic::error(FileSpan::new(file_id, span), code, msg, labels)
    }
}
