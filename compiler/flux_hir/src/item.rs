use std::ops::Deref;

use flux_id::id::{self, InMod};

use crate::builtin::BuiltinType;

#[repr(transparent)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ItemId(InMod<ItemTreeIdx>);

impl Deref for ItemId {
    type Target = InMod<ItemTreeIdx>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<InMod<ItemTreeIdx>> for ItemId {
    fn from(value: InMod<ItemTreeIdx>) -> Self {
        Self::new(value)
    }
}

impl ItemId {
    pub fn new(idx: InMod<ItemTreeIdx>) -> Self {
        Self(idx)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ItemTreeIdx {
    Apply(id::ApplyDecl),
    BuiltinType(BuiltinType),
    Enum(id::EnumDecl),
    Function(id::FnDecl),
    Module(id::ModDecl),
    Struct(id::StructDecl),
    Trait(id::TraitDecl),
    Use(id::UseDecl),
}

impl ItemTreeIdx {
    pub fn to_item_name(&self) -> &'static str {
        match self {
            ItemTreeIdx::Apply(_) => "apply",
            ItemTreeIdx::BuiltinType(_) => "builtin",
            ItemTreeIdx::Enum(_) => "enum",
            ItemTreeIdx::Function(_) => "function",
            ItemTreeIdx::Module(_) => "module",
            ItemTreeIdx::Struct(_) => "struct",
            ItemTreeIdx::Trait(_) => "trait",
            ItemTreeIdx::Use(_) => "use",
        }
    }
}

macro_rules! impl_try_from {
    ($item:ident, $variant:ident) => {
        paste::paste! {
            impl TryFrom<ItemTreeIdx> for id::$item {
                type Error = &'static str;

                fn try_from(value: ItemTreeIdx) -> Result<Self, Self::Error> {
                    match value {
                        ItemTreeIdx::$variant(id) => Ok(id),
                        x => Err(x.to_item_name()),
                    }
                }
            }
        }
    };
}

impl_try_from!(ApplyDecl, Apply);
impl_try_from!(EnumDecl, Enum);
impl_try_from!(FnDecl, Function);
impl_try_from!(ModDecl, Module);
impl_try_from!(StructDecl, Struct);
impl_try_from!(TraitDecl, Trait);
impl_try_from!(UseDecl, Use);
