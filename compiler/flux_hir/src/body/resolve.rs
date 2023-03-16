use crate::hir::Item;

use super::*;

impl<'a> LowerCtx<'a> {
    pub(super) fn try_get_item(&self, path: &Spanned<Path>) -> Option<InFile<Item>> {
        let def_map = self.def_map.unwrap();
        let result = def_map.resolve_path(path, self.cur_module_id);
        match result {
            Ok(per_ns) => per_ns
                .types
                .map(|(def_id, mod_id, vis)| {
                    let file_id = def_map[mod_id].file_id;
                    let item_tree = &def_map.item_trees[mod_id];
                    match def_id {
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
                        _ => todo!(),
                    }
                })
                .or_else(|| {
                    per_ns.values.map(|(def_id, mod_id, vis)| {
                        let file_id = def_map[mod_id].file_id;
                        let item_tree = &def_map.item_trees[mod_id];
                        match def_id {
                            ModuleDefId::FunctionId(id) => {
                                Item::Function(item_tree[id].clone()).in_file(file_id)
                            }
                            _ => todo!(),
                        }
                    })
                }),
            Err(_) => None,
        }
    }

    pub(super) fn get_struct(&mut self, path: &Spanned<Path>) -> Option<InFile<&Struct>> {
        let def_map = self.def_map.unwrap();
        let result = def_map.resolve_path(path, self.cur_module_id);
        match result {
            Ok(per_ns) => per_ns.types.map(|(def_id, mod_id, vis)| {
                let file_id = def_map[mod_id].file_id;
                let item_tree = &def_map.item_trees[mod_id];
                match def_id {
                    ModuleDefId::StructId(id) => Some((&item_tree[id]).in_file_ref(file_id)),
                    _ => {
                        self.diagnostics.push(
                            LowerError::CouldNotFindStructButFoundAnotherItem {
                                struct_path: path.to_string(self.string_interner),
                                struct_path_file_span: path.span.in_file(self.file_id()),
                                other_item_kind: def_id.to_item_kind().to_string(),
                            }
                            .to_diagnostic(),
                        );
                        None
                    }
                }
            }),
            Err(err) => {
                self.diagnostics.push(
                    err.to_lower_error(self.file_id(), self.string_interner)
                        .to_diagnostic(),
                );
                None
            }
        }
        .flatten()
    }

    pub(super) fn get_function(&mut self, path: &Spanned<Path>) -> Option<InFile<&Function>> {
        let def_map = self.def_map.unwrap();
        let result = def_map.resolve_path(path, self.cur_module_id);
        match result {
            Ok(per_ns) => per_ns.types.map(|(def_id, mod_id, vis)| {
                let file_id = def_map[mod_id].file_id;
                let item_tree = &def_map.item_trees[mod_id];
                match def_id {
                    ModuleDefId::FunctionId(id) => Some((&item_tree[id]).in_file_ref(file_id)),
                    _ => {
                        self.diagnostics.push(
                            LowerError::CouldNotFindFunctionButFoundAnotherItem {
                                function_path: path.to_string(self.string_interner),
                                function_path_file_span: path.span.in_file(self.file_id()),
                                other_item_kind: def_id.to_item_kind().to_string(),
                            }
                            .to_diagnostic(),
                        );
                        None
                    }
                }
            }),
            Err(err) => {
                self.diagnostics.push(
                    err.to_lower_error(self.file_id(), self.string_interner)
                        .to_diagnostic(),
                );
                None
            }
        }
        .flatten()
    }
}

impl ModuleDefId {
    fn to_item_kind(&self) -> &str {
        match self {
            ModuleDefId::ApplyId(_) => unreachable!(),
            ModuleDefId::EnumId(_) => "enum",
            ModuleDefId::FunctionId(_) => "function",
            ModuleDefId::ModuleId(_) => "module",
            ModuleDefId::StructId(_) => "struct",
            ModuleDefId::TraitId(_) => "trait",
            ModuleDefId::UseId(_) => unreachable!(),
        }
    }
}
