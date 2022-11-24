use std::{
    env, fs,
    path::{Path, PathBuf},
};

use diagnostics::DriverError;
use flux_diagnostics::{reporting::FileCache, Diagnostic, ToDiagnostic};
use flux_parser::parse;
use flux_span::{InFile, Span, Spanned, WithSpan};
use la_arena::Arena;
use lasso::{Spur, ThreadedRodeo};
use module_tree::ModuleData;
use once_cell::sync::Lazy;
use tracing::{info, instrument, trace, warn};

mod diagnostics;
mod module_tree;

pub static INTERNER: Lazy<ThreadedRodeo> = Lazy::new(ThreadedRodeo::new);
const ENTRY_MODULE_NAME: &str = "pkg";
const FILE_EXT: &str = ".flx";

struct Driver {
    file_cache: FileCache,
    project_dir: PathBuf,
    interner: &'static ThreadedRodeo,
    diagnostics: Vec<Diagnostic>,
    modules: Arena<ModuleData>,
}

impl Driver {
    pub fn new(project_dir: PathBuf, interner: &'static ThreadedRodeo) -> Self {
        Self {
            file_cache: FileCache::new(interner),
            project_dir,
            interner,
            diagnostics: vec![],
            modules: Arena::new(),
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
                path1.push(format!("{}{}", module_name, FILE_EXT));
                let mut path2 = PathBuf::from(dir);
                path2.push(module_name);
                path2.push(format!("{}{}", module_name, FILE_EXT));
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
            path1.push(format!("main{}", FILE_EXT));
            let mut path2 = PathBuf::from(dir);
            path2.push(module_name);
            path2.push(format!("main{}", FILE_EXT));
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
            warn!("could not read directory `{}`", project_dir_str);
            DriverError::ReadDir(format!("could not read directory `{}`", project_dir_str))
        })?;
        for entry in dir_entries {
            let entry = entry.map_err(|e| {
                warn!("could not read entry `{}`", e);
                DriverError::ReadDir(format!("could not read directory `{}`", project_dir_str))
            })?;
            let name = entry.file_name();
            if name == "main.flx" {
                info!("found entry path: {:?}", entry.path());
                return Ok(entry.path());
            }
        }
        warn!("could not find entry file `{}`", project_dir_str);
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

    #[instrument(level = "info", name = "Driver::parse_file", skip(self))]
    fn parse_module(
        &mut self,
        module_path: &[Spur],
        module_name_span: Option<InFile<Span>>,
        parent_module_dir: &Path,
    ) {
        let module_name = self.interner.resolve(module_path.last().unwrap());
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
                return;
            }
        };

        let file_id = self
            .file_cache
            .add_file(file_path.to_str().unwrap(), &file_content);
        let result = parse(&file_content, file_id, &INTERNER);
        let (root, diagnostics) = (result.syntax(), result.diagnostics);
        self.file_cache.report_diagnostics(&diagnostics);
        info!(diagnostics = diagnostics.len(), "parsed file");

        // let module_data = ModuleData::new()

        flux_hir::test(file_id, root, &INTERNER);

        // let module_name = self.interner.resolve(module_path.last().unwrap());
        // let parent_module_name = if module_path.len() >= 2 {
        //     module_path
        //         .get(module_path.len() - 2)
        //         .map(|spur| self.interner.resolve(spur))
        // } else {
        //     None
        // };
        // let (path1, path2) =
        //     self.possible_submodule_paths(module_name, parent_module_dir, &parent_module_name);
        // let file_content_result =
        //     self.get_module_file_content(module_name, module_name_span, &path1, &path2);
        // let (file_content, file_path) = match file_content_result {
        //     Ok(content) => content,
        //     Err(diagnostic) => {
        //         self.diagnostics.push(diagnostic.to_diagnostic());
        //         return;
        //     }
        // };

        // let file_id = self
        //     .file_cache
        //     .add_file(file_path.to_str().unwrap(), &file_content);
        // let result = parse(&file_content, file_id, &INTERNER);
        // let (root, diagnostics) = (result.syntax(), result.diagnostics);
        // self.file_cache.report_diagnostics(&diagnostics);
        // info!(diagnostics = diagnostics.len(), "parsed file");
        // let lowered_module = lower_items(root, file_id, self.interner, &mut self.item_tree);

        // let cur_dir = file_path.parent().unwrap();
        // for mod_idx in &lowered_module.mods {
        //     let m = &self.item_tree.mods[*mod_idx];
        //     let (_visibility, module_name) = &m.0;
        //     let name = self.interner.resolve(module_name);
        //     self.parse_module(
        //         &[module_path, &[self.interner.get_or_intern(name)]].concat(),
        //         Some(InFile::new(module_name.span, file_id)),
        //         cur_dir,
        //     );
        // }

        // let module_path = spur_iter_to_spur(module_path.iter(), self.interner);
        // self.module_tree.insert(module_path, lowered_module);
        // self.path_to_fileid.insert(module_path, file_id);
        // debug!(
        //     module_path = self.interner.resolve(&module_path),
        //     "inserting module to tree"
        // );
    }

    // fn hir_lowering_second_pass(&self) {
    //     for (path, module) in &self.module_tree {
    //         let file_id = self.path_to_fileid[path];
    //     }
    // }

    fn build(&mut self) {
        let entry_path = match self.find_entry_file() {
            Ok(path) => path,
            Err(err) => return self.file_cache.report_diagnostic(&err.to_diagnostic()),
        };
        self.parse_module(
            &[self.interner.get_or_intern_static(ENTRY_MODULE_NAME)],
            None,
            entry_path.parent().unwrap(),
        );
        // self.file_cache.report_diagnostics(&self.diagnostics);
        // self.hir_lowering_second_pass();
    }
}

pub fn build() {
    let args: Vec<_> = env::args().collect();
    let path = if args.len() > 1 {
        let mut buf = PathBuf::new();
        buf.push(&args[1]);
        buf
    } else {
        env::current_dir().unwrap()
    };
    let mut driver = Driver::new(path, &INTERNER);
    driver.build();
}

/*

    "foo": {
        submodules: [

        ],
        functions: [idx1, idx2, idx3],
        structs: [idx4],
    },
    "bar": {
        submodules: [
            "foo": {
                submodules: [],
                functions: [idx5, idx6]
            }
        ],
        functions: [],
        structs: []
    }
]

ModuleTree: Vec<HashMap<Spur, Module>>,
Module: Struct {
    submodules: Vec<Module>,
    functions: Vec<Idx<Function>>,
    structs: Vec<Idx<Struct>,
    // etc...
}

*/
