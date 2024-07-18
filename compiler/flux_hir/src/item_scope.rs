use std::collections::HashMap;

use flux_span::{Interner, Word};

use crate::{
    builtin::BuiltinType,
    hir::Visibility,
    item::{ItemId, ItemTreeIdx},
    module::ModuleTree,
};

#[derive(Debug, Default)]
pub struct ItemScope {
    pub items: HashMap<Word, (Visibility, ItemId)>,
}

impl ItemScope {
    pub fn builtin(interner: &'static Interner) -> Self {
        let items = BuiltinType::all(interner)
            .into_iter()
            .map(|(name, builtin_ty)| {
                (
                    name,
                    (
                        Visibility::Public,
                        ItemId::new(ModuleTree::PRELUDE_ID, ItemTreeIdx::BuiltinType(builtin_ty)),
                    ),
                )
            })
            .collect();
        Self { items }
    }

    pub fn declare(&mut self, name: Word, visibility: Visibility, item_id: ItemId) {
        self.items.insert(name, (visibility, item_id));
    }

    pub fn get(&self, name: &Word) -> Option<(Visibility, ItemId)> {
        self.items.get(name).cloned()
    }
}
