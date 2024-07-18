use flux_diagnostics::ice;
use flux_span::{Interner, Word};

use crate::{
    hir::{Path, Visibility},
    item::ItemId,
    item_scope::ItemScope,
    module::{ModuleId, ModuleTree},
    name_res::diagnostics::ResolutionError,
    PackageId,
};

pub(crate) struct ItemResolver<'a> {
    builtin_scope: ItemScope,
    interner: &'static Interner,
    package_id: PackageId,
    module_tree: &'a ModuleTree,
}

pub(crate) type ResolvedItem = (PackageId, ItemId);

impl<'a> ItemResolver<'a> {
    pub(crate) fn new(
        package_id: PackageId,
        module_tree: &'a ModuleTree,
        interner: &'static Interner,
    ) -> Self {
        Self {
            builtin_scope: ItemScope::builtin(interner),
            interner,
            package_id,
            module_tree,
        }
    }

    pub fn resolve_path(
        &self,
        path: &Path,
        module_id: ModuleId,
    ) -> Result<ResolvedItem, ResolutionError> {
        let mut segments = path.segments.iter().enumerate();
        let name = match segments.next() {
            Some((_, segment)) => segment,
            None => {
                return Err(ResolutionError::EmptyPath { path: path.clone() });
            }
        };

        let mut cur_ns = self.resolve_name_in_module(name, module_id);

        if cur_ns.is_none() {
            return self.try_resolve_in_dependency(path);
        }

        for (i, segment) in segments {
            let (vis, item_id) = match cur_ns {
                Some(ns) => ns,
                None => {
                    return Err(ResolutionError::UnresolvedPath {
                        path: path.clone(),
                        segment: i,
                    })
                }
            };

            cur_ns = match item_id.idx {
                crate::item::ItemTreeIdx::Module(_) => {
                    self.resolve_name_in_module(name, item_id.mod_id)
                }
                _ => match vis {
                    Visibility::Private => {
                        return Err(ResolutionError::PrivateModule {
                            path: path.clone(),
                            segment: i,
                        })
                    }
                    Visibility::Public => return Ok((self.package_id, item_id)),
                },
            };

            match &cur_ns {
                Some((vis, _item_id)) => {
                    if *vis == Visibility::Private {
                        return Err(ResolutionError::PrivateModule {
                            path: path.clone(),
                            segment: i,
                        });
                    }
                }
                None => {
                    return Err(ResolutionError::UnresolvedPath {
                        path: path.clone(),
                        segment: i,
                    })
                }
            }
        }

        let (vis, item_id) = cur_ns.unwrap(); // I think unwrap is okay here? but confirm
        Ok((self.package_id, item_id))
    }

    fn resolve_name_in_module(
        &self,
        name: &Word,
        module_id: ModuleId,
    ) -> Option<(Visibility, ItemId)> {
        let from_scope: Option<(crate::hir::Visibility, crate::item::ItemId)> =
            self.module_tree[module_id].scope.get(name);
        let from_builtin = || self.builtin_scope.get(name);
        // let from_prelude = || self.module_tree[ModuleTree::PRELUDE_ID].scope.get(name);
        from_scope.or_else(from_builtin)
    }

    fn try_resolve_in_dependency(&self, path: &Path) -> Result<ResolvedItem, ResolutionError> {
        ice("programmer gotta implement resolving in dependencies");
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
}

// impl DefMap {
// 	pub(crate) fn resolve_path(
// 			&self,
// 			package_name: &Spur,
// 			path: &Spanned<Path>,
// 			original_module_id: ModuleId,
// 			dependencies: &[PackageDependency],
// 			packages: &Arena<PackageData>,
// 	) -> Result<(Option<PackageId>, Option<ModuleItemWithVis>), ResolvePathError> {
// 			let mut segments = path.segments.iter().enumerate();
// 			let mut name = match segments.next() {
// 					Some((_, segment)) => segment,
// 					None => {
// 							return Err(ResolvePathError::EmptyPath {
// 									path_span: path.span,
// 							})
// 					}
// 			};

// 			// If the path is absolute (aka, begins with package name, skip to first segment that needs to be resolved)
// 			if name == package_name {
// 					match segments.next() {
// 							Some((_, segment)) => name = segment,
// 							None => {
// 									return Err(ResolvePathError::EmptyPath {
// 											path_span: path.span,
// 									})
// 							}
// 					};
// 			};

// 			let mut curr_per_ns = self.resolve_name_in_module(original_module_id, name);

// 			if curr_per_ns.is_none() {
// 					return self.try_resolve_in_dependency(
// 							path,
// 							original_module_id,
// 							dependencies,
// 							packages,
// 					);
// 			}

// 			for (i, segment) in segments {
// 					let (curr, m, vis) = match curr_per_ns {
// 							Some((curr, m, vis)) => (curr, m, vis),
// 							None => {
// 									return Err(ResolvePathError::UnresolvedPath {
// 											path: path.clone(),
// 											segment: i,
// 									})
// 							}
// 					};

// 					curr_per_ns = match curr {
// 							ModuleDefId::ModuleId(m) => self.resolve_name_in_module(m, segment),
// 							s => {
// 									if vis == Visibility::Private {
// 											return Err(ResolvePathError::PrivateModule {
// 													path: path.clone(),
// 													segment: i,
// 											});
// 									}
// 									return Ok((None, Some((s, m, vis))));
// 							}
// 					};

// 					if let Some((_, _, vis)) = curr_per_ns {
// 							if vis == Visibility::Private {
// 									return Err(ResolvePathError::PrivateModule {
// 											path: path.clone(),
// 											segment: i,
// 									});
// 							}
// 					}
// 					if curr_per_ns.is_none() {
// 							return Err(ResolvePathError::UnresolvedPath {
// 									path: path.clone(),
// 									segment: i,
// 							});
// 					}
// 			}

// 			Ok((None, (curr_per_ns)))
// 	}

// 	fn resolve_name_in_module(&self, module: ModuleId, name: &Spur) -> Option<ModuleItemWithVis> {
// 			let from_scope = self[module].scope.get(name);
// 			let from_builtin = self.builtin_scope.get(name).copied();
// 			let from_prelude = || self.resolve_in_prelude(name);
// 			from_scope.or(from_builtin).or_else(from_prelude)
// 	}

// 	fn resolve_in_prelude(&self, name: &Spur) -> Option<ModuleItemWithVis> {
// 			self[self.prelude].scope.get(name)
// 	}

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
// }
