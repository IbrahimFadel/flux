use flux_diagnostics::{Diagnostic, DiagnosticCode, ToDiagnostic};
use flux_span::{FileSpanned, InFile, Span, Spanned};

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
                            num.to_string(),
                            if *num == 1 { "" } else { "s" }
                        )
                    }),
                    got_number.map_inner_ref(|num| {
                        format!("got {} argument{}", num, if *num == 1 { "" } else { "s" })
                    }),
                ],
            ),
        }
    }
}
