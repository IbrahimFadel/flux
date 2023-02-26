mod body;
mod diagnostics;
pub mod hir;
mod item_scope;
mod item_tree;
mod name_res;
mod per_ns;
pub(crate) mod type_interner;

use hir::Function;
use la_arena::Idx;
use name_res::LocalModuleId;
pub use type_interner::TypeInterner;

use item_tree::{lower_ast_to_item_tree, FileItemTreeId};
pub use name_res::build_def_map;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ModuleDefId {
    ModuleId(ModuleId),
    FunctionId(FunctionId),
}

pub type ModuleId = LocalModuleId;
pub type FunctionId = Idx<Function>;

// impl From<FunctionId> for ModuleDefId {
//     fn from(value: FunctionId) -> Self {

//     }
// }
