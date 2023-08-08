use std::{
    env,
    path::{Path, PathBuf},
};

use flux_diagnostics::{reporting::FileCache, Diagnostic, ToDiagnostic};
use flux_hir::{BasicFileResolver, PackageData, PackageDependency};

use crate::{
    cfg::{parse_cfg, Dependencies, Workspace},
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
            FileCache::new(&INTERNER).report_diagnostic(&err);
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
    pub fn build(&mut self) {
        if let Some(workspace) = &self.cfg.workspace {
            self.build_multi_package_project(workspace.clone());
        } else {
            self.build_single_package_project();
        }
    }

    fn build_multi_package_project(&mut self, workspace: Workspace) {
        tracing::trace!(packages =? workspace.packages, "building multi package project");

        for package in &workspace.packages {
            let package_path = self.root_directory.join(package);
            self.build_package(&package_path, package);
        }
    }

    fn build_single_package_project(&mut self) {
        let package_cfg = self.cfg.package.as_ref().unwrap();
        tracing::trace!(package =? package_cfg, "building single package project");
        self.build_package(&self.root_directory.clone(), &package_cfg.name.to_string());
    }

    fn build_package(&mut self, root_directory: &Path, package_name: &str) {
        tracing::info!(package_name, "building package");
        let package_name_spur = INTERNER.get_or_intern(package_name);
        let cfg_file_content = match get_config(&root_directory, Some(package_name)) {
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
            .get_package_entry_file_path(&root_directory, package_name)
            .unwrap();
        let entry_file_path = entry_file_path.to_str().unwrap();

        let dependencies: Vec<_> = cfg
            .dependencies
            .local
            .iter()
            .map(|dep| {
                INTERNER.get_or_intern(Path::new(dep).file_name().unwrap().to_str().unwrap())
            })
            .filter_map(|name| {
                self.package_name_to_id
                    .get(&name)
                    .map(|package_id| PackageDependency {
                        name,
                        package_id: *package_id,
                    })
            })
            .collect();

        let (def_map, mut tenv, mut types, hir_first_pass_diagnostics) = flux_hir::build_def_map(
            package_name_spur,
            entry_file_path,
            &mut self.file_cache,
            &self.packages,
            &dependencies,
            &INTERNER,
            &BasicFileResolver,
        );

        let package = PackageData::new(package_name_spur, def_map, dependencies);
        let package_id = self.packages.alloc(package);

        let (bodies, hir_body_diagnostics) = flux_hir::lower_def_map_bodies(
            package_id,
            &self.packages,
            &mut tenv,
            &INTERNER,
            &mut types,
        );

        println!(
            "{}",
            self.fmt_package(&self.packages[package_id], &bodies, &tenv)
        );
        // println!(
        //     "{}",
        //     self.fmt_def_map(&self.packages[package_id].def_map, &bodies)
        // );

        // self.file_cache
        //     .report_diagnostics(&hir_first_pass_diagnostics);
        // self.file_cache.report_diagnostics(&hir_body_diagnostics);

        // let package_id = self.def_maps.alloc(Arc::new(def_map));
        self.package_name_to_id
            .insert(package_name_spur, package_id);
    }

    fn resolve_dependencies(
        &self,
        root_directory: &Path,
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
                    // self.build_package(&path, package_name);
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

// impl Driver {
//     fn build(&mut self) {
//         // if self.cfg.workspace.is_some() {
//         if let Some(workspace) = &self.cfg.workspace {
//             self.build_multi_package_project(&workspace.clone());
//         } else {
//             self.build_single_package_project();
//         }
//     }

//     pub(crate) fn build_single_package_project(&mut self) {
//         let package_cfg = self.cfg.package.as_ref().unwrap();
//         tracing::trace!(package =? package_cfg, "building single package project");
//         self.build_package(&self.root_directory.clone(), &package_cfg.name.to_string());
//     }

//     pub(crate) fn build_multi_package_project(&mut self, workspace_cfg: &Workspace) {
//         tracing::trace!(packages =? workspace_cfg.packages, "building multi package project");

//         for package in &workspace_cfg.packages {
//             let package_path = self.root_directory.join(package);
//             self.build_package(&package_path, package);
//         }
//     }

//     fn build_package(&mut self, root_directory: &PathBuf, name: &str) {
//         let name_spur = INTERNER.get_or_intern(name);
//         if self.package_name_to_id.get(&name_spur).is_some() {
//             return;
//         }

//         let cfg_file_content = match get_config(&root_directory, Some(name)) {
//             Err(err) => {
//                 self.file_cache.report_diagnostic(&err);
//                 return;
//             }
//             Ok(cfg) => cfg,
//         };
//         let cfg = parse_cfg(&cfg_file_content);
//         let package_cfg = cfg.package.as_ref().unwrap();
//         tracing::trace!(package =? package_cfg, "building package");

//         if let Err(err) = self.resolve_dependencies(root_directory, &cfg.dependencies) {
//             self.file_cache.report_diagnostic(&err);
//         }

//         let entry_file_path = self
//             .get_package_entry_file_path(&root_directory, &name)
//             .unwrap();
//         let entry_file_path = entry_file_path.to_str().unwrap();

//         let dependencies: Vec<_> = cfg
//             .dependencies
//             .local
//             .iter()
//             .map(|dep| Path::new(dep).file_name().unwrap().to_str().unwrap())
//             .filter_map(|name| self.package_name_to_id.get(&INTERNER.get_or_intern(name)))
//             .cloned()
//             .collect();

//         let (def_map, mut types, hir_first_pass_diagnostics) = flux_hir::build_def_map(
//             name_spur,
//             entry_file_path,
//             &mut self.file_cache,
//             &mut self.global_item_tree,
//             self.def_maps.clone(),
//             dependencies,
//             &INTERNER,
//             &BasicFileResolver,
//         );

//         let (bodies, hir_body_diagnostics) = flux_hir::lower_def_map_bodies(
//             &def_map,
//             &self.global_item_tree,
//             self.def_maps.clone(),
//             &INTERNER,
//             &mut types,
//         );

//         self.file_cache
//             .report_diagnostics(&hir_first_pass_diagnostics);
//         self.file_cache.report_diagnostics(&hir_body_diagnostics);

//         println!("{}", self.fmt_def_map(&def_map, &bodies));

//         let package_id = self.def_maps.alloc(Arc::new(def_map));
//         self.package_name_to_id.insert(name_spur, package_id);
//     }

//     fn resolve_dependencies(
//         &mut self,
//         root_directory: &PathBuf,
//         dependencies: &Dependencies,
//     ) -> Result<(), Diagnostic> {
//         let unresolved_deps: Vec<_> = dependencies
//             .local
//             .iter()
//             .filter(|local_dep| {
//                 let path = root_directory.join(local_dep);

//                 let package_name = path.file_name().unwrap().to_str().unwrap();

//                 let exists = path.exists();
//                 if exists {
//                     // self.build_package(&path, package_name);
//                 }
//                 !exists
//             })
//             .cloned()
//             .collect();
//         if !unresolved_deps.is_empty() {
//             Err(DriverError::UnresolvedDeps {
//                 deps: unresolved_deps,
//             }
//             .to_diagnostic())
//         } else {
//             Ok(())
//         }
//     }
// }
