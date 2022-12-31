use flux_syntax::{
    ast::{self, AstNode},
    SyntaxNode,
};
use hir::Module;
use lasso::ThreadedRodeo;
use lower::Context;

mod hir;
mod lower;
mod type_interner;

pub fn lower_ast_to_hir(root: SyntaxNode, interner: &'static ThreadedRodeo) -> Module {
    let root =
        ast::Root::cast(root).expect("internal compiler error: root node should always cast");
    let mut ctx = Context::new(interner);
    ctx.lower(root)
}
