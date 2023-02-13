use flux_diagnostics::{Diagnostic, DiagnosticCode, DiagnosticKind, ToDiagnostic};
use flux_span::{InFile, Spanned};

pub enum DriverError {
    ReadDir(String),
    FindEntryFile,
    FindSubmodule {
        submodule: InFile<Spanned<String>>,
        path1: String,
        path2: String,
    },
}

impl ToDiagnostic for DriverError {
    fn to_diagnostic(&self) -> Diagnostic {
        match self {
            Self::FindEntryFile => Diagnostic::new_without_file(
                DiagnosticKind::Error,
                DiagnosticCode::CouldNotFindEntryFile,
                "could not find entry file".to_string(),
            ),
            Self::ReadDir(msg) => Diagnostic::new_without_file(
                DiagnosticKind::Error,
                DiagnosticCode::CouldNotReadDir,
                msg.to_owned(),
            ),
            Self::FindSubmodule {
                submodule,
                path1,
                path2,
            } => Diagnostic::error(
                submodule.map_ref(|m| m.span.range.start().into()),
                DiagnosticCode::CouldNotFindSubmodule,
                format!("could not find submodule `{}`", submodule.inner.inner),
                vec![submodule.map_inner_ref(|m| format!("could not find submodule `{m}`"))],
            )
            .with_help(format!(
                "make sure that either `{path1}` or `{path2}` exist"
            )),
        }
    }
}
