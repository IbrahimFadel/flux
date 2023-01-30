use flux_diagnostics::{Diagnostic, DiagnosticCode, ToDiagnostic};
use flux_span::{FileSpanned, InFile, Span, WithSpan};

#[derive(Debug)]
pub(crate) enum LowerError {
    CouldNotResolveFunction {
        path: FileSpanned<String>,
    },
    CouldNotResolvePath {
        path: FileSpanned<String>,
    },
    CouldNotResolveStruct {
        path: FileSpanned<String>,
    },
    UninitializedFieldsInStructExpr {
        struct_name: String,
        missing_fields: FileSpanned<String>,
        declaration_span: InFile<Span>,
    },
    UnnecessaryFieldsInitializedInStructExpr {
        struct_name: String,
        unnecessary_fields: FileSpanned<String>,
        declaration_span: InFile<Span>,
    },
}

impl ToDiagnostic for LowerError {
    fn to_diagnostic(&self) -> flux_diagnostics::Diagnostic {
        match self {
            Self::CouldNotResolveFunction { path } => Diagnostic::error(
                path.map_ref(|span| span.span.range.start().into()),
                DiagnosticCode::CouldNotResolveFunction,
                "could not resolve function".to_string(),
                vec![path.map_inner_ref(|path| format!("could not resolve function `{}`", path))],
            ),
            Self::CouldNotResolvePath { path } => Diagnostic::error(
                path.map_ref(|span| span.span.range.start().into()),
                DiagnosticCode::CouldNotResolvePath,
                "could not resolve path".to_string(),
                vec![path.map_inner_ref(|path| format!("could not resolve path `{}`", path))],
            ),
            Self::CouldNotResolveStruct { path } => Diagnostic::error(
                path.map_ref(|span| span.span.range.start().into()),
                DiagnosticCode::CouldNotResolveStruct,
                "could not resolve struct".to_string(),
                vec![path.map_inner_ref(|path| format!("could not resolve struct `{}`", path))],
            ),
            Self::UninitializedFieldsInStructExpr {
                struct_name,
                missing_fields,
                declaration_span,
            } => Diagnostic::error(
                InFile::new(
                    missing_fields.inner.span.range.start().into(),
                    missing_fields.file_id,
                ),
                DiagnosticCode::UninitializedFieldsInStructExpr,
                "uninitialized fields in struct expression".to_string(),
                vec![
                    missing_fields.map_inner_ref(|list| {
                        format!("struct `{}` missing fields `{}`", struct_name, list)
                    }),
                    format!("`{}` fields declared here", struct_name)
                        .in_file(declaration_span.file_id, declaration_span.inner),
                ],
            ),
            Self::UnnecessaryFieldsInitializedInStructExpr {
                struct_name,
                unnecessary_fields,
                declaration_span,
            } => Diagnostic::error(
                InFile::new(
                    unnecessary_fields.inner.span.range.start().into(),
                    unnecessary_fields.file_id,
                ),
                DiagnosticCode::UnnecessaryFieldsInStructExpr,
                "unnecessary fields in struct expression".to_string(),
                vec![
                    unnecessary_fields.map_inner_ref(|list| {
                        format!(
                            "unnecessary fields `{}` initialized in struct `{}`",
                            list, struct_name
                        )
                    }),
                    format!("`{}` fields declared here", struct_name)
                        .in_file(declaration_span.file_id, declaration_span.inner),
                ],
            ),
        }
    }
}
