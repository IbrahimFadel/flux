use std::collections::HashMap;

use def::expr::Expr;
use flux_diagnostics::{Diagnostic, SourceCache};
use flux_id::{
    id::{self, WithMod, WithPackage},
    Map,
};
use flux_typesystem::{MethodResolver, TraitApplication, TraitResolver, Typed};
use flux_util::{FileId, Interner, Word};
use item::ItemTreeIdx;
use lower::lower_item_bodies;
use name_res::BasicFileResolver;
use package::PkgBuilder;

mod builtin;
pub mod def;
mod diagnostics;
mod fmt;
mod intrinsics;
mod item;
mod item_scope;
mod lower;
mod module;
mod name_res;
mod package;
mod prelude;

pub use name_res::item::ItemResolver;
pub use package::Package;

pub struct Config {
    pub debug_cst: bool,
    pub debug_item_tree: bool,
    pub debug_bodies: bool,
}

impl Config {
    pub fn release() -> Self {
        Self {
            debug_cst: false,
            debug_item_tree: false,
            debug_bodies: false,
        }
    }
}

pub fn build_package_definitions(
    name: Word,
    file_id: FileId,
    src: &str,
    source_cache: &mut SourceCache,
    interner: &'static Interner,
    diagnostics: &mut Vec<Diagnostic>,
) -> Package {
    let mut pkg_builder =
        PkgBuilder::new(name, diagnostics, interner, source_cache, BasicFileResolver);
    pkg_builder.seed_with_entry(file_id, src);
    pkg_builder.finish()
}

pub fn build_package_bodies(
    package_id: id::Pkg,
    packages: &Map<id::Pkg, Package>,
    exprs: &mut Map<id::Expr, Typed<Expr>>,
    interner: &'static Interner,
    diagnostics: &mut Vec<Diagnostic>,
    source_cache: &SourceCache,
) {
    let item_tree = &packages.get(package_id).item_tree;

    let (trait_resolution, method_resolver) = build_resolvers(packages, interner);
    for item_id in &item_tree.top_level {
        lower_item_bodies(
            item_id.mod_id.in_pkg(package_id),
            item_id,
            &trait_resolution,
            &method_resolver,
            packages,
            exprs,
            interner,
            diagnostics,
            source_cache,
        );
        // if matches!(item_id.inner, item::ItemTreeIdx::Apply(_)) {
        //     break;
        // }
    }
}

fn build_resolvers(
    packages: &Map<id::Pkg, Package>,
    interner: &'static Interner,
) -> (TraitResolver, MethodResolver) {
    let mut trait_applications: HashMap<id::InPkg<id::TraitDecl>, Vec<TraitApplication>> =
        HashMap::new();
    let mut methods = Vec::new();
    for package_id in packages.keys() {
        let item_tree = &packages.get(package_id).item_tree;
        item_tree
            .top_level
            .iter()
            .for_each(|item_id| match item_id.inner {
                ItemTreeIdx::Apply(apply_id) => {
                    let apply_decl = item_tree.applies.get(apply_id);

                    let tkind_methods = apply_decl
                        .methods
                        .iter()
                        .map(|method| {
                            let method = item_tree.functions.get(*method);
                            let name = method.name.inner;
                            let signaure = method.as_fn_signature();
                            (name, signaure)
                        })
                        .collect();

                    if let Some(trt) = &apply_decl.trt {
                        let item_resolver = ItemResolver::new(package_id, packages, interner);
                        let application = item_resolver
                            .resolve_trait_ids(trt.as_ref().inner.in_mod(item_id.mod_id))
                            .map(|(trait_package_id, _, trait_id)| {
                                let trait_id = trait_id.in_pkg(trait_package_id);
                                let apply_id = apply_id.in_pkg(package_id);
                                (trait_id, apply_id)
                            })
                            .ok();

                        if let Some((trait_id, apply_id)) = application {
                            let assoc_types = apply_decl
                                .assoc_types
                                .iter()
                                .map(|assoc_ty| (assoc_ty.name.inner, assoc_ty.ty.kind.clone()))
                                .collect();
                            let app = TraitApplication::new(
                                apply_decl.to_ty.kind.clone(),
                                trt.args.iter().map(|ty| ty.kind.clone()).collect(),
                                assoc_types,
                            );
                            trait_applications.entry(trait_id).or_default().push(app);
                        }
                    }

                    methods.push((apply_decl.to_ty.kind.clone(), tkind_methods));
                }
                _ => {}
            });
    }

    (
        TraitResolver::new(trait_applications),
        MethodResolver::new(methods),
    )
}
