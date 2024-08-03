use flux_diagnostics::{DiagnosticCode, IOError};

#[derive(Debug, Clone)]
pub enum DriverError {
    ReadConfigFile { candidate: String },
    ReadEntryFile { package: String, candidate: String },
}

impl DriverError {
    pub fn to_io_error(self) -> IOError {
        match self {
            DriverError::ReadConfigFile { candidate } => IOError::new(
                DiagnosticCode::CouldNotReadConfigFile,
                format!("could not read config file for project"),
                vec![format!(
                    "create the file `{candidate}` or change its permissions if it already exists"
                )],
            ),
            DriverError::ReadEntryFile { candidate, .. } => IOError::new(
                DiagnosticCode::CouldNotReadEntryFile,
                format!("could not read entry file for project"),
                vec![format!(
                    "create the file `{candidate}` or change its permissions if it already exists"
                )],
            ),
        }
    }
}
