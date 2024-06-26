use la_arena::Idx;

use crate::{
    hir::{ApplyDecl, FnDecl, ModDecl, TraitDecl},
    module::ModuleId,
};

#[derive(Debug, Clone)]
pub(crate) struct ItemId {
    pub mod_id: ModuleId,
    pub idx: ItemTreeIdx,
}

impl ItemId {
    pub fn new(mod_id: ModuleId, idx: ItemTreeIdx) -> Self {
        Self { mod_id, idx }
    }
}

#[derive(Debug, Clone)]
pub enum ItemTreeIdx {
    Apply(Idx<ApplyDecl>),
    Function(Idx<FnDecl>),
    Module(Idx<ModDecl>),
    Trait(Idx<TraitDecl>),
}

impl TryFrom<ItemTreeIdx> for Idx<FnDecl> {
    type Error = ();

    fn try_from(value: ItemTreeIdx) -> Result<Self, Self::Error> {
        match value {
            ItemTreeIdx::Function(fn_id) => Ok(fn_id),
            _ => Err(()),
        }
    }
}
