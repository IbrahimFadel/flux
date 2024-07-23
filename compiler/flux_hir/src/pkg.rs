use std::collections::HashMap;

use flux_diagnostics::{Diagnostic, SourceCache};
use flux_id::id::{self, M};
use flux_span::{FileId, Interner, Word};
use la_arena::{Arena, Idx};

use crate::{
    cfg::Config,
    hir::Expr,
    item::ItemId,
    item_tree::{lower_cst_to_item_tree, ItemTree},
    module::{collect::ModCollector, ModuleData, ModuleTree},
    name_res::{import::Import, FileResolver, ModDir},
    prelude::PRELUDE_SRC,
};

pub struct Package {
    pub name: Word,
    pub item_tree: ItemTree,
    pub module_tree: ModuleTree,
}

pub type PackageId = Idx<Package>;

// #[derive(Debug, Clone)]
// pub struct BuiltPackage {
//     pub traits: Vec<(PackageId, Idx<TraitDecl>, TraitDecl)>,
//     pub tenv: TEnv,
//     pub item_tree: ItemTree,
// }

// #[derive(Debug)]
// pub struct PackageDefs {
//     pub(crate) name: Word,
//     pub(crate) item_tree: ItemTree,
//     pub(crate) module_tree: ModuleTree,
// }

#[derive(Debug)]
pub struct PackageBodies {
    pub(crate) exprs: Arena<Expr>,
    pub(crate) fn_exprs: HashMap<M<id::FnDecl>, id::Expr>,
}

pub(crate) struct PkgBuilder<'a, R: FileResolver> {
    name: Word,
    pub item_tree: ItemTree,
    pub module_tree: ModuleTree,
    pub interner: &'static Interner,
    pub source_cache: &'a mut SourceCache,
    diagnostics: &'a mut Vec<Diagnostic>,
    config: &'a Config,
    pub resolver: R,
    unresolved_imports: Vec<Import>,
}

impl<'a, R: FileResolver> PkgBuilder<'a, R> {
    pub(crate) fn new(
        name: Word,
        diagnostics: &'a mut Vec<Diagnostic>,
        interner: &'static Interner,
        source_cache: &'a mut SourceCache,
        config: &'a Config,
        resolver: R,
    ) -> Self {
        Self {
            name,
            item_tree: ItemTree::new(),
            module_tree: ModuleTree::new(),
            interner,
            source_cache,
            config,
            resolver,
            diagnostics,
            unresolved_imports: vec![],
        }
    }

    pub fn finish(self) -> Package {
        Package {
            name: self.name,
            item_tree: self.item_tree,
            module_tree: self.module_tree,
        }
    }

    pub(crate) fn seed_with_entry(&mut self, file_id: FileId, src: &str) {
        let (root, items) = self.new_module(file_id, src, None);

        let prelude_fileid = FileId::prelude(self.interner);
        let (prelude, prelude_items) = self.new_module(prelude_fileid, PRELUDE_SRC, Some(root));
        ModCollector {
            file_id: prelude_fileid,
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

        if self.config.debug_cst {
            println!("{}", root.debug(self.interner, true));
        }

        self.diagnostics.append(&mut cst.diagnostics);

        let module_data = ModuleData::new(parent, file_id);
        let module_id = self.module_tree.insert(module_data);

        let items = lower_cst_to_item_tree(
            root,
            file_id,
            module_id,
            &mut self.item_tree,
            &mut self.diagnostics,
            self.interner,
        );

        (module_id, items)
    }
}
