#![feature(more_qualified_paths)]

use std::{env, fs, path::PathBuf};

use diagnostics::DriverError;
use flux_diagnostics::{reporting::FileCache, ToDiagnostic};
use flux_hir::BasicFileResolver;
use lasso::ThreadedRodeo;
use once_cell::sync::Lazy;

mod diagnostics;
#[cfg(test)]
mod tests;

pub static STRING_INTERNER: Lazy<ThreadedRodeo> = Lazy::new(ThreadedRodeo::new);

struct Driver {
    file_cache: FileCache,
    project_dir: PathBuf,
}

impl Driver {
    pub fn new(project_dir: PathBuf, interner: &'static ThreadedRodeo) -> Self {
        Self {
            file_cache: FileCache::new(interner),
            project_dir,
        }
    }

    fn find_entry_file(&self) -> Result<PathBuf, DriverError> {
        let project_dir_str = self.project_dir.to_str().unwrap();
        let dir_entries = fs::read_dir(&self.project_dir).map_err(|_| {
            tracing::warn!("could not read directory `{project_dir_str}`");
            DriverError::ReadDir(format!("could not read directory `{project_dir_str}`"))
        })?;
        for entry in dir_entries {
            let entry = entry.map_err(|e| {
                tracing::warn!("could not read entry `{}`", e);
                DriverError::ReadDir(format!("could not read directory `{project_dir_str}`"))
            })?;
            let name = entry.file_name();
            if name == "main.flx" {
                tracing::info!("found entry path: {:?}", entry.path());
                return Ok(entry.path());
            }
        }
        tracing::warn!("could not find entry file `{project_dir_str}`");
        Err(DriverError::FindEntryFile)
    }

    fn build(&mut self) {
        let entry_path = match self.find_entry_file() {
            Ok(path) => path,
            Err(err) => return self.file_cache.report_diagnostic(&err.to_diagnostic()),
        };
        let (def_map, mut types, diagnostics) = flux_hir::build_def_map(
            entry_path.to_str().unwrap(),
            &mut self.file_cache,
            &STRING_INTERNER,
            &BasicFileResolver,
        );
        self.file_cache.report_diagnostics(&diagnostics);

        let (_lowered_bodies, diagnostics) =
            flux_hir::lower_def_map_bodies(&def_map, &STRING_INTERNER, &mut types);
        self.file_cache.report_diagnostics(&diagnostics);
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
