use std::collections::HashMap;

use flux_diagnostics::ice;
use flux_id::id;
use flux_util::Word;

#[derive(Debug, Default, Clone)]
pub(crate) struct Scope {
    map: HashMap<Word, id::Ty>,
}

impl Scope {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub fn insert_local(&mut self, name: Word, tid: id::Ty) {
        self.map.insert(name, tid);
    }

    pub fn try_get_local(&self, name: &Word) -> Option<&id::Ty> {
        self.map.get(name)
    }

    pub fn get_local(&self, name: &Word) -> &id::Ty {
        self.map
            .get(name)
            .unwrap_or_else(|| ice("could not get local"))
    }
}
