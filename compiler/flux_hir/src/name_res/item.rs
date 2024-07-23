use flux_diagnostics::{ice, Diagnostic};
use flux_span::{Interner, Word};
use la_arena::{Arena, Idx};

use crate::{
    hir::{Path, TraitDecl, Visibility},
    item::{ItemId, ItemTreeIdx},
    item_scope::ItemScope,
    module::{ModuleId, ModuleTree},
    name_res::diagnostics::ResolutionError,
    PackageDefs, PackageId,
};

pub(crate) struct ItemResolver<'a> {
    builtin_scope: ItemScope,
    packages: &'a Arena<PackageDefs>,
    package_id: PackageId,
    pub(crate) dependency_ids: &'a [PackageId],
    interner: &'static Interner,
    module_tree: &'a ModuleTree,
}

pub(crate) type ResolvedItem = (PackageId, ItemId);

impl<'a> ItemResolver<'a> {
    pub(crate) fn new(
        packages: &'a Arena<PackageDefs>,
        package_id: PackageId,
        dependency_ids: &'a [PackageId],
        module_tree: &'a ModuleTree,
        interner: &'static Interner,
    ) -> Self {
        Self {
            builtin_scope: ItemScope::builtin(interner),
            interner,
            package_id,
            packages,
            dependency_ids,
            module_tree,
        }
    }
    pub fn resolve_path(
        &self,
        path: &Path,
        module_id: ModuleId,
    ) -> Result<ResolvedItem, ResolutionError> {
        let mut segments = path.segments.iter().enumerate();
        let mut name = match segments.next() {
            Some((_, segment)) => segment,
            None => return Err(ResolutionError::EmptyPath { path: path.clone() }),
        };

        // If the path is absolute (aka, begins with package name, skip to first segment that needs to be resolved)
        if *name == self.packages[self.package_id].name {
            match segments.next() {
                Some((_, segment)) => name = segment,
                None => return Err(ResolutionError::EmptyPath { path: path.clone() }),
            };
        };

        let mut curr_per_ns = self.resolve_name_in_module(name, module_id);

        if curr_per_ns.is_none() {
            return self.try_resolve_in_dependency(path);
        }

        for (i, segment) in segments {
            let (vis, item_id) = match curr_per_ns {
                Some((vis, item_id)) => (vis, item_id),
                None => {
                    return Err(ResolutionError::UnresolvedPath {
                        path: path.clone(),
                        segment: i,
                    })
                }
            };

            curr_per_ns = match &item_id.idx {
                ItemTreeIdx::Module(m) => {
                    let m = &self.packages[self.package_id].item_tree.mods[*m];
                    let (_, new_module_id) = self.module_tree[module_id]
                        .children
                        .iter()
                        .find(|(child_name, _)| **child_name == m.name.inner)
                        .unwrap_or_else(|| ice("mod should exist as child in module tree"));
                    self.resolve_name_in_module(segment, *new_module_id)
                }
                _ => {
                    if vis == Visibility::Private {
                        return Err(ResolutionError::PrivateModule {
                            path: path.clone(),
                            segment: i,
                        });
                    }
                    return Ok((self.package_id, item_id));
                }
            };

            if let Some((vis, _)) = curr_per_ns {
                if vis == Visibility::Private {
                    return Err(ResolutionError::PrivateModule {
                        path: path.clone(),
                        segment: i,
                    });
                }
            }
            if curr_per_ns.is_none() {
                return Err(ResolutionError::UnresolvedPath {
                    path: path.clone(),
                    segment: i,
                });
            }
        }

        let (vis, item_id) = curr_per_ns.unwrap(); // I think unwrap is okay here? but confirm
        Ok((self.package_id, item_id))
    }

    // pub fn resolve_path(
    //     &self,
    //     path: &Path,
    //     module_id: ModuleId,
    // ) -> Result<ResolvedItem, ResolutionError> {
    //     let mut segments = path.segments.iter().enumerate();
    //     let mut name = match segments.next() {
    //         Some((_, segment)) => segment,
    //         None => {
    //             return Err(ResolutionError::EmptyPath { path: path.clone() });
    //         }
    //     };

    //     if *name == self.packages[self.package_id].name {
    //         match segments.next() {
    //             Some((_, segment)) => name = segment,
    //             None => return Err(ResolutionError::EmptyPath { path: path.clone() }),
    //         };
    //     };

    //     let mut cur_ns = self.resolve_name_in_module(name, module_id);

    //     if cur_ns.is_none() {
    //         return self.try_resolve_in_dependency(path);
    //     }
    //     println!("{}", self.interner.resolve(name));

    //     for (i, segment) in segments {
    //         let (vis, item_id) = match cur_ns {
    //             Some(ns) => ns,
    //             None => {
    //                 return Err(ResolutionError::UnresolvedPath {
    //                     path: path.clone(),
    //                     segment: i + 1,
    //                 })
    //             }
    //         };

