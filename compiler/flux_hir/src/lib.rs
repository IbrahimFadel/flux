mod body;
mod diagnostics;
pub mod hir;
mod item_scope;
mod item_tree;
mod name_res;
mod per_ns;
mod type_interner;

use flux_diagnostics::ice;
use flux_syntax::ast::AstNode;
use hir::{Apply, Enum, Function, Struct, Trait, Use};
use la_arena::Idx;
use name_res::LocalModuleId;
pub use type_interner::TypeInterner;

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

pub(crate) fn lower_node<N, T, P, F>(node: Option<N>, poison_function: P, normal_function: F) -> T
where
    N: AstNode,
    P: FnOnce(N) -> T,
    F: FnOnce(N) -> T,
{
    let n = node.unwrap_or_else(|| ice("missing node that should always be emitted"));
    if n.is_poisoned() {
        poison_function(n)
    } else {
        normal_function(n)
    }
}
