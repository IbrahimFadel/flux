use la_arena::Idx;

use crate::{
    builtin::BuiltinType,
    hir::{ApplyDecl, EnumDecl, FnDecl, ModDecl, StructDecl, TraitDecl, UseDecl},
    module::ModuleId,
};

#[derive(Debug, Clone, Hash)]
pub struct ItemId {
    pub mod_id: ModuleId,
    pub idx: ItemTreeIdx,
}

impl ItemId {
    pub fn new(mod_id: ModuleId, idx: ItemTreeIdx) -> Self {
        Self { mod_id, idx }
    }

    pub fn to_item_name(&self) -> &'static str {
        match self.idx {
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

#[derive(Debug, Clone, Hash)]
pub enum ItemTreeIdx {
    Apply(Idx<ApplyDecl>),
    BuiltinType(BuiltinType),
    Enum(Idx<EnumDecl>),
    Function(Idx<FnDecl>),
    Module(Idx<ModDecl>),
    Struct(Idx<StructDecl>),
    Trait(Idx<TraitDecl>),
    Use(Idx<UseDecl>),
}

macro_rules! impl_try_from {
    ($item:ident, $variant:ident) => {
        impl TryFrom<ItemTreeIdx> for Idx<$item> {
            type Error = ();

            fn try_from(value: ItemTreeIdx) -> Result<Self, Self::Error> {
                match value {
                    ItemTreeIdx::$variant(id) => Ok(id),
                    _ => Err(()),
                }
            }
        }
    };
}

macro_rules! impl_from {
    ($item:ident, $variant:ident) => {
        impl From<ItemTreeIdx> for Idx<$item> {
            fn from(value: ItemTreeIdx) -> Idx<$item> {
                match value {
                    ItemTreeIdx::$variant(id) => id,
                    _ => unreachable!(),
                }
            }
        }
    };
}

// impl_try_from!(ApplyDecl, Apply);
// impl_try_from!(EnumDecl, Enum);
// impl_try_from!(FnDecl, Function);
// impl_try_from!(ModDecl, Module);
// impl_try_from!(StructDecl, Struct);
// impl_try_from!(TraitDecl, Trait);
// impl_try_from!(UseDecl, Use);

// impl From<ItemTreeIdx> for Idx<ApplyDecl> {
//     fn from(value: ItemTreeIdx) -> Self {
//         match value {
//             ItemTreeIdx::Apply(id) => id,
//             _ => unreachable!(),
//         }
//     }
// }

impl_from!(ApplyDecl, Apply);
impl_from!(EnumDecl, Enum);
impl_from!(FnDecl, Function);
impl_from!(ModDecl, Module);
impl_from!(StructDecl, Struct);
impl_from!(TraitDecl, Trait);
impl_from!(UseDecl, Use);

// impl TryFrom<ItemTreeIdx> for Idx<FnDecl> {
//     type Error = ();

//     fn try_from(value: ItemTreeIdx) -> Result<Self, Self::Error> {
//         match value {
//             ItemTreeIdx::Function(fn_id) => Ok(fn_id),
//             _ => Err(()),
//         }
//     }
// }
