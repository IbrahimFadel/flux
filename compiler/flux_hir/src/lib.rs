use cfg::Config;
use flux_diagnostics::{Diagnostic, SourceCache};
use flux_span::{FileId, Interner};
use la_arena::ArenaMap;
use module::{ModuleId, ModuleTree};
use name_res::{item::ItemResolver, BasicFileResolver};
use pkg::{Package, PkgBuilder};

mod body;
pub mod cfg;
mod diagnostics;
mod hir;
mod intrinsics;
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
    module_file_map: &mut ArenaMap<ModuleId, FileId>,
    config: &Config,
) -> (Package, Vec<Diagnostic>) {
    let mut pkg_builder: PkgBuilder<'_, BasicFileResolver> = PkgBuilder::new(
        interner,
        source_cache,
        config,
        BasicFileResolver,
        module_file_map,
    );
    pkg_builder.seed_with_entry(entry_file_id, entry_src);
    let (pkg, diagnostics) = pkg_builder.finish();
    if config.debug_item_tree {
        println!("{}", pkg.to_pretty(10, interner));
    }
    (pkg, diagnostics)
}

pub fn lower_package_bodies(
    package: &mut Package,
    module_file_map: &ArenaMap<ModuleId, FileId>,
    interner: &'static Interner,
) -> Vec<Diagnostic> {
    let mut diagnostics = vec![];
    for (module_id, _) in package
        .module_tree
        .get()
        .iter()
        .filter(|(module_id, _)| *module_id != ModuleTree::PRELUDE_ID)
    {
        let item_resolver = ItemResolver::new(&package.module_tree, interner);
        let file_id = module_file_map[module_id];
        let ctx = body::LowerCtx::new(
            item_resolver,
            Some(&package.item_tree),
            &mut package.tenvs[module_id],
            interner,
            file_id,
            module_id,
        );
        let mut d = ctx.lower_module_bodies();
        diagnostics.append(&mut d);
    }
    diagnostics
}
