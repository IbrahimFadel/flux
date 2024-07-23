use std::path::{Path, PathBuf};

use flux_diagnostics::{ice, IOError, SourceCache};
use flux_hir::pkg::{Package, PackageId};
use flux_span::Interner;
use la_arena::Arena;
use resolve_path::PathResolveExt;

use crate::{get_config, get_package_entry_file_path, ExitStatus};

pub(crate) struct Driver {
    project_root: PathBuf,
    compilation_config: flux_hir::cfg::Config,
    interner: &'static Interner,
    source_cache: SourceCache,
    packages: Arena<Package>,
}

impl Driver {
    pub(crate) fn new(
        project_root: PathBuf,
        compilation_config: flux_hir::cfg::Config,
        interner: &'static Interner,
    ) -> Self {
        Self {
            project_root,
            compilation_config,
            interner,
            source_cache: SourceCache::new(interner),
            packages: Arena::new(),
        }
    }

    pub(crate) fn build_project(&mut self, flux_config: &flux_cfg::Config) -> ExitStatus {
        let single_package_project = flux_config.packages.len() == 1;
        for package in &flux_config.packages {
            let package_root = if single_package_project {
                self.project_root.clone()
            } else {
                self.project_root.join(&package.name)
            };

            let package_id = self
                .build_package(&package_root.resolve(), &package.name)
                .map_err(|errs| errs.into_iter().for_each(|err| err.report()))
                .ok();

            if let Some(package_id) = package_id {
                let pkg = &self.packages[package_id];

                if self.compilation_config.debug_item_tree {
                    println!("{}", pkg.to_pretty(10, self.interner))
                }
            }
        }

        ExitStatus::Success
    }

    fn build_package(
        &mut self,
        package_root: &Path,
        name: &str,
    ) -> Result<PackageId, Vec<IOError>> {
        tracing::debug!(package_root =? package_root, "building package");

        let flux_config = match get_config(package_root) {
            Err(err) => {
                return Err(vec![err]);
            }
            Ok(cfg) => cfg,
        };

        self.build_dependencies(&flux_config);

        let (entry_path, content) = match get_package_entry_file_path(package_root, name) {
            Ok(x) => x,
            Err(err) => {
                return Err(vec![err]);
            }
        };

        let mut diagnostics = vec![];
        let file_id = self
            .source_cache
            .add_input_file(&entry_path, content.clone());

        let pkg = flux_hir::package_definitions(
            self.interner.get_or_intern(name),
            file_id,
            &content,
            &mut diagnostics,
            &self.compilation_config,
            self.interner,
            &mut self.source_cache,
        );

        self.source_cache.report_diagnostics(diagnostics.iter());

        Ok(self.packages.alloc(pkg))
    }

    fn build_dependencies(&mut self, flux_config: &flux_cfg::Config) {
        for (name, dependency) in flux_config.dependencies.iter() {
            let path = dependency.path.as_ref().unwrap_or_else(|| ice("ruh roh"));
            let package_root = self.project_root.join(path);
            if let Err(errs) = self.build_package(&package_root, name) {
                errs.into_iter()
                    .for_each(|err| eprintln!("{}", err.to_string()));
            }
        }
    }
}
