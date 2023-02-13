use flux_diagnostics::{Diagnostic, DiagnosticCode, ToDiagnostic};
use flux_span::{FileSpanned, InFile, Span, Spanned, WithSpan};
use itertools::Itertools;

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
    StmtFollowingTerminatorExpr {
        terminator: InFile<Span>,
        following_expr: InFile<Span>,
    },
    TraitMethodGenericsAlreadyDeclaredInTraitDecl {
        trait_name: String,
        trait_generics: InFile<Spanned<Vec<String>>>,
        method_generics: InFile<Spanned<Vec<String>>>,
        duplicates: Vec<String>,
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
                vec![path.map_inner_ref(|path| format!("could not resolve function `{path}`"))],
            ),
            Self::CouldNotResolvePath { path } => Diagnostic::error(
                path.map_ref(|span| span.span.range.start().into()),
                DiagnosticCode::CouldNotResolvePath,
                "could not resolve path".to_string(),
                vec![path.map_inner_ref(|path| format!("could not resolve path `{path}`"))],
            ),
            Self::CouldNotResolveStruct { path } => Diagnostic::error(
                path.map_ref(|span| span.span.range.start().into()),
                DiagnosticCode::CouldNotResolveStruct,
                "could not resolve struct".to_string(),
                vec![path.map_inner_ref(|path| format!("could not resolve struct `{path}`"))],
            ),
            Self::StmtFollowingTerminatorExpr {
                terminator,
                following_expr,
            } => Diagnostic::error(
                following_expr.map_ref(|span| span.range.start().into()),
                DiagnosticCode::StmtFollowingTerminatorExpr,
                "statements cannot follow a terminator expression in a block".to_string(),
                vec![
                    FileSpanned::new(
                        Spanned::new("terminator expression".to_string(), terminator.inner),
                        terminator.file_id,
                    ),
                    FileSpanned::new(
                        Spanned::new("illegal statement".to_string(), following_expr.inner),
                        following_expr.file_id,
                    ),
                ],
            ),

            Self::TraitMethodGenericsAlreadyDeclaredInTraitDecl {
                trait_name,
                trait_generics,
                method_generics,
                duplicates,
            } => Diagnostic::error(
                InFile::new(
                    method_generics.inner.span.range.start().into(),
                    method_generics.file_id,
                ),
                DiagnosticCode::TraitMethodGenericsAlreadyDeclaredInTraitDecl,
                format!("duplicate generics in trait `{trait_name}`"),
                vec![
                    trait_generics.map_inner_ref(|generics| {
                        format!(
                            "trait generic{} {} declared here",
                            if generics.len() <= 1 { "" } else { "s" },
                            generics
                                .iter()
                                .map(|generic| { format!("`{generic}`") })
                                .join(", ")
                        )
                    }),
                    method_generics.map_inner_ref(|generics| {
                        format!(
                            "method generic{} {} declared here",
                            if generics.len() <= 1 { "" } else { "s" },
                            generics
                                .iter()
                                .map(|generic| { format!("`{generic}`") })
                                .join(", ")
                        )
                    }),
                ],
            )
            .with_help(format!(
                "{} {} in either the trait generic list or the method generic list",
                duplicates.iter().map(|name| format!("`{name}`")).join(", "),
                if duplicates.len() <= 1 {
                    "is a duplicate, change the name".to_string()
                } else {
                    "are duplicates, change the names".to_string()
                },
            )),
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
                        format!("struct `{struct_name}` missing fields `{list}`")
                    }),
                    format!("`{struct_name}` fields declared here")
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
                        format!("unnecessary fields `{list}` initialized in struct `{struct_name}`")
                    }),
                    format!("`{struct_name}` fields declared here")
                        .in_file(declaration_span.file_id, declaration_span.inner),
                ],
            ),
        }
    }
}
