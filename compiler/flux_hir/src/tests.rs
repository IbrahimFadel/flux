use std::io::{BufWriter, Write};

use flux_diagnostics::{reporting::FileCache, Diagnostic};
use lasso::ThreadedRodeo;
use once_cell::sync::Lazy;
use pretty::BoxAllocator;

use crate::{
    body::LoweredBodies,
    name_res::{
        mod_res::{FileResolver, RelativePath},
        DefMap,
    },
};

mod enums;
mod generics;
mod resolution;
mod structs;

static STRING_INTERNER: Lazy<ThreadedRodeo> = Lazy::new(ThreadedRodeo::new);

struct TestFileResolver;

impl FileResolver for TestFileResolver {
    fn resolve_absolute_path(
        &self,
        path: &str,
        file_cache: &mut FileCache,
    ) -> Option<(flux_span::FileId, String)> {
        Some(file_cache.get_by_file_path(path))
    }

    fn resolve_relative_path(
        &self,
        path: RelativePath,
        file_cache: &mut flux_diagnostics::reporting::FileCache,
    ) -> Option<(flux_span::FileId, String)> {
        let anchor_path = file_cache.get_file_dir(&path.anchor);
        let absolute_path = format!("{anchor_path}/{}", path.path);
        Some(file_cache.get_by_file_path(&absolute_path))
    }
}

fn check(content: &str) -> (DefMap, LoweredBodies, Vec<Diagnostic>, FileCache) {
    let mut file_cache = FileCache::new(&STRING_INTERNER);
    let files = content.split("//-").skip(1);
    let mut entry_file_path = None;
    for (i, file) in files.enumerate() {
        let newline_loc = file
            .find("\n")
            .expect("malformated input to `check` function in name resolution unit test");
        let file_path = &file[..newline_loc];
        let content = &file[newline_loc..];
        file_cache.add_file(file_path, content);
        if i == 0 {
            entry_file_path = Some(file_path);
        }
    }
    let (def_map, mut types, mut diagnostics) = match entry_file_path {
        Some(entry_path) => crate::build_def_map(
            entry_path,
            &mut file_cache,
            &STRING_INTERNER,
            &TestFileResolver,
        ),
        None => panic!("malformated input to `check` function in name resolution unit test"),
    };
    let (lowered_bodies, mut diagnostics2) =
        crate::lower_def_map_bodies(&def_map, &STRING_INTERNER, &mut types);
    diagnostics.append(&mut diagnostics2);
    (def_map, lowered_bodies, diagnostics, file_cache)
}

fn fmt_def_map(
    def_map: &DefMap,
    lowered_bodies: &LoweredBodies,
    diagnostics: &[Diagnostic],
    file_cache: &FileCache,
) -> String {
    let mut buf: BufWriter<Vec<u8>> = BufWriter::new(Vec::new());
    let allocator = BoxAllocator;
    def_map
        .pretty::<_, ()>(&allocator, &STRING_INTERNER, lowered_bodies)
        .1
        .render(50, &mut buf)
        .unwrap();

    buf.write("\n\nDiagnostics\n\n".as_bytes()).unwrap();
    file_cache.write_diagnostics_to_buffer(diagnostics, &mut buf);
    let bytes: Vec<u8> = buf.into_inner().unwrap();
    let diagnostics_bytes_without_ansi = strip_ansi_escapes::strip(&bytes).unwrap();
    String::from_utf8(diagnostics_bytes_without_ansi).unwrap()
}

#[macro_export]
macro_rules! no_errors {
    ($name:ident, $src:literal) => {
        paste::paste! {
            #[test]
            fn [<$name>]() {
                let (_def_map, _lowered_bodies, diagnostics, _file_cache) = crate::tests::check($src);
                assert_eq!(diagnostics.len(), 0);
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
                let (def_map, lowered_bodies, diagnostics, file_cache) = crate::tests::check($src);
                assert_ne!(diagnostics.len(), 0);
                let s = crate::tests::fmt_def_map(&def_map, &lowered_bodies, &diagnostics, &file_cache);
                insta::assert_snapshot!(s);
            }
        }
    };
}
