use std::collections::HashMap;

use flux_span::Word;

use crate::TypeId;

#[derive(Debug, Default)]
pub(crate) struct Scope {
    pub map: HashMap<Word, TypeId>,
}

impl Scope {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub fn insert_local(&mut self, name: Word, tid: TypeId) {
        self.map.insert(name, tid);
    }

    pub fn try_get_local(&self, name: &Word) -> Option<TypeId> {
        self.map.get(name).cloned()
    }

    pub fn get_local(&self, name: &Word) -> TypeId {
        self.map[name]
    }
}
