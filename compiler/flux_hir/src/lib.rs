#![feature(is_some_and)]

mod diagnostics;
pub mod hir;
mod lower;
pub(crate) mod type_interner;

pub use lower::{lower_ast_to_hir, lower_hir_item_bodies};
pub use type_interner::TypeInterner;
