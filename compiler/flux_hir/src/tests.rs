use std::sync::OnceLock;

use flux_diagnostics::{Diagnostic, SourceCache};
use flux_span::Interner;

use crate::{
    cfg::Config,
    name_res::{FileResolver, RelativePath},
    pkg::{Package, PkgBuilder},
};

mod basic;

static INTERNER: OnceLock<Interner> = OnceLock::new();

fn check(content: &str) -> (Package, Vec<Diagnostic>) {
    let interner = INTERNER.get_or_init(|| Interner::new());
    let mut source_cache = SourceCache::new(interner);
    let config = Config { debug_cst: false };

    let files = content.split("//-").skip(1);
    let mut entry_file = None;
    for (i, file) in files.enumerate() {
        let newline_loc = file
            .find("\n")
            .expect("malformated input to `check` function in hir lowering unit test");
        let file_path = &file[..newline_loc];
        let content = &file[newline_loc..];

        let file_id = source_cache.add_input_file(file_path, content.to_string());
        if i == 0 {
            entry_file = Some(file_id);
        }
    }

    let (file_id, src) = match entry_file {
        Some(entry) => (entry, source_cache.get_file_content(&entry)),
        None => panic!("malformated input to `check` function in hir lowering unit test"),
    };
    let mut pkg_builder = PkgBuilder::new(interner, &mut source_cache, &config, TestFileResolver);
    pkg_builder.seed_with_entry(file_id, &src);
    pkg_builder.finish()
}

fn fmt_pkg(pkg: &Package) -> String {
    let interner = INTERNER.get().unwrap();
    pkg.to_pretty(10, interner)
}

#[macro_export]
macro_rules! no_errors {
    ($name:ident, $src:literal) => {
        paste::paste! {
            #[test]
            fn [<$name>]() {
                let (_, diagnostics) = crate::tests::check($src);
                assert!(diagnostics.is_empty());
            }
        }
    };
}

#[macro_export]
macro_rules! errors {
    ($name:ident, $src:literal) => {
        paste::paste! {
            #[test]
            fn [<$name>]() {
                let (pkg, diagnostics) = crate::tests::check($src);
                assert!(!diagnostics.is_empty());
                let s = crate::tests::fmt_pkg(&pkg);
                insta::with_settings!({sort_maps =>true}, {
                    insta::assert_snapshot!(s);
                });
            }
        }
    };
}

struct TestFileResolver;

impl FileResolver for TestFileResolver {
    fn resolve_absolute_path(
        &self,
        path: &str,
        source_cache: &mut SourceCache,
    ) -> Option<(flux_span::FileId, String)> {
        Some(source_cache.get_by_file_path(path))
    }

    fn resolve_relative_path(
        &self,
        path: RelativePath,
        source_cache: &mut SourceCache,
    ) -> Option<(flux_span::FileId, String)> {
        let anchor_path = source_cache.get_file_dir(&path.anchor);
        let absolute_path = format!("{anchor_path}/{}", path.path);
        Some(source_cache.get_by_file_path(&absolute_path))
    }
}
