use std::fs;

use flux_diagnostics::reporting::FileCache;
use flux_hir::lower_to_hir;
use flux_parser::parse;
use lasso::ThreadedRodeo;
use once_cell::sync::Lazy;

static INTERNER: Lazy<ThreadedRodeo> = Lazy::new(ThreadedRodeo::new);

fn main() {
    let path = "examples/main.flx";
    let src = fs::read_to_string(path).unwrap();
    let mut file_cache = FileCache::new(&INTERNER);
    let file_id = file_cache.add_file(path, &src);
    let result = parse(&src, file_id, &INTERNER);
    let (root, diagnostics) = (result.syntax(), result.diagnostics);
    file_cache.report_diagnostics(&diagnostics);
    println!("{}", root.debug(&*INTERNER, true));
    let (module, diagnostics) = lower_to_hir(root, file_id, &INTERNER);
    file_cache.report_diagnostics(&diagnostics);
}
