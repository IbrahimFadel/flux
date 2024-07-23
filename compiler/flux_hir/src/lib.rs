#![feature(iter_array_chunks)]

use cfg::Config;
use flux_diagnostics::{Diagnostic, SourceCache};
use flux_span::{FileId, Interner, Word};
use name_res::BasicFileResolver;
use pkg::{Package, PkgBuilder};

mod body;
mod builtin;
pub mod cfg;
mod common_lower;
mod diagnostics;
mod hir;
mod intrinsics;
mod item;
mod item_scope;
pub(crate) mod item_tree;
mod module;
mod name_res;
pub mod pkg;
pub(crate) mod prelude;
pub mod prettyprint;
#[cfg(test)]
mod tests;

const POISONED_NAME: &'static str = "<poisoned>";

pub fn package_definitions(
    name: Word,
    file_id: FileId,
    src: &str,
    diagnostics: &mut Vec<Diagnostic>,
    compilation_config: &Config,
    interner: &'static Interner,
    source_cache: &mut SourceCache,
) -> Package {
    let mut pkg_builder = PkgBuilder::new(
        name,
        diagnostics,
        interner,
        source_cache,
        compilation_config,
        BasicFileResolver,
    );
    pkg_builder.seed_with_entry(file_id, src);
    pkg_builder.finish()
}

pub fn package_bodies() {
    // let lowering_ctx = body::Lo
}

// pub fn lower_package_defs(
//     name: Word,
//     entry_file_id: FileId,
//     entry_src: &str,
//     interner: &'static Interner,
//     source_cache: &mut SourceCache,
//     config: &Config,
// ) -> (PackageDefs, PackageBodies, TEnv, Vec<Diagnostic>) {
//     let mut pkg_builder: PkgBuilder<'_, BasicFileResolver> =
//         PkgBuilder::new(name, interner, source_cache, config, BasicFileResolver);
//     pkg_builder.seed_with_entry(entry_file_id, entry_src);
//     let (pkg_defs, pkg_bodies, tenv, diagnostics) = pkg_builder.finish();
//     if config.debug_item_tree && !config.debug_with_bodies {
//         println!(
//             "{}",
//             pkg_defs.to_pretty(10, &pkg_bodies, &tenv, interner, config.debug_with_bodies)
//         );
//     }

//     (pkg_defs, pkg_bodies, tenv, diagnostics)
// }

// pub fn lower_package_defs(
//     name: Word,
//     entry_file_id: FileId,
//     entry_src: &str,
//     interner: &'static Interner,
//     source_cache: &mut SourceCache,
//     config: &Config,
// ) -> (PackageDefs, PackageBodies, TEnv, Vec<Diagnostic>) {
//     let mut pkg_builder: PkgBuilder<'_, BasicFileResolver> =
//         PkgBuilder::new(name, interner, source_cache, config, BasicFileResolver);
//     pkg_builder.seed_with_entry(entry_file_id, entry_src);
//     let (pkg_defs, pkg_bodies, tenv, diagnostics) = pkg_builder.finish();
//     if config.debug_item_tree && !config.debug_with_bodies {
//         println!(
//             "{}",
//             pkg_defs.to_pretty(10, &pkg_bodies, &tenv, interner, config.debug_with_bodies)
//         );
//     }

//     (pkg_defs, pkg_bodies, tenv, diagnostics)
// }

// pub fn lower_package_bodies(
//     package_id: PackageId,
//     packages: &Arena<PackageDefs>,
//     dependency_ids: &[PackageId],
//     dependencies: &[BuiltPackage],
//     package_bodies: &mut PackageBodies,
//     tenv: &mut TEnv,
//     interner: &'static Interner,
//     config: &Config,
// ) -> Vec<Diagnostic> {
//     let mut diagnostics = vec![];
//     let package = &packages[package_id];

//     let mut ctx = body::LowerCtx::new(
//         Some(ItemResolver::new(
//             packages,
//             package_id,
//             dependency_ids,
//             &package.module_tree,
//             interner,
//         )),
//         Some(packages),
//         Some(package_id),
//         ModuleTree::PRELUDE_ID,
//         dependencies,
//         package_bodies,
//         tenv,
//         interner,
//         FileId::prelude(interner),
//     );

//     ctx.attach_trait_type_contexts();
//     for (module_id, _) in package.module_tree.get().iter() {
//         ctx.lower_module_bodies(module_id);
//     }

//     diagnostics.append(&mut ctx.finish());

//     if config.debug_with_bodies {
//         println!(
//             "{}",
//             package.to_pretty(
//                 10,
//                 &package_bodies,
//                 tenv,
//                 interner,
//                 config.debug_with_bodies
//             )
//         );
//     }

//     diagnostics
// }

// pub fn finish_package(
//     id: PackageId,
//     defs: &PackageDefs,
//     bodies: &PackageBodies,
//     tenv: &TEnv,
// ) -> BuiltPackage {
//     let mut diagnostics = vec![];
//     let mut function_signatures = ArenaMap::new();
//     let mut traits = vec![];
//     defs.item_tree
//         .top_level
//         .iter()
//         .for_each(|item| match item.idx {
//             item::ItemTreeIdx::Apply(apply_idx) => {
//                 let a = &defs.item_tree.applies[apply_idx];
//                 a.methods.iter().for_each(|method_idx| {
//                     let method = &defs.item_tree.functions[*method_idx];
//                     let sig = get_function_signature(method, tenv, &mut diagnostics);
//                     function_signatures.insert(*method_idx, sig);
//                 });
//             }
//             item::ItemTreeIdx::Function(fn_idx) => {
//                 let f = &defs.item_tree.functions[fn_idx];
//                 let sig = get_function_signature(f, tenv, &mut diagnostics);
//                 function_signatures.insert(fn_idx, sig);
//             }
//             item::ItemTreeIdx::Trait(trait_idx) => {
//                 let t = &defs.item_tree.traits[trait_idx];
//                 // t.methods.iter().for_each(|method_idx| {
//                 //     let method = &defs.item_tree.functions[*method_idx];
//                 //     let sig = get_function_signature(method, tenv, &mut diagnostics);
//                 //     function_signatures.insert(*method_idx, sig);
//                 // });
//                 traits.push((id, trait_idx, t.clone()));
//             }
//             _ => {}
//         });
//     BuiltPackage {
//         traits,
//         tenv: tenv.clone(),
//         item_tree: defs.item_tree.clone(),
//     }
// }

// fn get_function_signature(
//     f: &FnDecl,
//     tenv: &TEnv,
//     diagnostics: &mut Vec<Diagnostic>,
// ) -> Vec<FileSpanned<TypeKind>> {
//     let param_types = f.params.iter().map(|param| tenv.reconstruct(&param.ty));
//     param_types
//         .chain(std::iter::once(tenv.reconstruct(&f.return_ty)))
//         .filter_map(|r| r.map_err(|(_, err)| diagnostics.push(err)).ok())
//         .collect()
// }
