use std::{env, io::BufWriter, path::PathBuf};

use flux_diagnostics::reporting::FileCache;
use flux_hir::BasicFileResolver;
use pretty::BoxAllocator;

use crate::{cfg::parse_cfg, commands::get_config, driver::Driver, ExitStatus, INTERNER};

#[derive(clap::Args, Debug)]
pub struct Args {
    /// Path to root directory of the flux project
    ///
    /// Defaults to current directory
    #[arg(long)]
    root_path: Option<PathBuf>,
}

pub fn build(args: Args) -> ExitStatus {
    let root_path = args.root_path.unwrap_or(env::current_dir().unwrap());
    tracing::info!(root_path =? root_path, "executing build command");

    let cfg_file_content = match get_config(&root_path, None) {
        Err(err) => {
            FileCache::new(&INTERNER).report_diagnostic(&err); // Unfortunately we need file cache for diagnostic reporting (everywhere except for the driver...)
            return ExitStatus::Failure;
        }
        Ok(cfg) => cfg,
    };
    let cfg = parse_cfg(&cfg_file_content);

    let mut driver = Driver::new(cfg, root_path);
    driver.build();
    ExitStatus::Success
}

impl Driver {
    fn build(&mut self) {
        if self.cfg.workspace.is_some() {
            self.build_multi_package_project();
        } else {
            self.build_single_package_project();
        }
    }

    pub(crate) fn build_single_package_project(&mut self) {
        let package_cfg = self.cfg.package.as_ref().unwrap();
        tracing::trace!(package =? package_cfg, "building single package project");

        let (entry_file_content, entry_file_path) = self
            .get_package_entry_file_content(&self.root_directory, &package_cfg.name)
            .unwrap();
        let entry_file_path = entry_file_path.to_str().unwrap();
        let _entry_file_id = self
            .file_cache
            .add_file(entry_file_path, &entry_file_content);

        let (def_map, mut types, hir_first_pass_diagnostics) = flux_hir::build_def_map(
            entry_file_path,
            &mut self.file_cache,
            &INTERNER,
            &BasicFileResolver,
        );

        let (lowered_bodies, hir_body_diagnostics) =
            flux_hir::lower_def_map_bodies(&def_map, &INTERNER, &mut types);

        self.file_cache
            .report_diagnostics(&hir_first_pass_diagnostics);
        self.file_cache.report_diagnostics(&hir_body_diagnostics);

        println!("{}", self.fmt_def_map(&def_map, &lowered_bodies));
    }

    pub(crate) fn build_multi_package_project(&self) {
        let workspace_cfg = self.cfg.workspace.as_ref().unwrap();
        tracing::trace!(packages =? workspace_cfg.packages, "building multi package project");
    }
}
