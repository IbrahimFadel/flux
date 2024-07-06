use la_arena::Idx;

use crate::{
    hir::{ApplyDecl, EnumDecl, FnDecl, ModDecl, StructDecl, TraitDecl, UseDecl},
    module::ModuleId,
};

#[derive(Debug, Clone)]
pub struct ItemId {
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
    Enum(Idx<EnumDecl>),
    Function(Idx<FnDecl>),
    Module(Idx<ModDecl>),
    Struct(Idx<StructDecl>),
    Trait(Idx<TraitDecl>),
    Use(Idx<UseDecl>),
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
