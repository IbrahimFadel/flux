use flux_diagnostics::ice;
use flux_id::{
    id::{self, InMod, InPkg, M, P},
    Map,
};
use flux_util::{InFile, Interner, Path, WithSpan, Word};

use crate::{
    def::item::{StructDecl, TraitDecl, Visibility},
    item::{ItemId, ItemTreeIdx},
    item_scope::ItemScope,
    module::ModuleTree,
    Package,
};

use super::diagnostics::ResolutionError;

pub(crate) type ResolvedItem = (id::Pkg, ItemId);

pub struct ItemResolver<'a> {
    builtin_scope: ItemScope,
    packages: &'a Map<id::Pkg, Package>,
    package_id: id::Pkg,
    interner: &'static Interner,
}

impl<'a> ItemResolver<'a> {
    pub fn new(
        package_id: id::Pkg,
        packages: &'a Map<id::Pkg, Package>,
        interner: &'static Interner,
    ) -> Self {
        Self {
            builtin_scope: ItemScope::builtin(interner),
            interner,
            package_id,
            packages,
        }
    }

    fn module_tree(&self, package_id: id::Pkg) -> &ModuleTree {
        &self.packages.get(package_id).module_tree
    }

    pub fn get_module_with_decl(&self, mod_id: M<id::ModDecl>) -> id::Mod {
        let m = &self
            .packages
            .get(self.package_id)
            .item_tree
            .mods
            .get(*mod_id);
        let (_, new_module_id) = self.module_tree(self.package_id)[mod_id.mod_id]
            .children
            .iter()
            .find(|(child_name, _)| **child_name == m.name.inner)
            .unwrap_or_else(|| ice("mod should exist as child in module tree"));
        *new_module_id
    }

    pub fn resolve_path<A: Clone>(
        &self,
        path: M<&Path<Word, A>>,
    ) -> Result<ResolvedItem, ResolutionError<A>> {
        let mut segments = path.segments.iter().enumerate();
        let mut name = match segments.next() {
            Some((_, segment)) => segment,
            None => {
                return Err(ResolutionError::EmptyPath {
                    path: path.inner.clone(),
                })
            }
        };

        let pkg = self.packages.get(self.package_id);
        if pkg
            .dependencies
            .iter()
            .find(|package| self.packages.get(**package).name == *name)
            .is_some()
        {
            return self.try_resolve_in_dependency(&path);
        }

        let mut mod_id = path.mod_id;

        // If the path is absolute
        if *name == pkg.name {
            mod_id = ModuleTree::ROOT_ID;
            match segments.next() {
                Some((_, segment)) => name = segment,
                None => {
                    return Err(ResolutionError::EmptyPath {
                        path: path.inner.clone(),
                    })
                }
            };
        }

        let mut curr_per_ns = self.resolve_name(name.in_mod(mod_id));

        if curr_per_ns.is_none() {
            return self.try_resolve_in_dependency(&path);
        }

        for (i, segment) in segments {
            let (vis, item_id) = match curr_per_ns {
                Some((vis, item_id)) => (vis, item_id),
                None => {
                    return Err(ResolutionError::UnresolvedPath {
                        path: path.inner.clone(),
                        segment: i,
                    })
                }
            };

            curr_per_ns = match &item_id.inner {
                ItemTreeIdx::Module(m) => {
                    let new_module_id = self.get_module_with_decl((*m).in_mod(mod_id));
                    self.resolve_name(segment.in_mod(new_module_id))
                }
                _ => {
                    if vis == Visibility::Private {
                        return Err(ResolutionError::PrivateModule {
                            path: path.inner.clone(),
                            segment: i,
                        });
                    }
                    return Ok((self.package_id, item_id));
                }
            };

            if let Some((vis, _)) = curr_per_ns {
                if vis == Visibility::Private {
                    return Err(ResolutionError::PrivateModule {
                        path: path.inner.clone(),
                        segment: i,
                    });
                }
            }
            if curr_per_ns.is_none() {
                return Err(ResolutionError::UnresolvedPath {
                    path: path.inner.clone(),
                    segment: i,
                });
            }
        }

        let (_vis, item_id) = curr_per_ns.unwrap(); // I think unwrap is okay here? but confirm
        Ok((self.package_id, item_id))
    }

