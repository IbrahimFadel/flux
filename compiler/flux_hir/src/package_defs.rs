use std::collections::VecDeque;

use flux_span::{FileId, Span, WithSpan};
use hashbrown::HashMap;
use itertools::Itertools;
use la_arena::{Arena, ArenaMap, Idx, RawIdx};
use lasso::{Spur, ThreadedRodeo};

use crate::hir::Module;

// use crate::{
//     hir::{ItemDefinitionId, Name, Path},
//     item_scope::ItemScope,
//     Module,
// };

#[derive(Debug)]
pub struct PackageDefs {
    pub modules: ArenaMap<ModuleID, Module>,
    pub module_tree: ModuleTree,
    // interner: &'static ThreadedRodeo,
}

impl PackageDefs {
    pub fn new() -> Self {
        Self {
            modules: ArenaMap::default(),
            module_tree: ModuleTree::new(),
            // interner,
        }
    }

    // fn create_item_scope(&self, module: &Module) -> ItemScope {
    //     let mut item_scope = ItemScope::new();
    //     for (f_idx, f) in module.functions.iter() {
    //         item_scope
    //             .functions
    //             .insert(f.name.inner, ItemDefinitionId::FunctionId(f_idx));
    //     }
    //     for (s_idx, s) in module.structs.iter() {
    //         item_scope
    //             .structs
    //             .insert(s.name.inner, ItemDefinitionId::StructId(s_idx));
    //     }
    //     for (m_idx, m) in module.mods.iter() {
    //         item_scope
    //             .mods
    //             .insert(m.name.inner, ItemDefinitionId::ModuleId(m_idx));
    //     }
    //     for (u_idx, u) in module.uses.iter() {
    //         item_scope.uses.push(ItemDefinitionId::UseId(u_idx));
    //     }
    //     item_scope
    // }

    pub fn add_root_module(
        &mut self,
        module: Module,
        module_name: Spur,
        file_id: FileId,
    ) -> ModuleID {
        // let item_scope = self.create_item_scope(&module);
        let module_data = ModuleData::root(file_id);
        let idx = self.module_tree.modules.alloc(module_data);
        self.modules.insert(idx, module);
        self.module_tree.set_root(idx);
        idx
    }

    pub fn add_module(
        &mut self,
        module: Module,
        module_name: Spur,
        parent: ModuleID,
        file_id: FileId,
    ) -> ModuleID {
        // let item_scope = self.create_item_scope(&module);
        let module_data = ModuleData::new(parent, file_id);
        let idx = self.module_tree.modules.alloc(module_data);
        self.modules.insert(idx, module);
        idx
    }

    // pub fn get_item_definition_id_with_absolute_path(
    //     &self,
    //     path: &Path,
    // ) -> Option<ItemDefinitionId> {
    //     todo!()
    // }

    pub fn get_module(&self, id: ModuleID) -> &Module {
        self.modules.get(id).unwrap()
    }

    pub fn get_module_data(&self, id: ModuleID) -> &ModuleData {
        &self.module_tree[id]
    }

    pub fn add_child_module(&mut self, parent: ModuleID, child_name: Spur, child: ModuleID) {
        self.module_tree[parent].children.insert(child_name, child);
    }

    pub fn fmt(&self, interner: &'static ThreadedRodeo) -> String {
        format!("Package Defs:\n{}", self.module_tree.fmt(interner))
    }
}

// pub type ModuleID = Idx<ModuleData>;

#[derive(Debug)]
pub struct ModuleTree {
    pub root: ModuleID,
    pub modules: Arena<ModuleData>,
}

impl ModuleTree {
    pub fn new() -> Self {
        Self {
            root: ModuleID::from_raw(RawIdx::from(0)),
            modules: Arena::new(),
        }
    }

    pub fn set_root(&mut self, id: ModuleID) {
        self.root = id;
    }

    // pub fn get_absolute_path_of_module(
    //     &self,
    //     module: ModuleID,
    //     interner: &'static ThreadedRodeo,
    // ) -> Path {
    //     let m = &self[module];
    //     let mut path = vec![m.name.at(Span::new(0..0))];
    //     // let mut path = VecDeque::from([m.name]);
    //     let mut parent = m.parent;

    //     while let Some(parent_idx) = parent {
    //         let parent_data = &self[parent_idx];
    //         path.push(parent_data.name.at(Span::new(0..0)));
    //         // path.push_front(parent_data.name);
    //         parent = parent_data.parent;
    //     }

    //     Path::from_segments(path.into_iter())
    //     // interner.get_or_intern(path.iter().join("::"))
    // }

    pub fn fmt(&self, interner: &'static ThreadedRodeo) -> String {
        self.modules[self.root].fmt(&self.modules, 0, interner)
    }
}

#[derive(Debug)]
pub struct ModuleData {
    pub parent: Option<ModuleID>,
    pub children: HashMap<Spur, ModuleID>,
    pub file_id: FileId,
}

impl std::ops::Index<ModuleID> for ModuleTree {
    type Output = ModuleData;
    fn index(&self, id: ModuleID) -> &ModuleData {
        &self.modules[id]
    }
}

impl std::ops::IndexMut<ModuleID> for ModuleTree {
    fn index_mut(&mut self, id: ModuleID) -> &mut ModuleData {
        &mut self.modules[id]
    }
}

impl ModuleData {
    pub fn root(file_id: FileId) -> Self {
        Self {
            parent: None,
            children: HashMap::new(),
            file_id,
        }
    }

    pub fn new(parent: ModuleID, file_id: FileId) -> Self {
        Self {
            parent: Some(parent),
            children: HashMap::new(),
            file_id,
        }
    }

    pub fn fmt(
        &self,
        modules: &Arena<ModuleData>,
        indent_level: usize,
        interner: &'static ThreadedRodeo,
    ) -> String {
        format!(
            "{}{}\n{}",
            "\t".repeat(indent_level),
            interner.resolve(&self.file_id.0),
            self.children
                .iter()
                .map(|(_, id)| { modules[*id].fmt(modules, indent_level + 1, interner) })
                .collect::<String>()
        )
    }
}
