use hashbrown::HashMap;
use la_arena::{Arena, Idx};
use lasso::Spur;

use crate::{
    hir::{Mod, Visibility},
    item_scope::ItemScope,
    item_tree::{FileItemTreeId, ItemTree, ModItem},
    per_ns::{PerNs, PerNsGlobImports},
    ModuleDefId, ModuleId,
};

pub struct DefMap {
    modules: Arena<ModuleData>,
}

pub type LocalModuleId = Idx<ModuleData>;

impl std::ops::Index<LocalModuleId> for DefMap {
    type Output = ModuleData;
    fn index(&self, id: LocalModuleId) -> &ModuleData {
        &self.modules[id]
    }
}

pub struct ModuleData {
    parent: Option<ModuleId>,
    children: HashMap<Spur, ModuleId>,
    scope: ItemScope,
}

impl ModuleData {
    fn new() -> Self {
        ModuleData {
            parent: None,
            children: HashMap::default(),
            scope: ItemScope::default(),
        }
    }
}

impl DefMap {
    pub fn empty(module_data: ModuleData) -> Self {
        let mut modules = Arena::new();
        modules.alloc(module_data);
        Self { modules }
    }
}

pub fn build_dep_map(item_trees: impl Iterator<Item = ItemTree>) -> DefMap {
    let root = ModuleData::new();
    let def_map = DefMap::empty(root);
    let collector = DefCollector {
        def_map,
        from_glob_import: Default::default(),
    };
    collector.collect();
    collector.def_map
}

struct DefCollector {
    def_map: DefMap,
    // glob_imports: HashMap<LocalModuleId, Vec<(LocalModuleId, Visibility)>>,
    from_glob_import: PerNsGlobImports,
}

impl DefCollector {
    fn collect(&self) {}

    fn update(
        &mut self,
        module_id: LocalModuleId,
        resolutions: &[(Option<Spur>, PerNs)],
        vis: Visibility,
    ) {
        for (name, res) in resolutions {
            match name {
                Some(name) => {
                    let scope = &mut self.def_map.modules[module_id].scope;
                    scope.push_res_with_import(
                        &mut self.from_glob_import,
                        (module_id, name.clone()),
                        res.with_visibility(vis),
                    );
                }
                None => {}
            }
        }
    }
}

struct ModCollector<'a> {
    def_collector: &'a mut DefCollector,
    module_id: LocalModuleId,
    item_tree: &'a ItemTree,
}

impl<'a> ModCollector<'a> {
    fn collect(&mut self, items: &[ModItem]) {
        let update_def = |def_collector: &mut DefCollector, id, name: Spur, vis| {
            def_collector.def_map.modules[self.module_id]
                .scope
                .declare(id);
            def_collector.update(
                self.module_id,
                &[(Some(name), PerNs::from_def(id, vis))],
                vis,
            );
        };

        for &item in items {
            match item {
                crate::item_tree::ModItem::Apply(_) => todo!(),
                crate::item_tree::ModItem::Enum(_) => todo!(),
                crate::item_tree::ModItem::Function(id) => {
                    let it = &self.item_tree[id];
                    update_def(self.def_collector, id.into(), it.name.inner, it.visibility);
                }
                crate::item_tree::ModItem::Mod(m) => self.collect_module(m),
                crate::item_tree::ModItem::Struct(_) => todo!(),
                crate::item_tree::ModItem::Trait(_) => todo!(),
            }
        }
    }

    fn collect_module(&mut self, module_id: FileItemTreeId<Mod>) {
        let module = &self.item_tree[module_id];
        let module_id = self.push_child_module(module.name.inner, module.visibility);
        let mut mod_collector = ModCollector {
            def_collector: self.def_collector,
            module_id,
            item_tree: self.item_tree,
        };
        // TODO: figure out how to now parse and generate item tree for this submodule. basically, no more doing this in the driver, we do it here, recursively.
        mod_collector.collect(items);
    }

    fn push_child_module(&mut self, name: Spur, visibility: Visibility) -> LocalModuleId {
        let def_map = &mut self.def_collector.def_map;
        let res = def_map.modules.alloc(ModuleData::new());
        def_map.modules[res].parent = Some(self.module_id);
        def_map.modules[self.module_id]
            .children
            .insert(name.clone(), res);
        let def = ModuleDefId::ModuleId(res);
        def_map.modules[self.module_id].scope.declare(def);
        self.def_collector.update(
            self.module_id,
            &[(Some(name), PerNs::from_def(def, visibility))],
            visibility,
        );
        res
    }
}
