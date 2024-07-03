use flux_diagnostics::{Diagnostic, SourceCache};
use flux_span::{FileId, Interner};
use flux_typesystem::TEnv;
use la_arena::ArenaMap;

use crate::{
    cfg::Config,
    item::ItemId,
    item_tree::{lower_cst_to_item_tree, ItemTree},
    module::{collect::ModCollector, ModuleData, ModuleId, ModuleTree},
    name_res::{import::Import, FileResolver, ModDir},
};

mod prettyprint;

#[derive(Debug)]
pub struct Package {
    pub(crate) item_tree: ItemTree,
    pub(crate) module_tree: ModuleTree,
    pub(crate) tenvs: ArenaMap<ModuleId, TEnv>,
}

pub(crate) struct PkgBuilder<'a, R: FileResolver> {
    pub item_tree: ItemTree,
    pub module_tree: ModuleTree,
    pub interner: &'static Interner,
    pub source_cache: &'a mut SourceCache,
    pub diagnostics: Vec<Diagnostic>,
    tenvs: ArenaMap<ModuleId, TEnv>,
    config: &'a Config,
    pub resolver: R,
    unresolved_imports: Vec<Import>,
}

impl<'a, R: FileResolver> PkgBuilder<'a, R> {
    pub(crate) fn new(
        interner: &'static Interner,
        source_cache: &'a mut SourceCache,
        config: &'a Config,
        resolver: R,
    ) -> Self {
        Self {
            item_tree: ItemTree::new(),
            module_tree: ModuleTree::new(),
            interner,
            tenvs: ArenaMap::new(),
            source_cache,
            config,
            resolver,
            diagnostics: vec![],
            unresolved_imports: vec![],
        }
    }

    pub fn finish(self) -> (Package, Vec<Diagnostic>) {
        (
            Package {
                item_tree: self.item_tree,
                module_tree: self.module_tree,
                tenvs: self.tenvs,
            },
            self.diagnostics,
        )
    }

    pub(crate) fn seed_with_entry(&mut self, file_id: FileId, src: &str) {
        let (root, items) = self.new_module(file_id, src, None);
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
        parent: Option<ModuleId>,
    ) -> (ModuleId, Vec<ItemId>) {
        let mut cst = flux_parser::parse(src, file_id, self.interner);
        let root = cst.syntax();

        if self.config.debug_cst {
            println!("{}", root.debug(self.interner, true));
        }

        self.diagnostics.append(&mut cst.diagnostics);

        let module_data = ModuleData::new(parent);
        let module_id = self.module_tree.alloc(module_data);
        self.tenvs.insert(module_id, TEnv::default());

        let items = lower_cst_to_item_tree(
            root,
            file_id,
            &mut self.item_tree,
            &mut self.module_tree,
            module_id,
            self.interner,
            &mut self.tenvs[module_id],
        );

        (module_id, items)
    }

    // fn handle_lowered_items(&mut self, items: &[ItemId], mod_dir: &ModDir) {
    //     for item_id in items {
    //         match item_id.idx {
    //             ItemTreeIdx::Function(fn_id) => {
    //                 let f = &self.item_tree.functions[fn_id];
    //                 self.module_tree[item_id.mod_id].scope.declare_function(
    //                     f.name.inner,
    //                     f.visibility.inner,
    //                     item_id.clone(),
    //                 );
    //             }
    //             ItemTreeIdx::Module(mod_id) => {
    //                 let m = &self.item_tree.mods[mod_id];
    //                 self.lower_child_module(m, mod_dir);
    //             }
    //         }
    //     }
    // }
}
