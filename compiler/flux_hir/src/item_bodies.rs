use flux_typesystem::TChecker;
use lasso::ThreadedRodeo;

use crate::item_tree::{Function, ItemTree, LocalItemTreeId, ModItem};

pub(super) struct Context<'a> {
    item_tree: &'a ItemTree,
    tchk: TChecker,
}

impl<'a> Context<'a> {
    pub fn new(item_tree: &'a ItemTree, interner: &'static ThreadedRodeo) -> Self {
        Self {
            item_tree,
            tchk: TChecker::new(interner),
        }
    }

    pub fn lower_bodies(&mut self) {
        println!("{:#?}", self.item_tree.top_level);
        for mod_item in &self.item_tree.top_level {
            match mod_item {
                ModItem::Function(function) => self.handle_function(function),
                ModItem::Use(uze) => todo!(),
                ModItem::Struct(strukt) => todo!(),
                ModItem::Mod(m) => todo!(),
            }
        }
    }

    fn handle_function(&self, function: &LocalItemTreeId<Function>) {
        let f = &self.item_tree[function.index];
        println!("{:#?}", f);
    }
}
