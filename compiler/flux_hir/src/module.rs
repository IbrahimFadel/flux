use std::ops::Index;
use std::{collections::HashMap, ops::IndexMut};

use flux_id::{id, Map};
use flux_util::{FileId, Word};

use crate::item_scope::ItemScope;

pub(super) mod collect;

#[derive(Debug)]
pub struct ModuleTree(Map<id::Mod, ModuleData>);

impl ModuleTree {
    pub(crate) const ROOT_ID: id::Mod = id::Mod::from_idx(0);
    pub(crate) const PRELUDE_ID: id::Mod = id::Mod::from_idx(1);

    pub fn new() -> Self {
        Self(Map::new())
    }

    pub fn get(&self, mod_id: id::Mod) -> Option<&ModuleData> {
        self.0.try_get(mod_id)
    }

    pub fn insert(&mut self, module: ModuleData) -> id::Mod {
        self.0.insert(module)
    }

    pub fn iter(&self) -> impl Iterator<Item = (id::Mod, &ModuleData)> {
        self.0.iter()
    }
}

impl Index<id::Mod> for ModuleTree {
    type Output = ModuleData;

    fn index(&self, index: id::Mod) -> &Self::Output {
        self.0.get(index)
    }
}

impl IndexMut<id::Mod> for ModuleTree {
    fn index_mut(&mut self, index: id::Mod) -> &mut Self::Output {
        self.0.get_mut(index)
    }
}

#[derive(Debug)]
pub struct ModuleData {
    pub parent: Option<id::Mod>,
    pub children: HashMap<Word, id::Mod>,
    pub scope: ItemScope,
    pub file_id: FileId,
}

impl ModuleData {
    pub(crate) fn new(parent: Option<id::Mod>, file_id: FileId) -> Self {
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
