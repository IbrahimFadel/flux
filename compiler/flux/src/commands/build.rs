use std::{
    env, fs,
    path::{Path, PathBuf},
};

use crate::{diagnostics::DriverError, ExitStatus};

use flux_cfg::{Config, CFG_FILE_NAME};
use flux_diagnostics::{ice, Diagnostic, SourceCache, ToDiagnostic};
use flux_hir::FluxParseInputFileExt;

#[derive(clap::Args, Debug)]
pub struct Args {
    /// Path to root directory of the flux project
    ///
    /// Defaults to current directory
    #[arg(long)]
    root_path: Option<PathBuf>,
}

pub fn build(args: Args) -> ExitStatus {
    let mut db = flux_db::Db::default();

    let project_root = args.root_path.unwrap_or(env::current_dir().unwrap_or_else(|err| ice(&format!("could not determine project root path, make sure you have the proper permissions for this directory: {:?}", err))));
    tracing::info!(project_root =? project_root, "executing build command");

    let cfg = match get_config(&project_root) {
        Ok(cfg) => cfg,
        Err(ref diagnostic) => {
            SourceCache::new(&db).report_diagnostic(diagnostic);
            return ExitStatus::Failure;
        }
    };

    for pkg in &cfg.packages {
        build_package(&mut db, &project_root.join(&pkg.name));
    }

    ExitStatus::Success
}

pub fn build_package(db: &mut flux_db::Db, root_dir: &Path) -> Vec<Diagnostic> {
    let cfg = match get_config(root_dir) {
        Err(err) => {
            return vec![err];
        }
        Ok(cfg) => cfg,
    };

    assert_eq!(cfg.packages.len(), 1);
    let pkg = &cfg.packages[0];
    tracing::info!(package =? pkg, "building package");

    let (entry_path, content) = match get_package_entry_file_path(root_dir, &pkg.name) {
        Ok(x) => x,
        Err(err) => {
            return vec![err];
        }
    };
    let entry_file = db.new_input_file(entry_path, content);
    entry_file.package(db);

    vec![]
}

fn get_config(project_root: &Path) -> Result<Config, Diagnostic> {
    let cfg_path = project_root.join(CFG_FILE_NAME);
    let content = fs::read_to_string(&cfg_path).map_err(|_error| {
        DriverError::ReadConfigFile {
            candidate: cfg_path.to_str().unwrap().to_string(),
        }
        .to_diagnostic()
    })?;
    Ok(flux_cfg::parse_cfg(&content))
}

fn get_package_entry_file_path(
    package_root: &Path,
    package_name: &str,
) -> Result<(String, String), Diagnostic> {
    let file_path = package_root.join("src/main.flx");
    std::fs::read_to_string(&file_path)
        .map_err(|_error| {
            DriverError::ReadEntryFile {
                package: package_name.to_string(),
                candidate: file_path.to_str().unwrap().to_string(),
            }
            .to_diagnostic()
        })
        .and_then(|content| Ok((file_path.to_str().unwrap().to_string(), content)))
}
