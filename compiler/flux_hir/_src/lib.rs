#![feature(drain_filter)]

use flux_syntax::{
    ast::{self, AstNode},
    SyntaxNode,
};
use lasso::ThreadedRodeo;
use lower::Context;

pub mod hir;
mod item_scope;
mod lower;
pub mod package_defs;
mod tchk;
mod type_interner;

pub use hir::Module;
pub use tchk::tychk_package;
pub use type_interner::TypeInterner;

pub fn lower_ast_to_hir(root: SyntaxNode, interner: &'static ThreadedRodeo) -> Module {
    let root =
        ast::Root::cast(root).expect("internal compiler error: root node should always cast");
    let ctx = Context::new(interner);
    ctx.lower(root)
}
