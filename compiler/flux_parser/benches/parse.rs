use std::path::Path;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use flux_diagnostics::reporting::FileCache;
use lasso::ThreadedRodeo;
use once_cell::sync::Lazy;

static INTERNER: Lazy<ThreadedRodeo> = Lazy::new(ThreadedRodeo::new);

fn check(path: &str) {
    println!("{:?}", Path::new("./").canonicalize());
    let mut file_cache = FileCache::new(&INTERNER);
    let content = &std::fs::read_to_string(path).unwrap();
    let file_id = file_cache.add_file(path, content);

    let parse = flux_parser::parse(content, file_id, &INTERNER);
    let (_root, _diagnostics) = (parse.syntax(), parse.diagnostics);
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("parse 10000 simple trait applications", |b| {
        b.iter(|| check(black_box("compiler/flux_parser/benches/bench.flx")))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
