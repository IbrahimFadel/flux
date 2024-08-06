use flux_diagnostics::{Diagnostic, SourceCache};
use flux_id::id;
use flux_util::{FileId, Interner, Word};

use crate::{
    item::ItemId,
    lower::{item_tree::ItemTree, lower_cst_to_item_tree},
    module::{collect::ModCollector, ModuleData, ModuleTree},
    name_res::{FileResolver, ModDir},
    prelude::PRELUDE_SRC,
};

#[derive(Debug)]
pub struct Package {
    pub name: Word,
    pub(crate) item_tree: ItemTree,
    pub(crate) module_tree: ModuleTree,
    pub(crate) dependencies: Vec<id::Pkg>,
}

impl Package {
    pub fn set_dependencies(&mut self, dependencies: Vec<id::Pkg>) {
        self.dependencies = dependencies;
    }
}

pub(super) struct PkgBuilder<'a, R: FileResolver> {
    name: Word,
    pub item_tree: ItemTree,
    pub module_tree: ModuleTree,
    pub interner: &'static Interner,
    pub source_cache: &'a mut SourceCache,
    diagnostics: &'a mut Vec<Diagnostic>,
    pub resolver: R,
}

impl<'a, R: FileResolver> PkgBuilder<'a, R> {
    pub(super) fn new(
        name: Word,
        diagnostics: &'a mut Vec<Diagnostic>,
        interner: &'static Interner,
        source_cache: &'a mut SourceCache,
        resolver: R,
    ) -> Self {
        Self {
            name,
            item_tree: ItemTree::new(),
            module_tree: ModuleTree::new(),
            interner,
            source_cache,
            resolver,
            diagnostics,
        }
    }

    pub fn finish(self) -> Package {
        Package {
            name: self.name,
            item_tree: self.item_tree,
            module_tree: self.module_tree,
            dependencies: vec![],
        }
    }

    pub(crate) fn seed_with_entry(&mut self, file_id: FileId, src: &str) {
        let (root, items) = self.new_module(file_id, src, None);

        let prelude_file_id = FileId::prelude(self.interner);
        let (prelude, prelude_items) = self.new_module(prelude_file_id, PRELUDE_SRC, Some(root));

        ModCollector {
            file_id: prelude_file_id,
            mod_dir: ModDir::prelude(),
            mod_id: prelude,
            diagnostics: vec![],
            pkg_builder: self,
        }
        .collect(&prelude_items);

        let root_mod_collector = ModCollector {
            file_id,
            mod_dir: ModDir::root(),
            mod_id: root,
            diagnostics: vec![],
            pkg_builder: self,
        };
        let mut diagnostics = root_mod_collector.collect(&items);
        self.diagnostics.append(&mut diagnostics);
    }

    pub fn new_module(
        &mut self,
        file_id: FileId,
        src: &str,
        parent: Option<id::Mod>,
    ) -> (id::Mod, Vec<ItemId>) {
        let mut cst = flux_parser::parse(src, file_id, self.interner);
        let root = cst.syntax();

        self.diagnostics.append(&mut cst.diagnostics);

        let module_data = ModuleData::new(parent, file_id);
        let module_id = self.module_tree.insert(module_data);

        let items = lower_cst_to_item_tree(
            root,
            file_id,
            module_id,
            &mut self.item_tree,
            self.interner,
            &mut self.diagnostics,
        );

        (module_id, items)
    }
}
