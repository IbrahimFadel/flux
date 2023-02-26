use flux_diagnostics::{Diagnostic, DiagnosticCode, ToDiagnostic};
use flux_span::{FileSpanned, InFile, Span, Spanned, WithSpan};
use itertools::Itertools;

#[derive(Debug)]
pub(crate) enum LowerError {
    CouldNotResolveModDecl {
        decl: FileSpanned<String>,
        candidate_paths: Vec<String>,
    },
    IncorrectNumArgsInCall {
        expected_number: FileSpanned<usize>,
        got_number: FileSpanned<usize>,
        function: String,
    },
    StmtFollowingTerminatorExpr {
        terminator: InFile<Span>,
        following_expr: InFile<Span>,
    },
    TriedCallingPrivateFunction {
        function: String,
        declared_as_private: InFile<Span>,
        call: InFile<Span>,
    },
    UnknownGenericInWherePredicate {
        /// The unknown generic
        generic: FileSpanned<String>,
        /// Where the generics are declared
        generic_params: InFile<Span>,
    },
    UnresolvedFunction {
        function: FileSpanned<String>,
    },
    UnresolvedImport {
        import: FileSpanned<String>,
    },
}

impl ToDiagnostic for LowerError {
    fn to_diagnostic(&self) -> flux_diagnostics::Diagnostic {
        match self {
            Self::CouldNotResolveModDecl {
                decl,
                candidate_paths,
            } => Diagnostic::error(
                decl.map_ref(|span| span.span.range.start().into()),
                DiagnosticCode::CouldNotResolveModDecl,
                "could not resolve module declaration".to_string(),
                vec![decl.map_inner_ref(|name| {
                    format!("could not resolve module declaration for `{name}`")
                })],
            )
            .with_help(format!(
                "create the module at one of the following paths: {}",
                candidate_paths
                    .iter()
                    .map(|path| format!("`{path}`"))
                    .join(", ")
            )),
            Self::IncorrectNumArgsInCall {
                expected_number,
                got_number,
                function,
            } => Diagnostic::error(
                got_number.map_ref(|span| span.span.range.start().into()),
                DiagnosticCode::IncorrectNumArgsInCall,
                "incorrect number of arguments in function call".to_string(),
                vec![
                    got_number.map_inner_ref(|num| format!("got {num} arguments")),
                    expected_number.map_inner_ref(|num| {
                        format!("expected {num} arguments in function `{function}`")
                    }),
                ],
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
            Self::TriedCallingPrivateFunction {
                function,
                declared_as_private,
                call,
            } => Diagnostic::error(
                call.map_ref(|span| span.range.start().into()),
                DiagnosticCode::TriedCallingPrivateFunction,
                "function is private and inaccessible".to_string(),
                vec![
                    format!("function `{function}` is private").file_span(call.file_id, call.inner),
                    "declared here as private"
                        .to_string()
                        .file_span(declared_as_private.file_id, declared_as_private.inner),
                ],
            ),
            Self::UnknownGenericInWherePredicate {
                generic,
                generic_params,
            } => Diagnostic::error(
                generic.map_ref(|span| span.span.range.start().into()),
                DiagnosticCode::UnknownGenericInWherePredicate,
                "unknown generic used in where predicate".to_string(),
                vec![
                    generic.map_inner_ref(|generic| {
                        format!("unknown generic `{generic}` used in where predicate")
                    }),
                    generic_params
                        .map_ref(|span| "generic parameters declared here".to_string().at(*span)),
                ],
            ),
            Self::UnresolvedFunction { function } => Diagnostic::error(
                function.map_ref(|span| span.span.range.start().into()),
                DiagnosticCode::UnresolvedFunction,
                "unresolved import".to_string(),
                vec![function.map_inner_ref(|path| format!("unresolved function `{path}`"))],
            ),
            Self::UnresolvedImport { import } => Diagnostic::error(
                import.map_ref(|span| span.span.range.start().into()),
                DiagnosticCode::UnresolvedImport,
                "unresolved import".to_string(),
                vec![import.map_inner_ref(|path| format!("unresolved import `{path}`"))],
            ),
        }
    }
}