    fn resolve_name(&self, name: M<&Word>) -> Option<(Visibility, ItemId)> {
        // println!(
        //     "Segment: {} in mod: {:?}",
        //     self.interner.resolve(&name),
        //     name.mod_id
        // );
        let from_scope = self
            .module_tree(self.package_id)
            .get(name.mod_id)
            .map(|module_data| module_data.scope.get(&name))
            .flatten();
        let from_builtin = || self.builtin_scope.get(&name);
        let from_prelude = || self.resolve_in_prelude(&name);
        from_scope.or_else(from_builtin).or_else(from_prelude)
    }

    fn resolve_in_prelude(&self, name: &Word) -> Option<(Visibility, ItemId)> {
        self.module_tree(self.package_id)
            .get(ModuleTree::PRELUDE_ID)
            .map(|module_data| module_data.scope.get(name))
            .flatten()
    }

    fn try_resolve_in_dependency<A: Clone>(
        &self,
        path: &Path<Word, A>,
    ) -> Result<ResolvedItem, ResolutionError<A>> {
        for package_id in &self.packages.get(self.package_id).dependencies {
            let pkg = &self.packages.get(*package_id);
            if &pkg.name != path.get_nth(0) {
                continue;
            }
            let package_item_resolver =
                ItemResolver::new(*package_id, self.packages, self.interner);
            let x = package_item_resolver.resolve_path(
                (&Path::new(path.segments[1..].to_vec(), path.args.clone()))
                    .in_mod(ModuleTree::ROOT_ID),
            );
            return x;
        }
        Err(ResolutionError::UnresolvedPath {
            path: path.clone(),
            segment: 0,
        })
    }

    pub(crate) fn resolve_trait_ids<A: Clone>(
        &self,
        path: M<&Path<Word, A>>,
    ) -> Result<P<M<id::TraitDecl>>, ResolutionError<A>> {
        let (package_id, item_id) = self.resolve_path(path)?;
        let trait_id = match &item_id.inner {
            ItemTreeIdx::Trait(id) => *id,
            ItemTreeIdx::Use(use_id) => {
                let u = self
                    .packages
                    .get(self.package_id)
                    .item_tree
                    .uses
                    .get(use_id.clone());
                let use_path = u.path.inner.clone().allow_args();
                return self.resolve_trait_ids((&use_path).in_mod(path.mod_id));
            }
            got => {
                return Err(ResolutionError::UnexpectedItem {
                    path: path.inner.clone(),
                    expected: String::from("trait"),
                    got: got.to_item_name().to_string(),
                })
            }
        };
        Ok(trait_id.in_mod(item_id.mod_id).in_pkg(package_id))
    }

    pub(crate) fn resolve_trait<A: Clone>(
        &self,
        path: M<&Path<Word, A>>,
    ) -> Result<InFile<&TraitDecl>, ResolutionError<A>> {
        let trait_id = self.resolve_trait_ids(path)?;
        let pkg = self.packages.get(trait_id.pkg_id);
        let trait_decl = pkg.item_tree.traits.get(trait_id.inner.inner);
        let file_id = pkg.module_tree[trait_id.mod_id].file_id;
        Ok(trait_decl.in_file(file_id))
    }

    pub(crate) fn resolve_struct<A: Clone>(
        &self,
        path: M<&Path<Word, A>>,
    ) -> Result<InFile<&StructDecl>, ResolutionError<A>> {
        let (package_id, item_id) = self.resolve_path(path)?;
        let struct_id: Result<id::StructDecl, _> = item_id.inner.clone().try_into();
        let struct_id = struct_id.map_err(|got| ResolutionError::UnexpectedItem {
            path: path.inner.clone(),
            expected: String::from("struct"),
            got: got.to_string(),
        })?;
        let pkg = self.packages.get(package_id);
        let struct_decl = self
            .packages
            .get(package_id)
            .item_tree
            .structs
            .get(struct_id);
        let file_id = pkg.module_tree[item_id.mod_id].file_id;

        Ok(struct_decl.in_file(file_id))
    }
}
