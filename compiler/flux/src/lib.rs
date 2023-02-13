use std::{
    collections::HashMap,
    env, fs,
    path::{Path, PathBuf},
};

use diagnostics::DriverError;
use flux_diagnostics::{reporting::FileCache, Diagnostic, ToDiagnostic};
use flux_hir::{
    hir::{FunctionId, Module, ModuleId, StructId, TraitId},
    lower_ast_to_hir, lower_hir_item_bodies, TypeInterner,
};
use flux_parser::parse;
use flux_span::{InFile, Span, Spanned, WithSpan};
use itertools::Itertools;
use la_arena::Arena;
use lasso::{Spur, ThreadedRodeo};
use once_cell::sync::Lazy;
use tracing::{debug, info, instrument, trace, warn};

mod diagnostics;

pub static STRING_INTERNER: Lazy<ThreadedRodeo> = Lazy::new(ThreadedRodeo::new);
pub static TYPE_INTERNER: Lazy<TypeInterner> = Lazy::new(|| TypeInterner::new(&STRING_INTERNER));
const ENTRY_MODULE_NAME: &str = "pkg";
const FILE_EXT: &str = ".flx";

struct Driver {
    file_cache: FileCache,
    project_dir: PathBuf,
    interner: &'static ThreadedRodeo,
    diagnostics: Vec<Diagnostic>,
    modules: Arena<Module>,
    mod_namespace: HashMap<Spur, ModuleId>,
    function_namespace: HashMap<Spur, (FunctionId, ModuleId)>,
    struct_namespace: HashMap<Spur, (StructId, ModuleId)>,
    trait_namespace: HashMap<Spur, (TraitId, ModuleId)>,
}

impl Driver {
    pub fn new(project_dir: PathBuf, interner: &'static ThreadedRodeo) -> Self {
        Self {
            file_cache: FileCache::new(interner),
            project_dir,
            interner,
            diagnostics: vec![],
            modules: Arena::new(),
            function_namespace: HashMap::new(),
            mod_namespace: HashMap::new(),
            struct_namespace: HashMap::new(),
            trait_namespace: HashMap::new(),
        }
    }

    fn possible_submodule_paths(
        &self,
        module_name: &str,
        dir: &Path,
        parent_mod_name: &Option<&str>,
    ) -> (PathBuf, PathBuf) {
        if let Some(parent_mod_name) = parent_mod_name {
            if *parent_mod_name == ENTRY_MODULE_NAME {
                let mut path1 = PathBuf::from(dir);
                path1.push(format!("{module_name}{FILE_EXT}"));
                let mut path2 = PathBuf::from(dir);
                path2.push(module_name);
                path2.push(format!("{module_name}{FILE_EXT}"));
                (path1, path2)
            } else {
                let mut path1 = PathBuf::from(dir);
                path1.push(parent_mod_name);
                path1.push(module_name.to_string() + FILE_EXT);
                let mut path2 = PathBuf::from(dir);
                path2.push(parent_mod_name);
                path2.push(module_name);
                path2.push(module_name.to_string() + FILE_EXT);
                (path1, path2)
            }
        } else {
            let mut path1 = PathBuf::from(dir);
            path1.push(format!("main{FILE_EXT}"));
            let mut path2 = PathBuf::from(dir);
            path2.push(module_name);
            path2.push(format!("main{FILE_EXT}"));
            (path1, path2)
        }
    }

