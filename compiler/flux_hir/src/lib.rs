use cfg::Config;
use flux_diagnostics::{Diagnostic, SourceCache};
use flux_span::{FileId, Interner};
use name_res::BasicFileResolver;
use pkg::{Package, PkgBuilder};

mod body;
pub mod cfg;
mod diagnostics;
mod hir;
mod item;
mod item_scope;
pub(crate) mod item_tree;
mod module;
mod name_res;
mod pkg;
#[cfg(test)]
mod tests;

const POISONED_NAME: &'static str = "<poisoned>";

pub fn lower_package(
    entry_file_id: FileId,
    entry_src: &str,
    interner: &'static Interner,
    source_cache: &mut SourceCache,
    config: &Config,
) -> (Package, Vec<Diagnostic>) {
    let mut pkg_builder = PkgBuilder::new(interner, source_cache, config, BasicFileResolver);
    pkg_builder.seed_with_entry(entry_file_id, entry_src);
    let (pkg, diagnostics) = pkg_builder.finish();
    // println!("{:#?}", pkg);
    if config.debug_item_tree {
        println!("{}", pkg.to_pretty(10, interner, &pkg.tenv));
    }
    (pkg, diagnostics)
}
