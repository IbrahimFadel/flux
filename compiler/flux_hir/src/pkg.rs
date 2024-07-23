use std::collections::HashMap;

use flux_diagnostics::{Diagnostic, SourceCache};
use flux_span::{FileId, Interner, Word};
use flux_typesystem::TEnv;
use la_arena::{Arena, Idx};

use crate::{
    cfg::Config,
    hir::{Expr, ExprIdx, FnDecl, TraitDecl},
    item::ItemId,
    item_tree::{lower_cst_to_item_tree, ItemTree},
    module::{collect::ModCollector, ModuleData, ModuleId, ModuleTree},
    name_res::{import::Import, FileResolver, ModDir},
    prelude::PRELUDE_SRC,
};

mod prettyprint;

#[derive(Debug, Clone)]
pub struct BuiltPackage {
    pub traits: Vec<(PackageId, Idx<TraitDecl>, TraitDecl)>,
    pub tenv: TEnv,
    pub item_tree: ItemTree,
}

#[derive(Debug)]
pub struct PackageDefs {
    pub(crate) name: Word,
    pub(crate) item_tree: ItemTree,
    pub(crate) module_tree: ModuleTree,
}

pub type PackageId = Idx<PackageDefs>;

#[derive(Debug)]
pub struct PackageBodies {
    pub(crate) exprs: Arena<Expr>,
    pub(crate) fn_exprs: HashMap<(ModuleId, Idx<FnDecl>), ExprIdx>,
}

pub(crate) struct PkgBuilder<'a, R: FileResolver> {
    name: Word,
    pub item_tree: ItemTree,
    pub module_tree: ModuleTree,
    pub interner: &'static Interner,
    pub source_cache: &'a mut SourceCache,
    pub diagnostics: Vec<Diagnostic>,
    package_bodies: PackageBodies,
    tenv: TEnv,
    config: &'a Config,
    pub resolver: R,
    unresolved_imports: Vec<Import>,
}

impl<'a, R: FileResolver> PkgBuilder<'a, R> {
    pub(crate) fn new(
        name: Word,
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
            package_bodies: PackageBodies {
                exprs: Arena::new(),
                fn_exprs: HashMap::new(),
            },
            tenv: TEnv::new(interner),
            source_cache,
            config,
            resolver,
            diagnostics: vec![],
            unresolved_imports: vec![],
        }
    }

    pub fn finish(self) -> (PackageDefs, PackageBodies, TEnv, Vec<Diagnostic>) {
        (
            PackageDefs {
                name: self.name,
                item_tree: self.item_tree,
                module_tree: self.module_tree,
            },
            self.package_bodies,
            self.tenv,
            self.diagnostics,
        )
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
        parent: Option<ModuleId>,
    ) -> (ModuleId, Vec<ItemId>) {
        let mut cst = flux_parser::parse(src, file_id, self.interner);
        let root = cst.syntax();

        if self.config.debug_cst {
            println!("{}", root.debug(self.interner, true));
        }

        self.diagnostics.append(&mut cst.diagnostics);

        let module_data = ModuleData::new(parent, file_id);
        let module_id = self.module_tree.alloc(module_data);

        let (items, mut diagnostics) = lower_cst_to_item_tree(
            root,
            file_id,
            &mut self.item_tree,
            module_id,
            self.interner,
            &mut self.package_bodies,
            &mut self.tenv,
        );

        self.diagnostics.append(&mut diagnostics);

        (module_id, items)
    }
}
