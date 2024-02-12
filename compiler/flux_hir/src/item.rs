use la_arena::Idx;

use crate::{hir::Function, module::ModuleId};

#[derive(Debug, Clone)]
pub(crate) struct ItemId {
    mod_id: ModuleId,
    idx: ItemTreeIdx,
}

impl ItemId {
    pub fn new(mod_id: ModuleId, idx: ItemTreeIdx) -> Self {
        Self { mod_id, idx }
    }
}

#[derive(Debug, Clone)]
pub enum ItemTreeIdx {
    Function(Idx<Function>),
}
