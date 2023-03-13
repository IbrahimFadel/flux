mod body;
mod diagnostics;
pub mod hir;
mod item_scope;
mod item_tree;
mod name_res;
mod per_ns;
#[cfg(test)]
mod tests;

use hir::{Apply, Enum, Function, Struct, Trait, Use};
use la_arena::Idx;
use name_res::LocalModuleId;

pub use body::lower_def_map_bodies;
pub use name_res::{build_def_map, mod_res::BasicFileResolver};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ModuleDefId {
    ApplyId(ApplyId),
    EnumId(EnumId),
    FunctionId(FunctionId),
    ModuleId(ModuleId),
    StructId(StructId),
    TraitId(TraitId),
    UseId(UseId),
}

pub type ApplyId = Idx<Apply>;
pub type EnumId = Idx<Enum>;
pub type FunctionId = Idx<Function>;
pub type ModuleId = LocalModuleId;
pub type StructId = Idx<Struct>;
pub type TraitId = Idx<Trait>;
pub type UseId = Idx<Use>;
