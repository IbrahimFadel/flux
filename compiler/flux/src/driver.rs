use std::path::{Component, Path, PathBuf};

use flux_diagnostics::{Diagnostic, IOError, SourceCache};
use flux_hir::Package;
use flux_id::{id, Map};
use flux_util::Interner;
use tracing::info;

use crate::{
    cfg::{self, Config},
    get_config, get_package_entry_file_path, ExitStatus,
};

pub(crate) struct Driver {
    compilation_config: flux_hir::Config,
    interner: &'static Interner,
    source_cache: SourceCache,
    packages: Map<id::Pkg, Package>,
    diagnostics: Vec<Diagnostic>,
}

impl Driver {
    pub(crate) fn new(compilation_config: flux_hir::Config, interner: &'static Interner) -> Self {
        Self {
            compilation_config,
            interner,
            source_cache: SourceCache::new(interner),
            packages: Map::new(),
            diagnostics: vec![],
        }
    }

    pub(crate) fn build_project(
        &mut self,
        project_root: PathBuf,
        flux_config: &Config,
    ) -> (Vec<id::Pkg>, ExitStatus) {
        let single_package_project = flux_config.packages.len() == 1;

        let mut all_dependencies = vec![];
        let mut built_packages = vec![];
        for package in &flux_config.packages {
            let package_root = if single_package_project {
                project_root.clone()
            } else {
                project_root.join(&package.name)
            };

            if !flux_config.dependencies.map.is_empty() {
                info!(package =? package.name, "building dependencies");
            }
            let (dependencies, errors) =
                self.build_dependencies(&package_root, &flux_config.dependencies);
            all_dependencies.extend(dependencies.iter().copied());
            errors.into_iter().for_each(|err| err.report());

            info!(package =? package.name, "building package definitions");
            let package_id =
                match self.build_package_definitions(&package.name, &resolve(&package_root)) {
                    Ok(id) => id,
                    Err(err) => {
                        err.report();
                        continue;
                    }
                };
            built_packages.push(package_id);
            self.packages
                .get_mut(package_id)
                .set_dependencies(dependencies);
        }

        let mut exprs = Map::new();
        for (package_id, package) in self
            .packages
            .iter()
            .filter(|(package_id, _)| !all_dependencies.contains(package_id))
        {
            info!(package =? self.interner.resolve(&package.name), "building package bodies");
            flux_hir::build_package_bodies(
                package_id,
                &self.packages,
                &mut exprs,
                self.interner,
                &mut self.diagnostics,
            );
        }

        self.source_cache
            .report_diagnostics(self.diagnostics.iter());

        (built_packages, ExitStatus::Success)
    }

    fn build_dependencies(
        &mut self,
        project_root: &Path,
        dependencies: &cfg::Dependencies,
    ) -> (Vec<id::Pkg>, Vec<IOError>) {
        let mut built_packages = vec![];
        let mut errors = vec![];
        for (_, dependency) in dependencies.map.iter() {
            let path = dependency
                .path
                .as_ref()
                .expect("no other type of dependency than local right now");
            let path = resolve(&project_root.join(path));

            let flux_config = match get_config(&path) {
                Ok(cfg) => cfg,
                Err(diagnostic) => {
                    errors.push(diagnostic);
                    continue;
                }
            };

            let (mut packages, _) = self.build_project(path, &flux_config);
            built_packages.append(&mut packages);
        }
        (built_packages, errors)
    }

    fn build_package_definitions(
        &mut self,
        name: &str,
        package_root: &Path,
    ) -> Result<id::Pkg, IOError> {
        let (entry_path, content) = match get_package_entry_file_path(package_root, name) {
            Ok(x) => x,
            Err(err) => {
                return Err(err);
            }
        };

        let name = self.interner.get_or_intern(name);
        let file_id = self
            .source_cache
            .add_input_file(&entry_path, content.clone());

        let package = flux_hir::build_package_definitions(
            name,
            file_id,
            &content,
            &mut self.source_cache,
            self.interner,
            &mut self.diagnostics,
        );
        let package_id = self.packages.insert(package);
        Ok(package_id)
    }
}

fn resolve(path: &Path) -> PathBuf {
    let mut resolved = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => assert!(resolved.pop()),
            segment => resolved.push(segment),
        }
    }
    resolved
}