    #[instrument(
        level = "info",
        name = "Driver::find_entry_file",
        skip(self),
        fields(
            project_dir = %self.project_dir.to_str().unwrap()
        ),
    )]
    fn find_entry_file(&self) -> Result<PathBuf, DriverError> {
        let project_dir_str = self.project_dir.to_str().unwrap();
        let dir_entries = fs::read_dir(&self.project_dir).map_err(|_| {
            warn!("could not read directory `{project_dir_str}`");
            DriverError::ReadDir(format!("could not read directory `{project_dir_str}`"))
        })?;
        for entry in dir_entries {
            let entry = entry.map_err(|e| {
                warn!("could not read entry `{}`", e);
                DriverError::ReadDir(format!("could not read directory `{project_dir_str}`"))
            })?;
            let name = entry.file_name();
            if name == "main.flx" {
                info!("found entry path: {:?}", entry.path());
                return Ok(entry.path());
            }
        }
        warn!("could not find entry file `{project_dir_str}`");
        Err(DriverError::FindEntryFile)
    }

    #[instrument(
        level = "trace",
        name = "Driver::find_submodule_path",
        skip(self, module_name)
    )]
    fn find_submodule_path(
        &self,
        parent_path: &Path,
        module_name: InFile<Spanned<&str>>,
    ) -> Result<PathBuf, DriverError> {
        let mut path = PathBuf::from(parent_path.parent().unwrap());
        let module_file_name = module_name.to_string() + FILE_EXT;
        path.push(&module_file_name);
        trace!(path = ?path, "trying path");
        if Path::exists(&path) {
            trace!(module = module_name.inner.inner, path = ?path, "found submodule path");
            return Ok(path);
        }
        let path1 = path.to_str().unwrap().to_string();
        path.pop();
        path.push(module_name.inner.inner);
        path.push(&module_file_name);
        trace!("trying path {:?}", path);
        if Path::exists(&path) {
            trace!(module = module_name.inner.inner, path = ?path, "found submodule path");
            return Ok(path);
        }
        warn!("could not find submodule path");
        Err(DriverError::FindSubmodule {
            submodule: module_name.map_inner(|s| s.to_owned()),
            path1,
            path2: path.to_str().unwrap().to_string(),
        })
    }

    fn get_module_file_content(
        &self,
        module_name: &str,
        module_name_span: Option<InFile<Span>>,
        path1: &Path,
        path2: &Path,
    ) -> Result<(String, PathBuf), DriverError> {
        trace!(path1 = ?path1, path2 = ?path2, "checking paths");
        match fs::read_to_string(path1) {
            Ok(s) => Ok((s, PathBuf::from(path1))),
            Err(_) => match fs::read_to_string(path2) {
                Ok(s) => Ok((s, PathBuf::from(path2))),
                Err(_) => {
                    warn!("could not find module");
                    match module_name_span {
                        Some(span) => Err(DriverError::FindSubmodule {
                            submodule: module_name.to_string().in_file(span.file_id, span.inner),
                            path1: path1.to_str().unwrap().to_string(),
                            path2: path2.to_str().unwrap().to_string(),
                        }),
                        None => Err(DriverError::FindEntryFile),
                    }
                }
            },
        }
    }

    // #[instrument(level = "info", name = "Driver::parse_file", skip(self))]
    // fn parse_module(
    //     &mut self,
    //     module_path: &[Spur],
    //     module_name_span: Option<InFile<Span>>,
    //     parent_module_dir: &Path,
    //     parent_module_id: Option<ModuleID>,
    // ) -> Option<ModuleID> {
    //     let module_name = self.interner.resolve(
    //         module_path
    //             .last()
    //             .expect("internal compiler error: module path should never be empty"),
    //     );
    //     let parent_module_name = if module_path.len() >= 2 {
    //         module_path
    //             .get(module_path.len() - 2)
    //             .map(|spur| self.interner.resolve(spur))
    //     } else {
    //         None
    //     };
    //     let (path1, path2) =
    //         self.possible_submodule_paths(module_name, parent_module_dir, &parent_module_name);
    //     let file_content_result =
    //         self.get_module_file_content(module_name, module_name_span, &path1, &path2);
    //     let (file_content, file_path) = match file_content_result {
    //         Ok(content) => content,
    //         Err(diagnostic) => {
    //             self.diagnostics.push(diagnostic.to_diagnostic());
    //             return None;
    //         }
    //     };

    //     let module_path_str = module_path
    //         .iter()
    //         .map(|spur| self.interner.resolve(spur))
    //         .join("::");
    //     let file_id = self
    //         .file_cache
    //         .add_file(file_path.to_str().unwrap(), &file_content);
    //     let result = parse(&file_content, file_id, &INTERNER);
    //     let (root, diagnostics) = (result.syntax(), result.diagnostics);
    //     self.file_cache.report_diagnostics(&diagnostics);
    //     debug!(module_path = module_path_str, "parsed file");

    //     let hir_module = lower_ast_to_hir(root, self.interner);
    //     debug!(module_path = module_path_str, "generated HIR module");

    //     let module_name_spur = self.interner.get_or_intern(module_name);
    //     let module_id = if module_path.len() == 1 {
    //         self.package_defs
    //             .add_root_module(hir_module, module_name_spur, file_id)
    //     } else {
    //         self.package_defs.add_module(
    //             hir_module,
    //             module_name_spur,
    //             parent_module_id
    //                 .expect("internal compiler error: parent module id should not be None"),
    //             file_id,
    //         )
    //     };

    //     let cur_dir = file_path.parent().unwrap();
    //     let module = self.package_defs.get_item_scope(module_id);
    //     for (_, m) in module.mods.clone().iter() {
    //         let m = match m {
    //             ItemDefinitionId::ModuleId(mod_id) => *mod_id,
    //             _ => unreachable!(),
    //         };
    //         let m = self.package_defs.get_module_data(module_id).module.mods[m].clone();
    //         let module_name = &m.name;
    //         let name = self.interner.resolve(module_name);
    //         let child_module_id = self.parse_module(
    //             &[module_path, &[self.interner.get_or_intern(name)]].concat(),
    //             Some(InFile::new(module_name.span, file_id)),
    //             cur_dir,
    //             Some(module_id),
    //         );
    //         if let Some(child_module_id) = child_module_id {
    //             self.package_defs
    //                 .add_child_module(module_id, module_name.inner, child_module_id);
    //         }
    //     }
    //     Some(module_id)
    // }

    #[instrument(level = "info", name = "Driver::parse_file", skip(self))]
    fn parse_module(
        &mut self,
        module_path: &[Spur],
        module_name_span: Option<InFile<Span>>,
        parent_module_dir: &Path,
        parent_module_id: Option<ModuleId>,
    ) -> Option<ModuleId> {
        let module_name = self.interner.resolve(
            module_path
                .last()
                .expect("internal compiler error: module path should never be empty"),
        );
        let parent_module_name = if module_path.len() >= 2 {
            module_path
                .get(module_path.len() - 2)
                .map(|spur| self.interner.resolve(spur))
        } else {
            None
        };
        let (path1, path2) =
            self.possible_submodule_paths(module_name, parent_module_dir, &parent_module_name);
        let file_content_result =
            self.get_module_file_content(module_name, module_name_span, &path1, &path2);
        let (file_content, file_path) = match file_content_result {
            Ok(content) => content,
            Err(diagnostic) => {
                self.diagnostics.push(diagnostic.to_diagnostic());
                return None;
            }
        };

        let module_path_str = module_path
            .iter()
            .map(|spur| self.interner.resolve(spur))
            .join("::");
        let file_id = self
            .file_cache
            .add_file(file_path.to_str().unwrap(), &file_content);
        let result = parse(&file_content, file_id, &STRING_INTERNER);
        let (root, diagnostics) = (result.syntax(), result.diagnostics);
        self.file_cache.report_diagnostics(&diagnostics);
        debug!(module_path = module_path_str, "parsed file");

        let module_id = self
            .modules
            .alloc(Module::new(file_id, module_path.to_vec()));
        let (hir_module, diagnostics) = lower_ast_to_hir(
            root,
            module_path.to_vec(),
            module_id,
            self.interner,
            &TYPE_INTERNER,
            &mut self.mod_namespace,
            &mut self.function_namespace,
            &mut self.struct_namespace,
            &mut self.trait_namespace,
            file_id,
        );
        self.file_cache.report_diagnostics(&diagnostics);
        self.modules[module_id] = hir_module;
        debug!(module_path = module_path_str, "generated HIR module");

        let cur_dir = file_path.parent().unwrap();
        for (_, m) in self.modules[module_id].mods.clone().iter() {
            let module_name = &m.name;
            self.parse_module(
                &[module_path, &[module_name.inner]].concat(),
                Some(InFile::new(module_name.span, file_id)),
                cur_dir,
                Some(module_id),
            );
        }
        Some(module_id)
    }

    fn build(&mut self) {
        let entry_path = match self.find_entry_file() {
            Ok(path) => path,
            Err(err) => return self.file_cache.report_diagnostic(&err.to_diagnostic()),
        };
        self.parse_module(
            &[self.interner.get_or_intern_static(ENTRY_MODULE_NAME)],
            None,
            entry_path.parent().unwrap(),
            None,
        );
        self.file_cache.report_diagnostics(&self.diagnostics);

        for (path, module_id) in &self.mod_namespace {
            let path = self.interner.resolve(path);
            println!("{path} -> Module({})", module_id.into_raw());
        }

        for (path, (function_id, module_id)) in &self.function_namespace {
            let path = self.interner.resolve(path);
            println!(
                "{path} -> Function({}) in Module({})",
                function_id.into_raw(),
                module_id.into_raw()
            );
        }

        for (path, (struct_id, module_id)) in &self.struct_namespace {
            let path = self.interner.resolve(path);
            println!(
                "{path} -> Struct({}) in Module({})",
                struct_id.into_raw(),
                module_id.into_raw()
            );
        }

        // for (_, module) in self.modules.iter_mut() {
        let diagnostics = lower_hir_item_bodies(
            self.interner,
            &TYPE_INTERNER,
            &mut self.modules,
            &self.function_namespace,
            &self.struct_namespace,
            &self.trait_namespace,
        );
        self.file_cache.report_diagnostics(&diagnostics);
        // }
    }
}

pub fn build() {
    let args: Vec<_> = env::args().collect();
    let project_dir = if args.len() > 1 {
        let mut buf = PathBuf::new();
        buf.push(&args[1]);
        buf
    } else {
        env::current_dir().unwrap()
    };
    let mut driver = Driver::new(project_dir, &STRING_INTERNER);
    driver.build();
}
