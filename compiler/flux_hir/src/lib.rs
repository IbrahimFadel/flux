#![feature(iter_array_chunks)]

use cfg::Config;
use flux_diagnostics::{Diagnostic, SourceCache};
use flux_span::{FileId, Interner};
use flux_typesystem::TEnv;
use la_arena::Arena;
use module::ModuleTree;
use name_res::{item::ItemResolver, BasicFileResolver};
use pkg::PkgBuilder;

pub use hir::{Expr, ExprIdx, FnDecl};
pub use module::ModuleId;
pub use pkg::{PackageBodies, PackageDefs, PackageId};

mod body;
mod builtin;
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
pub(crate) mod prelude;
#[cfg(test)]
mod tests;

const POISONED_NAME: &'static str = "<poisoned>";

pub fn lower_package_defs(
    entry_file_id: FileId,
    entry_src: &str,
    interner: &'static Interner,
    source_cache: &mut SourceCache,
    config: &Config,
) -> (PackageDefs, PackageBodies, TEnv, Vec<Diagnostic>) {
    let mut pkg_builder: PkgBuilder<'_, BasicFileResolver> =
        PkgBuilder::new(interner, source_cache, config, BasicFileResolver);
    pkg_builder.seed_with_entry(entry_file_id, entry_src);
    let (pkg_defs, pkg_bodies, tenv, diagnostics) = pkg_builder.finish();
    if config.debug_item_tree && !config.debug_with_bodies {
        println!(
            "{}",
            pkg_defs.to_pretty(10, &pkg_bodies, &tenv, interner, config.debug_with_bodies)
        );
    }

    (pkg_defs, pkg_bodies, tenv, diagnostics)
}

pub fn lower_package_bodies(
    package_id: PackageId,
    packages: &Arena<PackageDefs>,
    package_bodies: &mut PackageBodies,
    tenv: &mut TEnv,
    interner: &'static Interner,
    config: &Config,
) -> Vec<Diagnostic> {
    tracing::info!(package =? package_id, "building package bodies");
    let mut diagnostics = vec![];
    let package = &packages[package_id];

    let mut ctx = body::LowerCtx::new(
        Some(ItemResolver::new(
            package_id,
            &package.module_tree,
            interner,
        )),
        Some(packages),
        Some(package_id),
        ModuleTree::PRELUDE_ID,
        package_bodies,
        tenv,
        interner,
        FileId::prelude(interner),
    );
    ctx.populate_trait_map();
    ctx.populate_applications();

    for (module_id, _) in package.module_tree.get().iter() {
        let file_id = packages[package_id].module_tree[module_id].file_id;

        // This is gross.
        ctx.set_module_id(module_id);
        ctx.set_file_id(file_id);
        ctx.lower_module_bodies();
    }
    diagnostics.append(&mut ctx.finish());

    if config.debug_with_bodies {
        println!(
            "{}",
            package.to_pretty(
                10,
                &package_bodies,
                tenv,
                interner,
                config.debug_with_bodies
            )
        );
    }

    diagnostics
}
