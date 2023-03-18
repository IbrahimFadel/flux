use crate::{hir::Item, name_res::path_res::PathResolutionResultKind};

use super::*;

impl<'a> LowerCtx<'a> {
    pub(super) fn get_item(
        &mut self,
        path: &Spanned<Path>,
        expected_kind: PathResolutionResultKind,
    ) -> Option<(InFile<Item>, ModuleId)> {
        match self.try_get_item(path, expected_kind) {
            Ok(res) => res,
            Err(e) => {
                self.diagnostics.push(e);
                None
            }
        }
    }

    pub(super) fn try_get_item(
        &self,
        path: &Spanned<Path>,
        expected_kind: PathResolutionResultKind,
    ) -> Result<Option<(InFile<Item>, ModuleId)>, Diagnostic> {
        let def_map = self.def_map.unwrap();
        let result = def_map.resolve_path(path, self.cur_module_id);
        result
            .map(|per_ns| {
                per_ns.map(|(def_id, mod_id, _)| {
                    let file_id = def_map[mod_id].file_id;
                    let item_tree = &def_map.item_trees[mod_id];
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
                        _ => todo!(),
                    };
                    (item, mod_id)
                })
            })
            .map_err(|err| {
                err.to_lower_error(self.file_id(), self.string_interner, expected_kind)
                    .to_diagnostic()
            })
    }

    pub(super) fn get_type(&mut self, path: &Spanned<Path>) -> Option<(InFile<Item>, ModuleId)> {
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
    ) -> Result<Option<(InFile<Item>, ModuleId)>, Diagnostic> {
        let def_map = self.def_map.unwrap();
        let result = def_map.resolve_path(path, self.cur_module_id);
        result
            .map(|per_ns| {
                per_ns.map(|(def_id, mod_id, vis)| {
                    if let ModuleDefId::BuiltinType(builtin_type) = def_id {
                        return (
                            Item::BuiltinType(builtin_type).in_file(FileId::poisoned()),
                            mod_id,
                        );
                    }
                    let file_id = def_map[mod_id].file_id;
                    let item_tree = &def_map.item_trees[mod_id];
                    let item = match def_id {
                        ModuleDefId::StructId(id) => {
                            Item::Struct(item_tree[id].clone()).in_file(file_id)
                        }
                        _ => todo!(),
                    };
                    (item, mod_id)
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

    // pub(super) fn try_get_type(&self, path: &Spanned<Path>) -> Option<(InFile<Item>, ModuleId)> {
    //     let def_map = self.def_map.unwrap();
    //     let result = def_map.resolve_path(path, self.cur_module_id);
    //     match result {
    //         Ok(per_ns) => per_ns.types.map(|(def_id, mod_id, _vis)| {
    //             if let ModuleDefId::BuiltinType(builtin_type) = def_id {
    //                 return (
    //                     Item::BuiltinType(builtin_type).in_file(FileId::poisoned()),
    //                     mod_id,
    //                 );
    //             }
    //             let file_id = def_map[mod_id].file_id;
    //             let item_tree = &def_map.item_trees[mod_id];
    //             let item = match def_id {
    //                 ModuleDefId::EnumId(id) => Item::Enum(item_tree[id].clone()).in_file(file_id),
    //                 ModuleDefId::StructId(id) => {
    //                     Item::Struct(item_tree[id].clone()).in_file(file_id)
    //                 }
    //                 _ => todo!(),
    //             };
    //             (item, mod_id)
    //         }),
    //         Err(_) => None,
    //     }
    // }

    pub(super) fn get_struct(
        &mut self,
        path: &Spanned<Path>,
    ) -> Option<(InFile<Struct>, ModuleId)> {
        match self.try_get_item(path, PathResolutionResultKind::Struct) {
            Ok(item) => item.map(|(item, mod_id)| {
                let item = item.map(|item| match item {
                    Item::Struct(s) => s,
                    _ => todo!(),
                });
                (item, mod_id)
            }),
            Err(e) => {
                self.diagnostics.push(e);
                // self.diagnostics.push(
                //     LowerError::UnresolvedStruct {
                //         strukt: path.to_string(self.string_interner),
                //         strukt_file_span: path.span.in_file(self.file_id()),
                //     }
                //     .to_diagnostic(),
                // );
                None
            }
        }
        // self.get_item(path).map(|(item, mod_id, vis)| {
        //     let item = item.map(|item| match item {
        //         Item::Struct(trt) => trt,
        //         _ => todo!(),
        //     });
        //     (item, mod_id, vis)
        // })
    }

    pub(super) fn get_trait(&mut self, path: &Spanned<Path>) -> Option<(InFile<Trait>, ModuleId)> {
        match self.try_get_item(path, PathResolutionResultKind::Trait) {
            Ok(item) => item.map(|(item, mod_id)| {
                let item = item.map(|item| match item {
                    Item::Trait(trt) => trt,
                    _ => todo!(),
                });
                (item, mod_id)
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
        // self.get_item(path).map(|(item, mod_id, vis)| {
        //     let item = item.map(|item| match item {
        //         Item::Trait(trt) => trt,
        //         _ => todo!(),
        //     });
        //     (item, mod_id, vis)
        // })
    }
}
