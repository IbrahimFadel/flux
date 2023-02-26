use flux_diagnostics::{Diagnostic, DiagnosticCode, DiagnosticKind, ToDiagnostic};

pub enum DriverError {
    ReadDir(String),
    FindEntryFile,
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
        }
    }
}
