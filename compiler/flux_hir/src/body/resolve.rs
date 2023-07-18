use crate::{hir::Item, name_res::path_res::PathResolutionResultKind};

use super::*;

impl<'a> LowerCtx<'a> {
    // pub(super) fn get_item(
    //     &mut self,
    //     path: &Spanned<Path>,
    //     expected_kind: PathResolutionResultKind,
    // ) -> Option<(InFile<Item>, ModuleId)> {
    //     match self.try_get_item(path, expected_kind) {
    //         Ok(res) => res,
    //         Err(e) => {
    //             self.diagnostics.push(e);
    //             None
    //         }
    //     }
    // }

    // pub(super) fn try_get_item(
    //     &mut self,
    //     path: &Spanned<Path>,
    //     expected_kind: PathResolutionResultKind,
    // ) -> Result<Option<InFile<Item>>, Diagnostic> {
    //     // if let Some((item, _)) = self.resolution_cache.get(&path) {
    //     //     return Ok(Some(item.clone()));
    //     // }
    //     let mut def_map = self.def_map.unwrap();
    //     let result = def_map.resolve_path(path, self.cur_module_id);
    //     result
    //         .map(|(package_id, per_ns)| {
    //             per_ns.map(|(def_id, mod_id, _)| {
    //                 if let Some(package_id) = package_id {
    //                     def_map = &self.packages[package_id];
    //                 }
    //                 let file_id = def_map[mod_id].file_id;
    //                 let item = match def_id {
    //                     ModuleDefId::ApplyId(id) => {
    //                         Item::Apply(self.global_item_tree.unwrap()[id].clone()).in_file(file_id)
    //                     }
    //                     ModuleDefId::EnumId(id) => {
    //                         Item::Enum(self.global_item_tree.unwrap()[id].clone()).in_file(file_id)
    //                     }
    //                     ModuleDefId::StructId(id) => {
    //                         Item::Struct(self.global_item_tree.unwrap()[id].clone())
    //                             .in_file(file_id)
    //                     }
    //                     ModuleDefId::TraitId(id) => {
    //                         Item::Trait(self.global_item_tree.unwrap()[id].clone()).in_file(file_id)
    //                     }
    //                     ModuleDefId::FunctionId(id) => {
    //                         Item::Function(self.global_item_tree.unwrap()[id].clone())
    //                             .in_file(file_id)
    //                     }
    //                     ModuleDefId::ModuleId(_) => Item::Mod.in_file(file_id),
    //                     _ => todo!("{:#?}", def_id),
    //                 };
    //                 // self.resolution_cache
    //                 //     .insert(path.inner.clone(), (item, def_id));
    //                 // self.resolution_cache[&path].0.clone()
    //                 item
    //             })
    //         })
    //         .map_err(|err| {
    //             err.to_lower_error(self.file_id(), self.string_interner, expected_kind)
    //                 .to_diagnostic()
    //         })
    // }

    pub(super) fn try_get_item(
        &mut self,
        path: &Spanned<Path>,
        expected_kind: PathResolutionResultKind,
    ) -> Result<Option<InFile<Item>>, Diagnostic> {
        // if let Some((item, _)) = self.resolution_cache.get(&path) {
        //     return Ok(Some(item.clone()));
        // }
        let mut package = &self.packages[self.package_id];
        let result = package.resolve_path(path, self.cur_module_id, self.packages);
        // let result = def_map.resolve_path(path, self.cur_module_id);
        result
            .map(|(package_id, per_ns)| {
                per_ns.map(|(def_id, mod_id, _)| {
                    if let Some(package_id) = package_id {
                        package = &self.packages[package_id];
                    }
                    let file_id = package.def_map[mod_id].file_id;
                    let item_tree = &package.def_map.item_trees[mod_id];
                    let item = match def_id {
                        ModuleDefId::ApplyId(id) => {
                            Item::Apply(item_tree[id].clone()).in_file(file_id)
                        }
                        ModuleDefId::EnumId(id) => {
                            Item::Enum(item_tree[id].clone()).in_file(file_id)
                        }
                        ModuleDefId::StructId(id) => {
                            Item::Struct(item_tree[id].clone()).in_file(file_id)
                        }
                        ModuleDefId::TraitId(id) => {
                            Item::Trait(item_tree[id].clone()).in_file(file_id)
                        }
                        ModuleDefId::FunctionId(id) => {
                            Item::Function(item_tree[id].clone()).in_file(file_id)
                        }
                        ModuleDefId::ModuleId(_) => Item::Mod.in_file(file_id),
                        _ => todo!("{:#?}", def_id),
                    };
                    // self.resolution_cache
                    //     .insert(path.inner.clone(), (item, def_id));
                    // self.resolution_cache[&path].0.clone()
                    item
                })
            })
            .map_err(|err| {
                err.to_lower_error(self.file_id(), self.string_interner, expected_kind)
                    .to_diagnostic()
            })
    }

