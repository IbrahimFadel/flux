use std::path::PathBuf;

use flux_diagnostics::ice;
use lasso::ThreadedRodeo;

use crate::{driver::Driver, get_config, ExitStatus, INTERNER, PRE_INTERNED_VALUES};

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
    #[arg(long)]
    debug_item_tree: bool,

    /// Debug the Item Tree with Expression Bodies
    ///
    /// Defaults to false.
    #[arg(long)]
    debug_bodies: bool,

    /// Prints the stack trace on panic
    #[arg(long)]
    stack_trace: bool,
}

pub fn build(args: Args) -> ExitStatus {
    let project_root = args.root_path.unwrap_or(std::env::current_dir().unwrap_or_else(|err| ice(&format!("could not determine project root path, make sure you have the proper permissions for this directory: {:?}", err))));
    tracing::info!(project_root =? project_root, "executing build command");

    let flux_config = match get_config(&project_root) {
        Ok(cfg) => cfg,
        Err(diagnostic) => {
            diagnostic.report();
            return ExitStatus::Failure;
        }
    };
    let compilation_config = flux_hir::Config {
        debug_cst: args.debug_cst,
        debug_item_tree: args.debug_item_tree,
        debug_bodies: args.debug_bodies,
    };
    let interner = INTERNER.get_or_init(|| ThreadedRodeo::from_iter(PRE_INTERNED_VALUES));
    let mut driver = Driver::new(compilation_config, interner);

    driver.build_project(project_root, &flux_config).1
}
