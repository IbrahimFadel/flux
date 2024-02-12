use flux_parser::FluxHirInputFileExt;
use flux_span::InputFile;
use la_arena::Arena;

use crate::{item_tree::ItemTree, module::ModuleData};

pub(crate) struct PkgBuilder<'db> {
    db: &'db dyn crate::Db,
    item_tree: ItemTree,
    module_tree: Arena<ModuleData>,
}

impl<'db> PkgBuilder<'db> {
    pub(crate) fn new(db: &'db dyn crate::Db) -> Self {
        Self {
            db,
            item_tree: ItemTree::new(),
            module_tree: Arena::new(),
        }
    }

    pub(crate) fn seed_with_entry(&mut self, file: InputFile) {
        let cst = file.cst(self.db);
    }
}
