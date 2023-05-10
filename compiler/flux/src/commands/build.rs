use std::{
    env,
    path::{Path, PathBuf},
    sync::Arc,
};

use flux_diagnostics::{reporting::FileCache, Diagnostic, ToDiagnostic};
use flux_hir::BasicFileResolver;

use crate::{
    cfg::{parse_cfg, Dependencies},
    commands::get_config,
    diagnostics::DriverError,
    driver::Driver,
    ExitStatus, INTERNER,
};

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
        self.build_package(&self.root_directory.clone(), &package_cfg.name.to_string());
    }

    pub(crate) fn build_multi_package_project(&mut self) {
        let workspace_cfg = self.cfg.workspace.clone().unwrap();
        tracing::trace!(packages =? workspace_cfg.packages, "building multi package project");

        for package in &workspace_cfg.packages {
            let package_path = self.root_directory.join(package);
            self.build_package(&package_path, package);
        }
    }

    fn build_package(&mut self, root_directory: &PathBuf, name: &str) {
        let name_spur = INTERNER.get_or_intern(name);
        if self.package_name_to_id.get(&name_spur).is_some() {
            return;
        }

        let cfg_file_content = match get_config(&root_directory, Some(name)) {
            Err(err) => {
                self.file_cache.report_diagnostic(&err);
                return;
            }
            Ok(cfg) => cfg,
        };
        let cfg = parse_cfg(&cfg_file_content);
        let package_cfg = cfg.package.as_ref().unwrap();
        tracing::trace!(package =? package_cfg, "building package");

        if let Err(err) = self.resolve_dependencies(root_directory, &cfg.dependencies) {
            self.file_cache.report_diagnostic(&err);
        }

        let entry_file_path = self
            .get_package_entry_file_path(&root_directory, &name)
            .unwrap();
        let entry_file_path = entry_file_path.to_str().unwrap();

        let dependencies: Vec<_> = cfg
            .dependencies
            .local
            .iter()
            .map(|dep| Path::new(dep).file_name().unwrap().to_str().unwrap())
            .filter_map(|name| self.package_name_to_id.get(&INTERNER.get_or_intern(name)))
            .cloned()
            .collect();

        let (def_map, mut types, hir_first_pass_diagnostics) = flux_hir::build_def_map(
            name_spur,
            entry_file_path,
            &mut self.file_cache,
            &mut self.global_item_tree,
            self.def_maps.clone(),
            dependencies,
            &INTERNER,
            &BasicFileResolver,
        );

        let (bodies, hir_body_diagnostics) = flux_hir::lower_def_map_bodies(
            &def_map,
            &self.global_item_tree,
            self.def_maps.clone(),
            &INTERNER,
            &mut types,
        );

        self.file_cache
            .report_diagnostics(&hir_first_pass_diagnostics);
        self.file_cache.report_diagnostics(&hir_body_diagnostics);

        println!("{}", self.fmt_def_map(&def_map, &bodies));

        let package_id = self.def_maps.alloc(Arc::new(def_map));
        self.package_name_to_id.insert(name_spur, package_id);
    }

    fn resolve_dependencies(
        &mut self,
        root_directory: &PathBuf,
        dependencies: &Dependencies,
    ) -> Result<(), Diagnostic> {
        let unresolved_deps: Vec<_> = dependencies
            .local
            .iter()
            .filter(|local_dep| {
                let path = root_directory.join(local_dep);

                let package_name = path.file_name().unwrap().to_str().unwrap();

                let exists = path.exists();
                if exists {
                    self.build_package(&path, package_name);
                }
                !exists
            })
            .cloned()
            .collect();
        if !unresolved_deps.is_empty() {
            Err(DriverError::UnresolvedDeps {
                deps: unresolved_deps,
            }
            .to_diagnostic())
        } else {
            Ok(())
        }
    }
}
