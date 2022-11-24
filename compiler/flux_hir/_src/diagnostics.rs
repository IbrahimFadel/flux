use flux_diagnostics::{Diagnostic, DiagnosticCode, ToDiagnostic};
use flux_span::{FileSpanned, InFile, Span, Spanned};
use itertools::Itertools;

pub(crate) enum LoweringDiagnostic {
    Missing {
        msg: FileSpanned<String>,
    },
    CouldNotParseInt {
        span: InFile<Span>,
        msg: String,
    },
    IncorrectNumberOfArgsInCall {
        call_path: InFile<Spanned<String>>,
        expected_number: InFile<Spanned<usize>>,
        got_number: InFile<Spanned<usize>>,
    },
    UnknownFieldInStructExpr {
        unknown_field: InFile<Spanned<String>>,
        struct_definition: InFile<Spanned<String>>,
    },
    IncorrectNumberOfTypeArgs {
        type_decl: InFile<Spanned<String>>,
        num_params: InFile<Spanned<usize>>,
        num_args: InFile<Spanned<usize>>,
    },
    UnusedGenericParams {
        declared_params: InFile<Spanned<Vec<String>>>,
        unused_params: InFile<Spanned<Vec<String>>>,
    },
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
            Self::IncorrectNumberOfArgsInCall {
                call_path,
                expected_number,
                got_number,
            } => Diagnostic::error(
                got_number.map_ref(|num| num.span.range.start().into()),
                DiagnosticCode::IncorrectNumberOfArgsInCall,
                "incorrect number of arguments in call expression".to_string(),
                vec![
                    call_path.map_inner_ref(|msg| {
                        format!("incorrect number of arguments in call to `{}`", msg)
                    }),
                    expected_number.map_inner_ref(|num| {
                        format!(
                            "expected {} argument{}",
                            num,
                            if *num == 1 { "" } else { "s" }
                        )
                    }),
                    got_number.map_inner_ref(|num| {
                        format!("got {} argument{}", num, if *num == 1 { "" } else { "s" })
                    }),
                ],
            ),
            Self::UnknownFieldInStructExpr {
                unknown_field,
                struct_definition,
            } => Diagnostic::error(
                unknown_field.map_ref(|field| field.span.range.start().into()),
                DiagnosticCode::IncorrectNumberOfArgsInCall,
                "unknown field in struct expression".to_string(),
                vec![
                    unknown_field.map_inner_ref(|field| format!("unknown field `{}`", field)),
                    struct_definition
                        .map_inner_ref(|strukt| format!("struct `{}` declared here", strukt)),
                ],
            ),
            Self::IncorrectNumberOfTypeArgs {
                type_decl,
                num_params,
                num_args,
            } => Diagnostic::error(
                num_args.map_ref(|num| num.span.range.start().into()),
                DiagnosticCode::IncorrectNumberOfArgsInCall,
                "incorrect number of type arguments in expression".to_string(),
                vec![
                    num_args.map_inner_ref(|num| {
                        format!(
                            "{} type argument{} supplied",
                            num,
                            if *num == 1 { "" } else { "s" }
                        )
                    }),
                    type_decl.map_inner_ref(|name| {
                        format!(
                            "`{}` declared with {} type parameter{}",
                            name,
                            num_params.inner.inner,
                            if num_params.inner.inner == 1 { "" } else { "s" }
                        )
                    }),
                ],
            ),
            Self::UnusedGenericParams {
                declared_params,
                unused_params,
            } => Diagnostic::error(
                declared_params.map_ref(|params| params.span.range.start().into()),
                DiagnosticCode::UnusedGenericParams,
                "unused generic parameters".to_string(),
                vec![
                    unused_params.map_inner_ref(|params| {
                        format!(
                            "unused generic parameter{} {}",
                            if unused_params.len() == 1 { ""} else { "s" },
                            params.iter().map(|param| format!("`{param}`")).join(", ")
                        )
                    }),
                    declared_params.map_inner_ref(|params| {
                        format!(
                            "declared generic parameter{} {}",
                            if declared_params.len() == 1 { ""} else { "s" },
                            params.iter().map(|param| format!("`{param}`")).join(", ")
                        )
                    }),
                ],
            ),
        }
    }
}
