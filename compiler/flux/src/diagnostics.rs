use flux_diagnostics::{Diagnostic, DiagnosticCode, DiagnosticKind, ToDiagnostic};

#[derive(Debug, Clone)]
pub enum DriverError {
    ReadEntryFile {
        package: String,
        candidate: String,
    },
    ReadConfigFile {
        package: Option<String>,
        candidate: String,
    },
}

impl ToDiagnostic for DriverError {
    fn to_diagnostic(&self) -> Diagnostic {
        match self {
            Self::ReadEntryFile { package, candidate } => Diagnostic::new_without_file(
                DiagnosticKind::Error,
                DiagnosticCode::CouldNotReadEntryFile,
                format!("could not read entry file for package `{package}`"),
            )
            .with_help(format!(
                "create the file `{candidate}` or change its permissions if it already exists"
            )),
            Self::ReadConfigFile { package, candidate } => Diagnostic::new_without_file(
                DiagnosticKind::Error,
                DiagnosticCode::CouldNotReadConfigFile,
                format!(
                    "could not read config file for {}",
                    package
                        .as_ref()
                        .map(|package| format!("package `{package}`"))
                        .unwrap_or(format!("project"))
                ),
            )
            .with_help(format!(
                "create the file `{candidate}` or change its permissions if it already exists"
            )),
        }
    }
}
