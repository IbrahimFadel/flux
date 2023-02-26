use flux_diagnostics::{reporting::FileCache, Diagnostic};
use flux_span::FileId;
use hashbrown::HashMap;
use la_arena::{Arena, Idx};
use lasso::{Spur, ThreadedRodeo};

use crate::{
    hir::{Mod, Visibility},
    item_scope::ItemScope,
    item_tree::{lower_ast_to_item_tree, FileItemTreeId, ItemTree, ModItem},
    per_ns::{PerNs, PerNsGlobImports},
    ModuleDefId, ModuleId, TypeInterner,
};

use self::mod_res::ModDir;

mod mod_res;
mod path_res;

#[derive(Debug)]
pub struct DefMap {
    modules: Arena<ModuleData>,
    root: LocalModuleId,
}

pub type LocalModuleId = Idx<ModuleData>;

impl std::ops::Index<LocalModuleId> for DefMap {
    type Output = ModuleData;
    fn index(&self, id: LocalModuleId) -> &ModuleData {
        &self.modules[id]
    }
}

#[derive(Debug)]
pub struct ModuleData {
    parent: Option<ModuleId>,
    children: HashMap<Spur, ModuleId>,
    scope: ItemScope,
}

impl ModuleData {
    fn new() -> Self {
        ModuleData {
            parent: None,
            children: HashMap::default(),
            scope: ItemScope::default(),
        }
    }
}

impl DefMap {
    pub fn empty(module_data: ModuleData) -> Self {
        let mut modules = Arena::new();
        let root = modules.alloc(module_data);
        Self { modules, root }
    }
}

#[tracing::instrument(skip_all, name = "hir::build_def_map")]
pub fn build_def_map(
    entry_path: &str,
    file_cache: &mut FileCache,
    string_interner: &'static ThreadedRodeo,
    type_interner: &'static TypeInterner,
) -> (DefMap, Vec<Diagnostic>) {
    tracing::info!("building definition map for project");
    let root = ModuleData::new();
    let def_map = DefMap::empty(root);
    let mut collector = DefCollector {
        def_map,
        from_glob_import: Default::default(),
        diagnostics: vec![],
    };
    collector.seed_with_entry(entry_path, file_cache, string_interner, type_interner);
    (collector.def_map, collector.diagnostics)
}

#[derive(Debug)]
struct DefCollector {
    def_map: DefMap,
    from_glob_import: PerNsGlobImports,
    diagnostics: Vec<Diagnostic>,
}

impl DefCollector {
    fn update(
        &mut self,
        module_id: LocalModuleId,
        resolutions: &[(Option<Spur>, PerNs)],
        vis: Visibility,
    ) {
        for (name, res) in resolutions {
            match name {
                Some(name) => {
                    let scope = &mut self.def_map.modules[module_id].scope;
                    scope.push_res_with_import(
                        &mut self.from_glob_import,
                        (module_id, *name),
                        res.with_visibility(vis),
                    );
                }
                None => {}
            }
        }
    }

    pub fn seed_with_entry(
        &mut self,
        entry_path: &str,
        file_cache: &mut FileCache,
        string_interner: &'static ThreadedRodeo,
        type_interner: &'static TypeInterner,
    ) {
        let (file_id, content) = match std::fs::read_to_string(entry_path) {
            Ok(result) => {
                let file_id = file_cache.add_file(entry_path, &result);
                (file_id, result)
            }
            Err(_) => todo!(),
        };
        let (item_tree, diagnostics) =
            self.build_item_tree(file_id, &content, string_interner, type_interner);
        self.diagnostics = diagnostics;

        let module_id = self.def_map.root;
        let mod_collector = ModCollector {
            def_collector: self,
            module_id,
            item_tree: &item_tree,
            mod_dir: ModDir::root(),
            file_id,
            string_interner,
            diagnostics: vec![],
        };
        let mut diagnostics = mod_collector.collect(item_tree.items(), file_cache, type_interner);
        self.diagnostics.append(&mut diagnostics)
    }

