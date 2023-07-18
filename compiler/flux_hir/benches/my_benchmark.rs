use std::path::{Path, PathBuf};

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use flux_diagnostics::reporting::FileCache;
use flux_hir::{BasicFileResolver, FileResolver, ItemTree, RelativePath};
use la_arena::Arena;
use lasso::ThreadedRodeo;
use once_cell::sync::Lazy;

static INTERNER: Lazy<ThreadedRodeo> = Lazy::new(ThreadedRodeo::new);

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

// fn thir(file: &str) {
//     let name_spur = INTERNER.get_or_intern(file);
//     let content = &std::fs::read_to_string(file).unwrap();

//     let mut file_cache = FileCache::new(&INTERNER);
//     file_cache.add_file(file, content);
//     let mut global_item_tree = ItemTree::default();
//     let def_maps = Arena::new();
//     let dependencies = vec![];

//     let (def_map, mut types, _hir_first_pass_diagnostics) = flux_hir::build_def_map(
//         name_spur,
//         file,
//         &mut file_cache,
//         &mut global_item_tree,
//         def_maps.clone(),
//         dependencies,
//         &INTERNER,
//         &TestFileResolver,
//     );

//     let (_bodies, _hir_body_diagnostics) = flux_hir::lower_def_map_bodies(
//         &def_map,
//         &global_item_tree,
//         def_maps.clone(),
//         &INTERNER,
//         &mut types,
//     );
// }

fn check(content: &str) {
    let mut file_cache = FileCache::new(&INTERNER);
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
    let def_maps = Arena::new();
    let mut global_item_tree = ItemTree::default();

    let (def_map, mut types, _diagnostics) = match entry_file_path {
        Some(entry_path) => flux_hir::build_def_map(
            INTERNER.get_or_intern_static("test"),
            entry_path,
            &mut file_cache,
            &mut global_item_tree,
            def_maps.clone(),
            vec![],
            &INTERNER,
            &TestFileResolver,
        ),
        None => panic!("malformated input to `check` function in name resolution unit test"),
    };

    let (_bodies, _diagnostics2) = flux_hir::lower_def_map_bodies(
        &def_map,
        &global_item_tree,
        def_maps.clone(),
        &INTERNER,
        &mut types,
    );

    // diagnostics.append(&mut diagnostics2);
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("10000 simple trait applications", |b| {
        b.iter(|| check(black_box(include_str!("./bench.flx"))))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
