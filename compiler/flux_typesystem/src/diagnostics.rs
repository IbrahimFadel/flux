use flux_diagnostics::{Diagnostic, DiagnosticCode, ToDiagnostic};
use flux_span::{FileSpanned, InFile, Span, Spanned};

pub enum TypeError {
    /// A type mismatch
    ///
    /// `a` and `b` are both formatted to `String`, where
    /// `a` is the type the must be matched by `b`, and
    /// `span` is the location in which the type unification was triggered.
    TypeMismatch {
        a: FileSpanned<String>,
        b: FileSpanned<String>,
        span: InFile<Span>,
    },
    UnknownVariable {
        name: FileSpanned<String>,
    },
    UnknownFunction {
        path: FileSpanned<String>,
    },
    UnknownStruct {
        path: FileSpanned<String>,
    },
    UnknownType {
        path: FileSpanned<String>,
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
                    a.clone(),
                    b.clone(),
                ],
            ),
            Self::UnknownVariable { name } => Diagnostic::error(
                name.map_ref(|span| span.span.range.start().into()),
                DiagnosticCode::UnknownLocal,
                "unknown variable referenced".to_string(),
                vec![FileSpanned::new(
                    Spanned::new(
                        format!("unknown variable `{}` referenced", name.inner.inner),
                        name.span,
                    ),
                    name.file_id,
                )],
            ),
            Self::UnknownFunction { path } => Diagnostic::error(
                path.map_ref(|span| span.span.range.start().into()),
                DiagnosticCode::UnknownFunction,
                "unknown function referenced".to_string(),
                vec![FileSpanned::new(
                    Spanned::new(
                        format!("unknown function `{}` referenced", path.inner.inner),
                        path.span,
                    ),
                    path.file_id,
                )],
            ),
            Self::UnknownStruct { path } => Diagnostic::error(
                path.map_ref(|span| span.span.range.start().into()),
                DiagnosticCode::UnknownStruct,
                "unknown struct referenced".to_string(),
                vec![FileSpanned::new(
                    Spanned::new(
                        format!("unknown struct `{}` referenced", path.inner.inner),
                        path.span,
                    ),
                    path.file_id,
                )],
            ),
            Self::UnknownType { path } => Diagnostic::error(
                path.map_ref(|span| span.span.range.start().into()),
                DiagnosticCode::UnknownStruct,
                "unknown type referenced".to_string(),
                vec![FileSpanned::new(
                    Spanned::new(
                        format!("unknown type `{}` referenced", path.inner.inner),
                        path.span,
                    ),
                    path.file_id,
                )],
            ),
        }
    }
}