    fn build_item_tree(
        &self,
        file_id: FileId,
        content: &str,
        string_interner: &'static ThreadedRodeo,
        type_interner: &'static TypeInterner,
    ) -> (ItemTree, Vec<Diagnostic>) {
        let parse = flux_parser::parse(content, file_id, string_interner);
        let (root, diagnostics) = (parse.syntax(), parse.diagnostics);
        (
            lower_ast_to_item_tree(root, file_id, string_interner, type_interner),
            diagnostics,
        )
    }
}

struct ModCollector<'a> {
    def_collector: &'a mut DefCollector,
    module_id: LocalModuleId,
    item_tree: &'a ItemTree,
    mod_dir: ModDir,
    file_id: FileId,
    string_interner: &'static ThreadedRodeo,
    diagnostics: Vec<Diagnostic>,
}

impl<'a> ModCollector<'a> {
    fn collect(
        mut self,
        items: &[ModItem],
        file_cache: &mut FileCache,
        type_interner: &'static TypeInterner,
    ) -> Vec<Diagnostic> {
        tracing::debug!(
            file_id = file_cache.get_file_path(&self.file_id),
            "collecting module items"
        );
        for &item in items {
            let update_def = |def_collector: &mut DefCollector, id, name: Spur, vis| {
                def_collector.def_map.modules[self.module_id]
                    .scope
                    .declare(id);
                def_collector.update(
                    self.module_id,
                    &[(Some(name), PerNs::from_def(id, vis))],
                    vis,
                );
            };
            match item {
                crate::item_tree::ModItem::Apply(_) => todo!(),
                crate::item_tree::ModItem::Enum(_) => todo!(),
                crate::item_tree::ModItem::Function(id) => {
                    let it = &self.item_tree[id];
                    update_def(self.def_collector, id.into(), it.name.inner, it.visibility);
                }
                crate::item_tree::ModItem::Mod(m) => {
                    self.collect_module(m, file_cache, type_interner);
                }
                crate::item_tree::ModItem::Struct(_) => todo!(),
                crate::item_tree::ModItem::Trait(_) => todo!(),
            }
        }
        self.diagnostics
    }

    fn collect_module(
        &mut self,
        module_id: FileItemTreeId<Mod>,
        file_cache: &mut FileCache,
        type_interner: &'static TypeInterner,
    ) {
        let module = &self.item_tree[module_id];
        let module_id = self.push_child_module(module.name.inner, module.visibility);
        let (file_id, content, mod_dir) = match self.mod_dir.resolve_declaration(
            self.file_id,
            module
                .name
                .map_ref(|name| self.string_interner.resolve(name))
                .in_file(self.file_id),
            file_cache,
        ) {
            Ok((file_id, content, mod_dir)) => (file_id, content, mod_dir),
            Err(err) => {
                self.diagnostics.push(err);
                return;
            }
        };

        let (item_tree, mut diagnostics) = self.def_collector.build_item_tree(
            file_id,
            &content,
            self.string_interner,
            type_interner,
        );
        self.diagnostics.append(&mut diagnostics);

        let mod_collector = ModCollector {
            def_collector: self.def_collector,
            module_id,
            item_tree: &item_tree,
            mod_dir,
            file_id,
            string_interner: self.string_interner,
            diagnostics: vec![],
        };
        let mut diagnostics = mod_collector.collect(item_tree.items(), file_cache, type_interner);
        self.diagnostics.append(&mut diagnostics);
    }

    fn push_child_module(&mut self, name: Spur, visibility: Visibility) -> LocalModuleId {
        let def_map = &mut self.def_collector.def_map;
        let res = def_map.modules.alloc(ModuleData::new());
        def_map.modules[res].parent = Some(self.module_id);
        def_map.modules[self.module_id].children.insert(name, res);
        let def = ModuleDefId::ModuleId(res);
        def_map.modules[self.module_id].scope.declare(def);
        self.def_collector.update(
            self.module_id,
            &[(Some(name), PerNs::from_def(def, visibility))],
            visibility,
        );
        res
    }
}
