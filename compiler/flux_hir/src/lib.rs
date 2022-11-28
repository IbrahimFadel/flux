pub use item_tree::generate_item_tree;
use item_tree::ItemTree;

use lasso::ThreadedRodeo;

mod hir;
mod item_bodies;
mod item_tree;

pub fn test1(item_tree: &ItemTree, interner: &'static ThreadedRodeo) {
    let mut ctx = item_bodies::Context::new(item_tree, interner);
    ctx.lower_bodies();
}
