use std::{
    env, fs,
    path::{Path, PathBuf},
};

use crate::{diagnostics::DriverError, ExitStatus, INTERNER};

use flux_cfg::{Config, CFG_FILE_NAME};
use flux_diagnostics::{ice, IOError, SourceCache};
use flux_hir::{lower_package, lower_package_bodies};
use lasso::ThreadedRodeo;

#[derive(clap::Args, Debug)]
pub struct Args {
    /// Path to root directory of the flux project
    ///
    /// Defaults to current directory
    #[arg(long)]
    root_path: Option<PathBuf>,

    /// Debug the CST
    ///
    /// Defaults to false.
    /// If true prints the CST
    #[arg(long)]
    debug_cst: bool,

    /// Debug the Item Tree
    ///
    /// Defaults to false.
    /// If true prints the Item Tree
    #[arg(long)]
    debug_item_tree: bool,
}

pub fn build(args: Args) -> ExitStatus {
    let project_root = args.root_path.unwrap_or(env::current_dir().unwrap_or_else(|err| ice(&format!("could not determine project root path, make sure you have the proper permissions for this directory: {:?}", err))));
    let debug_cst = args.debug_cst;
    let debug_item_tree = args.debug_item_tree;
    tracing::info!(project_root =? project_root, "executing build command");

    let cfg = match get_config(&project_root) {
        Ok(cfg) => cfg,
        Err(diagnostic) => {
            diagnostic.report();
            return ExitStatus::Failure;
        }
    };

    let lowering_config = flux_hir::cfg::Config {
        debug_cst,
        debug_item_tree,
    };

    for pkg in &cfg.packages {
        build_package(&project_root.join(&pkg.name), &lowering_config)
            .into_iter()
            .for_each(|err| {
                err.report();
            });
    }

    ExitStatus::Success
}

pub fn build_package(root_dir: &Path, config: &flux_hir::cfg::Config) -> Vec<IOError> {
    let cfg = match get_config(root_dir) {
        Err(err) => {
            return vec![err];
        }
        Ok(cfg) => cfg,
    };

    let interner = INTERNER.get_or_init(|| ThreadedRodeo::new());
    let mut source_cache = SourceCache::new(interner);

    assert_eq!(cfg.packages.len(), 1);
    let pkg = &cfg.packages[0];
    tracing::info!(package =? pkg, "building package");

    let (entry_path, content) = match get_package_entry_file_path(root_dir, &pkg.name) {
        Ok(x) => x,
        Err(err) => {
            return vec![err];
        }
    };

    let entry_file_id = source_cache.add_input_file(&entry_path, content.clone());

    let (mut pkg, diagnostics) =
        lower_package(entry_file_id, &content, interner, &mut source_cache, config);
    source_cache.report_diagnostics(diagnostics.iter());
    let diagnostics = lower_package_bodies(&mut pkg, interner);
    source_cache.report_diagnostics(diagnostics.iter());

    vec![]
}

fn get_config(project_root: &Path) -> Result<Config, IOError> {
    let cfg_path = project_root.join(CFG_FILE_NAME);
    let content = fs::read_to_string(&cfg_path).map_err(|_error| {
        DriverError::ReadConfigFile {
            candidate: cfg_path.to_str().unwrap().to_string(),
        }
        .to_io_error()
    })?;
    Ok(flux_cfg::parse_cfg(&content))
}

fn get_package_entry_file_path(
    package_root: &Path,
    package_name: &str,
) -> Result<(String, String), IOError> {
    let file_path = package_root.join("src/main.flx");
    std::fs::read_to_string(&file_path)
        .map_err(|_error| {
            DriverError::ReadEntryFile {
                package: package_name.to_string(),
                candidate: file_path.to_str().unwrap().to_string(),
            }
            .to_io_error()
        })
        .and_then(|content| Ok((file_path.to_str().unwrap().to_string(), content)))
}
