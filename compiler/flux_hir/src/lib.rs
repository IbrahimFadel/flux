pub mod hir;
// mod package_defs;
pub(crate) mod type_interner;

mod lower;
pub use lower::{lower_ast_to_hir, lower_hir_item_bodies};
// pub use package_defs::{ModuleID, PackageDefs};
pub use type_interner::TypeInterner;
