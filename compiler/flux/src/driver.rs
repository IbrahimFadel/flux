use std::{io::BufWriter, path::PathBuf};

use flux_diagnostics::{reporting::FileCache, Diagnostic, ToDiagnostic};
use flux_hir::{DefMap, LoweredBodies};
use pretty::BoxAllocator;

use crate::{cfg::Config, diagnostics::DriverError, INTERNER};

pub(crate) struct Driver {
    pub(crate) cfg: Config,
    pub(crate) root_directory: PathBuf,
    pub(crate) file_cache: FileCache,
}

impl Driver {
    pub fn new(cfg: Config, root_directory: PathBuf) -> Self {
        Self {
            cfg,
            root_directory,
            file_cache: FileCache::new(&INTERNER),
        }
    }

    pub(crate) fn get_package_entry_file_content(
        &self,
        package_root: &PathBuf,
        package_name: &str,
    ) -> Result<(String, PathBuf), Diagnostic> {
        let file_path = package_root.join("src/main.flx");
        std::fs::read_to_string(&file_path)
            .map_err(|_error| {
                DriverError::ReadEntryFile {
                    package: package_name.to_string(),
                    candidate: file_path.to_str().unwrap().to_string(),
                }
                .to_diagnostic()
            })
            .and_then(|content| Ok((content, file_path)))
    }

    pub(crate) fn fmt_def_map(&self, def_map: &DefMap, lowered_bodies: &LoweredBodies) -> String {
        let mut buf = BufWriter::new(Vec::new());
        let allocator = BoxAllocator;
        def_map
            .pretty::<_, ()>(&allocator, &INTERNER, &lowered_bodies)
            .1
            .render(50, &mut buf)
            .unwrap();
        let bytes = buf.into_inner().unwrap();
        String::from_utf8(bytes).unwrap()
    }
}