    //         cur_ns = match item_id.idx {
    //             crate::item::ItemTreeIdx::Module(_) => {
    //                 println!("mod {}", self.interner.resolve(name));
    //                 self.resolve_name_in_module(segment, module_id)
    //             }
    //             _ => match vis {
    //                 Visibility::Private => {
    //                     return Err(ResolutionError::PrivateModule {
    //                         path: path.clone(),
    //                         segment: i + 1,
    //                     })
    //                 }
    //                 Visibility::Public => return Ok((self.package_id, item_id)),
    //             },
    //         };
    //         println!("result {:?}", cur_ns);

    //         match &cur_ns {
    //             Some((vis, _item_id)) => {
    //                 if *vis == Visibility::Private {
    //                     return Err(ResolutionError::PrivateModule {
    //                         path: path.clone(),
    //                         segment: i + 1,
    //                     });
    //                 }
    //             }
    //             None => {
    //                 return Err(ResolutionError::UnresolvedPath {
    //                     path: path.clone(),
    //                     segment: i + 1,
    //                 })
    //             }
    //         }
    //     }

    //     let (vis, item_id) = cur_ns.unwrap(); // I think unwrap is okay here? but confirm
    //     Ok((self.package_id, item_id))
    // }

    fn resolve_name_in_module(
        &self,
        name: &Word,
        module_id: ModuleId,
    ) -> Option<(Visibility, ItemId)> {
        let from_scope = self.module_tree[module_id].scope.get(name);
        let from_builtin = || self.builtin_scope.get(name);
        from_scope.or_else(from_builtin)
    }

    fn try_resolve_in_dependency(&self, path: &Path) -> Result<ResolvedItem, ResolutionError> {
        for package_id in self.dependency_ids {
            let pkg = &self.packages[*package_id];
            if &pkg.name != path.get(0) {
                continue;
            }
            let module_tree = &pkg.module_tree;
            let package_item_resolver =
                ItemResolver::new(self.packages, *package_id, &[], module_tree, self.interner);
            return package_item_resolver.resolve_path(
                &Path::new(path.segments[1..].to_vec(), path.generic_args.clone()),
                ModuleTree::ROOT_ID,
            );
        }
        Err(ResolutionError::UnresolvedPath {
            path: path.clone(),
            segment: 0,
        })
    }

    // fn try_resolve_in_dependency(
    //     &self,
    //     path: &Spanned<Path>,
    //     original_module_id: ModuleId,
    //     dependencies: &[PackageDependency],
    //     packages: &Arena<PackageData>,
    // ) -> Result<(Option<PackageId>, Option<ModuleItemWithVis>), ResolvePathError> {
    //     for dep in dependencies {
    //         let package = &packages[dep.package_id];
    //         if &package.name == path.nth(0) {
    //             return package.resolve_path(path, original_module_id, packages);
    //             // return packagedef_map
    //             //     .resolve_path(path, def_map.root)
    //             //     .map(|(_, mod_item)| (Some(dep.clone()), mod_item));
    //         }
    //     }

    //     Err(ResolvePathError::UnresolvedPath {
    //         path: path.clone(),
    //         segment: 0,
    //     })
    // }

    // 	fn try_resolve_in_dependency(
    // 			&self,
    // 			path: &Spanned<Path>,
    // 			original_module_id: ModuleId,
    // 			dependencies: &[PackageDependency],
    // 			packages: &Arena<PackageData>,
    // 	) -> Result<(Option<PackageId>, Option<ModuleItemWithVis>), ResolvePathError> {
    // 			for dep in dependencies {
    // 					let package = &packages[dep.package_id];
    // 					if &package.name == path.nth(0) {
    // 							return package.resolve_path(path, original_module_id, packages);
    // 							// return packagedef_map
    // 							//     .resolve_path(path, def_map.root)
    // 							//     .map(|(_, mod_item)| (Some(dep.clone()), mod_item));
    // 					}
    // 			}

    // 			Err(ResolvePathError::UnresolvedPath {
    // 					path: path.clone(),
    // 					segment: 0,
    // 			})
    // 	}

    pub(crate) fn resolve_trait(
        &self,
        path: &Path,
        module_id: ModuleId,
    ) -> Result<(PackageId, Idx<TraitDecl>), ResolutionError> {
        let (package_id, item_id) = self.resolve_path(path, module_id)?;
        let trait_idx: Result<Idx<TraitDecl>, _> = item_id.idx.try_into();
        let trait_idx = trait_idx.map_err(|got| ResolutionError::ExpectedTrait {
            path: path.clone(),
            got: got.to_string(),
        })?;
        Ok((package_id, trait_idx))
    }
}