    pub(super) fn try_get_item_with_id(
        &self,
        path: &Spanned<Path>,
        expected_kind: PathResolutionResultKind,
    ) -> Result<Option<(InFile<Item>, ModuleDefId)>, Diagnostic> {
        let mut package = &self.packages[self.package_id];
        let result = package.resolve_path(path, self.cur_module_id, self.packages);
        // let result = def_map.resolve_path(path, self.cur_module_id);
        result
            .map(|(package_id, per_ns)| {
                per_ns.map(|(def_id, mod_id, _)| {
                    if let Some(package_id) = package_id {
                        package = &self.packages[package_id];
                    }
                    let file_id = package.def_map[mod_id].file_id;
                    let item_tree = &package.def_map.item_trees[mod_id];
                    let item = match def_id {
                        ModuleDefId::ApplyId(id) => {
                            Item::Apply(item_tree[id].clone()).in_file(file_id)
                        }
                        ModuleDefId::EnumId(id) => {
                            Item::Enum(item_tree[id].clone()).in_file(file_id)
                        }
                        ModuleDefId::StructId(id) => {
                            Item::Struct(item_tree[id].clone()).in_file(file_id)
                        }
                        ModuleDefId::TraitId(id) => {
                            Item::Trait(item_tree[id].clone()).in_file(file_id)
                        }
                        ModuleDefId::FunctionId(id) => {
                            Item::Function(item_tree[id].clone()).in_file(file_id)
                        }
                        ModuleDefId::ModuleId(_) => Item::Mod.in_file(file_id),
                        _ => todo!("{:#?}", def_id),
                    };
                    (item, def_id)
                })
            })
            .map_err(|err| {
                err.to_lower_error(self.file_id(), self.string_interner, expected_kind)
                    .to_diagnostic()
            })
    }

    pub(super) fn try_get_item_with_module_id(
        &self,
        path: &Spanned<Path>,
        expected_kind: PathResolutionResultKind,
    ) -> Result<Option<(InFile<Item>, ModuleDefId, ModuleId)>, Diagnostic> {
        let mut package = &self.packages[self.package_id];
        let result = package.resolve_path(path, self.cur_module_id, self.packages);
        // let result = def_map.resolve_path(path, self.cur_module_id);
        result
            .map(|(package_id, per_ns)| {
                per_ns.map(|(def_id, mod_id, _)| {
                    if let Some(package_id) = package_id {
                        package = &self.packages[package_id];
                    }
                    let file_id = package.def_map[mod_id].file_id;
                    let item_tree = &package.def_map.item_trees[mod_id];
                    let item = match def_id {
                        ModuleDefId::ApplyId(id) => {
                            Item::Apply(item_tree[id].clone()).in_file(file_id)
                        }
                        ModuleDefId::EnumId(id) => {
                            Item::Enum(item_tree[id].clone()).in_file(file_id)
                        }
                        ModuleDefId::StructId(id) => {
                            Item::Struct(item_tree[id].clone()).in_file(file_id)
                        }
                        ModuleDefId::TraitId(id) => {
                            Item::Trait(item_tree[id].clone()).in_file(file_id)
                        }
                        ModuleDefId::FunctionId(id) => {
                            Item::Function(item_tree[id].clone()).in_file(file_id)
                        }
                        ModuleDefId::ModuleId(_) => Item::Mod.in_file(file_id),
                        _ => todo!("{:#?}", def_id),
                    };
                    (item, def_id, mod_id)
                })
            })
            .map_err(|err| {
                err.to_lower_error(self.file_id(), self.string_interner, expected_kind)
                    .to_diagnostic()
            })
    }

    pub(super) fn get_type(&mut self, path: &Spanned<Path>) -> Option<InFile<Item>> {
        match self.try_get_type(path) {
            Ok(res) => res,
            Err(e) => {
                self.diagnostics.push(e);
                None
            }
        }
    }

