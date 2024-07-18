use std::ops::Index;
use std::{collections::HashMap, ops::IndexMut};

use flux_span::{FileId, Word};
use la_arena::{Arena, Idx, RawIdx};

use crate::item_scope::ItemScope;

pub mod collect;

pub type ModuleId = Idx<ModuleData>;

#[derive(Debug)]
pub(crate) struct ModuleTree(Arena<ModuleData>);

impl ModuleTree {
    pub(crate) const PRELUDE_ID: ModuleId = Idx::from_raw(RawIdx::from_u32(1));

    pub fn new() -> Self {
        Self(Arena::new())
    }

    pub fn alloc(&mut self, module: ModuleData) -> ModuleId {
        self.0.alloc(module)
    }

    pub fn get(&self) -> &Arena<ModuleData> {
        &self.0
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

#[derive(Debug)]
pub struct ModuleData {
    pub parent: Option<ModuleId>,
    pub children: HashMap<Word, ModuleId>,
    pub scope: ItemScope,
    pub file_id: FileId,
}

impl ModuleData {
    pub(crate) fn new(parent: Option<ModuleId>, file_id: FileId) -> Self {
        Self {
            parent,
            children: HashMap::new(),
            scope: ItemScope::default(),
            file_id,
        }
    }

    // fn prelude() -> Self {
    //     Self {
    //         parent: None,
    //         children: HashMap::new(),
    //         scope: ItemScope::default(),
    //     }
    // }

    // pub(crate) fn empty() -> Self {
    //     Self {
    //         parent: None,
    //         children: HashMap::default(),
    //         scope: ItemScope::default(),
    //         file_id: FileId::new(TokenKey::try_from_u32(1).unwrap()), // THIS MUST BE UPDATED
    //     }
    // }
}
