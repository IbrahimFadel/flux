use crate::{hir::Visibility, ModuleDefId, ModuleId};
use hashbrown::HashMap;
use lasso::Spur;

pub(crate) type ModuleItemWithVis = (ModuleDefId, ModuleId, Visibility);

#[derive(Debug, Default)]
pub(crate) struct ItemScope {
    items: HashMap<Spur, (ModuleDefId, ModuleId, Visibility)>,
    declarations: Vec<ModuleDefId>,
}

impl ItemScope {
    pub(crate) fn get(&self, name: &Spur) -> Option<ModuleItemWithVis> {
        self.items.get(name).copied()
    }

    pub(crate) fn declare(&mut self, id: ModuleDefId) {
        self.declarations.push(id);
    }

    pub(crate) fn define_item(&mut self, name: Spur, item: ModuleItemWithVis) {
        self.items.insert(name, item);
    }
}
