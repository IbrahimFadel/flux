use flux_span::Spanned;
use hashbrown::HashMap;
use lasso::Spur;

use crate::hir::ItemDefinitionId;

#[derive(Debug)]
pub struct ItemScope {
    pub functions: HashMap<Spur, ItemDefinitionId>,
    pub structs: HashMap<Spur, ItemDefinitionId>,
    pub mods: HashMap<Spur, ItemDefinitionId>,
    pub uses: Vec<ItemDefinitionId>,
}

impl ItemScope {
    pub fn new() -> Self {
        Self {
            functions: HashMap::new(),
            structs: HashMap::new(),
            mods: HashMap::new(),
            uses: vec![],
        }
    }

    pub fn get(&self, name: &Spur) -> Option<ItemDefinitionId> {
        self.functions.get(name).copied().or_else(|| self.get(name))
    }
}
