use flux_diagnostics::{Diagnostic, DiagnosticCode, DiagnosticKind, ToDiagnostic};

#[derive(Debug, Clone)]
pub enum DriverError {
    ReadConfigFile { candidate: String },
    ReadEntryFile { package: String, candidate: String },
}

impl ToDiagnostic for DriverError {
    fn to_diagnostic(&self) -> Diagnostic {
        match self {
            Self::ReadConfigFile { candidate } => Diagnostic::new_without_file(
                DiagnosticKind::Error,
                DiagnosticCode::CouldNotReadConfigFile,
                format!("could not read config file for project"),
            )
            .with_help(format!(
                "create the file `{candidate}` or change its permissions if it already exists"
            )),
            Self::ReadEntryFile { package, candidate } => Diagnostic::new_without_file(
                DiagnosticKind::Error,
                DiagnosticCode::CouldNotReadEntryFile,
                format!("could not read entry file for project"),
            )
            .with_help(format!(
                "create the file `{candidate}` or change its permissions if it already exists"
            )),
        }
    }
}
