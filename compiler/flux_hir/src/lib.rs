use flux_span::FileId;
use flux_syntax::SyntaxNode;
use item_tree::{generate_item_tree, Function, ItemTree, LocalItemTreeId, ModItem};
use lasso::ThreadedRodeo;

mod hir;
mod item_bodies;
mod item_tree;
// mod source_id;

pub fn test(file_id: FileId, root: SyntaxNode, interner: &'static ThreadedRodeo) {
    let item_tree = generate_item_tree(file_id, root, interner);
}

pub fn test1(item_tree: &ItemTree, interner: &'static ThreadedRodeo) {
    let mut ctx = item_bodies::Context::new(item_tree, interner);
    ctx.lower_bodies();
}