    pub(super) fn try_get_type(
        &self,
        path: &Spanned<Path>,
    ) -> Result<Option<InFile<Item>>, Diagnostic> {
        let mut package = &self.packages[self.package_id];
        let result = package.resolve_path(path, self.cur_module_id, self.packages);
        // let mut def_map = self.def_map.unwrap();
        // let result = def_map.resolve_path(path, self.cur_module_id);
        result
            .map(|(package_id, per_ns)| {
                per_ns.map(|(def_id, mod_id, _)| {
                    if let ModuleDefId::BuiltinType(builtin_type) = def_id {
                        return Item::BuiltinType(builtin_type).in_file(FileId::poisoned());
                    }
                    if let Some(package_id) = package_id {
                        package = &self.packages[package_id];
                    }
                    let file_id = package.def_map[mod_id].file_id;
                    let item_tree = &package.def_map.item_trees[mod_id];
                    match def_id {
                        ModuleDefId::StructId(id) => {
                            Item::Struct(item_tree[id].clone()).in_file(file_id)
                        }
                        ModuleDefId::EnumId(id) => {
                            Item::Enum(item_tree[id].clone()).in_file(file_id)
                        }
                        _ => todo!(),
                    }
                })
            })
            .map_err(|err| {
                err.to_lower_error(
                    self.file_id(),
                    self.string_interner,
                    PathResolutionResultKind::Type,
                )
                .to_diagnostic()
            })
    }

    pub(super) fn get_struct(&mut self, path: &Spanned<Path>) -> Option<InFile<Struct>> {
        match self.try_get_item(path, PathResolutionResultKind::Struct) {
            Ok(item) => item.map(|item| {
                item.map(|item| match item {
                    Item::Struct(s) => s,
                    _ => todo!(),
                })
            }),
            Err(e) => {
                self.diagnostics.push(e);
                None
            }
        }
    }

    pub(super) fn get_trait(&mut self, path: &Spanned<Path>) -> Option<InFile<Trait>> {
        match self.try_get_item(path, PathResolutionResultKind::Trait) {
            Ok(item) => item.map(|item| {
                item.map(|item| match item {
                    Item::Trait(trt) => trt,
                    _ => todo!(),
                })
            }),
            Err(_) => {
                self.diagnostics.push(
                    LowerError::UnresolvedTrait {
                        trt: path.to_string(self.string_interner),
                        trt_file_span: path.span.in_file(self.file_id()),
                    }
                    .to_diagnostic(),
                );
                None
            }
        }
    }

    pub(super) fn get_trait_with_id(
        &mut self,
        path: &Spanned<Path>,
    ) -> Option<(InFile<Trait>, TraitId)> {
        match self.try_get_item_with_id(path, PathResolutionResultKind::Trait) {
            Ok(item) => item.map(|(item, def_id)| {
                let trt = item.map(|item| match item {
                    Item::Trait(trt) => trt,
                    _ => todo!(),
                });
                let id = match def_id {
                    ModuleDefId::TraitId(id) => id,
                    _ => todo!(),
                };
                (trt, id)
            }),
            Err(_) => {
                self.diagnostics.push(
                    LowerError::UnresolvedTrait {
                        trt: path.to_string(self.string_interner),
                        trt_file_span: path.span.in_file(self.file_id()),
                    }
                    .to_diagnostic(),
                );
                None
            }
        }
    }

    pub(super) fn get_trait_with_module_id(
        &mut self,
        path: &Spanned<Path>,
    ) -> Option<(InFile<Trait>, TraitId, ModuleId)> {
        match self.try_get_item_with_module_id(path, PathResolutionResultKind::Trait) {
            Ok(item) => item.map(|(item, def_id, mod_id)| {
                let trt = item.map(|item| match item {
                    Item::Trait(trt) => trt,
                    _ => todo!(),
                });
                let id = match def_id {
                    ModuleDefId::TraitId(id) => id,
                    _ => todo!(),
                };
                (trt, id, mod_id)
            }),
            Err(_) => {
                self.diagnostics.push(
                    LowerError::UnresolvedTrait {
                        trt: path.to_string(self.string_interner),
                        trt_file_span: path.span.in_file(self.file_id()),
                    }
                    .to_diagnostic(),
                );
                None
            }
        }
    }
}
