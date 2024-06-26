use std::ops::Index;
use std::{collections::HashMap, ops::IndexMut};

use cstree::interning::{InternKey, TokenKey};
use flux_span::{FileId, Word};
use la_arena::{Arena, Idx};

use crate::item_scope::ItemScope;

pub mod collect;

pub(crate) type ModuleId = Idx<ModuleData>;

#[derive(Debug)]
pub(crate) struct ModuleTree(Arena<ModuleData>);

impl ModuleTree {
    pub fn new() -> Self {
        Self(Arena::new())
    }

    pub fn alloc(&mut self, module: ModuleData) -> ModuleId {
        self.0.alloc(module)
    }

    pub fn get(&self) -> &Arena<ModuleData> {
        &self.0
    }

    pub fn get_mut(&mut self) -> &mut Arena<ModuleData> {
        &mut self.0
    }
}

impl Index<ModuleId> for ModuleTree {
    type Output = ModuleData;

    fn index(&self, index: ModuleId) -> &Self::Output {
        &self.0[index]
    }
}

impl IndexMut<ModuleId> for ModuleTree {
    fn index_mut(&mut self, index: ModuleId) -> &mut Self::Output {
        &mut self.0[index]
    }
}

#[derive(Debug, Default)]
pub(crate) struct ModuleData {
    pub parent: Option<ModuleId>,
    pub children: HashMap<Word, ModuleId>,
    pub scope: ItemScope,
}

impl ModuleData {
    pub(crate) fn new(parent: Option<ModuleId>, file_id: FileId) -> Self {
        Self {
            parent,
            children: HashMap::new(),
            scope: ItemScope::default(),
        }
    }

    // pub(crate) fn empty() -> Self {
    //     Self {
    //         parent: None,
    //         children: HashMap::default(),
    //         scope: ItemScope::default(),
    //         file_id: FileId::new(TokenKey::try_from_u32(1).unwrap()), // THIS MUST BE UPDATED
    //     }
    // }
}
