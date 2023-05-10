use std::{fs, path::PathBuf};

use crate::diagnostics::DriverError;
use flux_diagnostics::{Diagnostic, ToDiagnostic};

pub(super) mod build;

fn get_config(
    project_dir_path: &PathBuf,
    package_name: Option<&str>,
) -> Result<String, Diagnostic> {
    let cfg_path = project_dir_path.join("flux.toml");
    fs::read_to_string(&cfg_path).map_err(|_error| {
        // TODO: give specific messages based on io error
        DriverError::ReadConfigFile {
            package: package_name.map(|s| s.to_string()),
            candidate: cfg_path.to_str().unwrap().to_string(),
        }
        .to_diagnostic()
    })
}

// TODO: other commands (separate to their own files when i implement, for now inline mods)
pub mod run {}
pub mod lsp {}
pub mod test {}
pub mod new {}
