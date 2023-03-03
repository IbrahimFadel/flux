use flux_diagnostics::{ice, reporting::FileCache, Diagnostic, ToDiagnostic};
use flux_span::FileId;
use hashbrown::HashMap;
use la_arena::{Arena, ArenaMap, Idx};
use lasso::{Spur, ThreadedRodeo};

use crate::{
    diagnostics::LowerError,
    hir::{Mod, Use, Visibility},
    item_scope::ItemScope,
    item_tree::{lower_ast_to_item_tree, FileItemTreeId, ItemTree, ModItem},
    per_ns::{PerNs, PerNsGlobImports},
    ModuleDefId, ModuleId, TypeInterner,
};

use self::{
    mod_res::{FileResolver, ModDir},
    path_res::ResolvePathError,
};

pub(crate) mod mod_res;
pub(crate) mod path_res;
#[cfg(test)]
mod tests;

#[derive(Debug)]
pub struct DefMap {
    pub modules: Arena<ModuleData>,
    pub item_trees: ArenaMap<ModuleId, ItemTree>,
    root: LocalModuleId,
}

pub type LocalModuleId = Idx<ModuleData>;

impl std::ops::Index<LocalModuleId> for DefMap {
    type Output = ModuleData;
    fn index(&self, index: LocalModuleId) -> &ModuleData {
        &self.modules[index]
    }
}

impl std::ops::IndexMut<LocalModuleId> for DefMap {
    fn index_mut(&mut self, index: LocalModuleId) -> &mut Self::Output {
        &mut self.modules[index]
    }
}

#[derive(Debug)]
pub struct ModuleData {
    parent: Option<ModuleId>,
    children: HashMap<Spur, ModuleId>,
    pub(crate) scope: ItemScope,
    pub file_id: FileId,
}

impl ModuleData {
    pub fn new() -> Self {
        ModuleData {
            parent: None,
            children: HashMap::default(),
            scope: ItemScope::default(),
            file_id: FileId::poisoned(),
        }
    }
}

impl Default for ModuleData {
    fn default() -> Self {
        Self::new()
    }
}

impl DefMap {
    pub fn empty(module_data: ModuleData) -> Self {
        let mut modules = Arena::new();
        let root = modules.alloc(module_data);
        Self {
            modules,
            item_trees: ArenaMap::default(),
            root,
        }
    }
}

#[tracing::instrument(skip_all, name = "hir::build_def_map")]
pub fn build_def_map<R: FileResolver>(
    entry_path: &str,
    file_cache: &mut FileCache,
    string_interner: &'static ThreadedRodeo,
    type_interner: &'static TypeInterner,
    resolver: &R,
) -> (DefMap, Vec<Diagnostic>) {
    tracing::info!("building definition map for project");
    let root = ModuleData::new();
    let def_map = DefMap::empty(root);
    let mut collector = DefCollector {
        def_map,
        from_glob_import: Default::default(),
        unresolved_imports: vec![],
        diagnostics: vec![],
        string_interner,
    };
    collector.seed_with_entry(
        entry_path,
        file_cache,
        string_interner,
        type_interner,
        resolver,
    );
    collector.resolve_imports();
    (collector.def_map, collector.diagnostics)
}

#[derive(Debug, Eq, PartialEq)]
struct Import {
    /// The module this import directive is in.
    module_id: LocalModuleId,
    use_decl: FileItemTreeId<Use>,
    status: PartialResolvedImport,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum PartialResolvedImport {
    /// None of any namespaces is resolved
    Unresolved(ResolvePathError),
    /// All namespaces are resolved, OR it comes from other crate
    Resolved(PerNs),
}

impl PartialResolvedImport {
    fn namespaces(self) -> PerNs {
        match self {
            PartialResolvedImport::Unresolved(_) => PerNs::none(),
            PartialResolvedImport::Resolved(ns) => ns,
        }
    }
}

#[derive(Debug)]
struct DefCollector {
    def_map: DefMap,
    from_glob_import: PerNsGlobImports,
    unresolved_imports: Vec<Import>,
    diagnostics: Vec<Diagnostic>,
    string_interner: &'static ThreadedRodeo,
}

impl DefCollector {
    fn update(
        &mut self,
        module_id: LocalModuleId,
        resolutions: &[(Spur, PerNs)],
        vis: Option<Visibility>,
    ) {
        for (name, res) in resolutions {
            let scope = &mut self.def_map.modules[module_id].scope;
            let res = match vis {
                Some(vis) => res.with_visibility(vis),
                None => *res,
            };
            scope.push_res_with_import(&mut self.from_glob_import, (module_id, *name), res);
        }
    }

    fn resolve_imports(&mut self) {
        let imports = std::mem::take(&mut self.unresolved_imports);

        self.unresolved_imports = imports
            .into_iter()
            .filter_map(|mut import| {
                import.status = self.resolve_import(import.module_id, &import);
                match &import.status {
                    PartialResolvedImport::Resolved(_) => {
                        self.record_resolved_import(&import);
                        None
                    }
                    PartialResolvedImport::Unresolved(err) => {
                        self.diagnostics.push(
                            err.to_lower_error(
                                self.def_map.modules[import.module_id].file_id,
                                self.string_interner,
                            )
                            .to_diagnostic(),
                        );
                        Some(import)
                    }
                }
            })
            .collect();
    }

    fn resolve_import(&self, module_id: LocalModuleId, import: &Import) -> PartialResolvedImport {
        let u = &self.def_map.item_trees.get(module_id).unwrap()[import.use_decl];
        let res = self.def_map.resolve_path(&u.path, module_id);

        match res {
            Err(err) => PartialResolvedImport::Unresolved(err),
            Ok(def) => {
                if def.take_types().is_some() || def.take_values().is_some() {
                    PartialResolvedImport::Resolved(def)
                } else {
                    ice("path resolution result cannot be `Ok` with no items in the `PerNs`")
                }
            }
        }
    }

    fn record_resolved_import(&mut self, import: &Import) {
        let u = &self.def_map.item_trees.get(import.module_id).unwrap()[import.use_decl];

        let name = match &u.alias {
            Some(name) => Some(name.inner),
            None => match u.path.segments.last() {
                Some(last_segment) => Some(last_segment).copied(),
                None => {
                    return;
                }
            },
        };
        if let Some(name) = name {
            self.update(
                import.module_id,
                &[(name, import.status.clone().namespaces())],
                None,
            );
        }
    }

    pub fn seed_with_entry<R: FileResolver>(
        &mut self,
        entry_path: &str,
        file_cache: &mut FileCache,
        string_interner: &'static ThreadedRodeo,
        type_interner: &'static TypeInterner,
        resolver: &R,
    ) {
        let (file_id, content) = resolver
            .resolve_absolute_path(entry_path, file_cache)
            .unwrap();
        let (item_tree, diagnostics) =
            self.build_item_tree(file_id, &content, string_interner, type_interner);
        self.diagnostics = diagnostics;

        let module_id = self.def_map.root;
        self.def_map[module_id].file_id = file_id;

        let mod_collector = ModCollector {
            def_collector: self,
            module_id,
            item_tree: &item_tree,
            mod_dir: ModDir::root(),
            file_id,
            string_interner,
            diagnostics: vec![],
        };
        let mut diagnostics =
            mod_collector.collect(item_tree.items(), file_cache, type_interner, resolver);
        self.def_map.item_trees.insert(module_id, item_tree);
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
        let item_tree = lower_ast_to_item_tree(root, file_id, string_interner, type_interner);
        (item_tree, diagnostics)
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
    fn collect<R: FileResolver>(
        mut self,
        items: &[ModItem],
        file_cache: &mut FileCache,
        type_interner: &'static TypeInterner,
        resolver: &R,
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
                    &[(name, PerNs::from_def(id, self.module_id, vis))],
                    Some(vis),
                );
            };
            match item {
                crate::item_tree::ModItem::Apply(_) => {}
                crate::item_tree::ModItem::Enum(_) => todo!(),
                crate::item_tree::ModItem::Function(id) => {
                    let f = &self.item_tree[id];
                    update_def(
                        self.def_collector,
                        id.into(),
                        f.name.inner,
                        f.visibility.inner,
                    );
                }
                crate::item_tree::ModItem::Mod(id) => {
                    self.collect_module(id, file_cache, type_interner, resolver);
                }
                crate::item_tree::ModItem::Struct(_) => todo!(),
                crate::item_tree::ModItem::Trait(id) => {
                    let t = &self.item_tree[id];
                    update_def(self.def_collector, id.into(), t.name.inner, *t.visibility);
                }
                crate::item_tree::ModItem::Use(id) => {
                    self.def_collector.unresolved_imports.push(Import {
                        module_id: self.module_id,
                        use_decl: id,
                        status: PartialResolvedImport::Unresolved(
                            ResolvePathError::UnresolvedModule {
                                path: self.item_tree[id].path.clone(),
                                segment: 0,
                            },
                        ),
                    });
                }
            }
        }
        self.diagnostics
    }

    fn collect_module<R: FileResolver>(
        &mut self,
        module_id: FileItemTreeId<Mod>,
        file_cache: &mut FileCache,
        type_interner: &'static TypeInterner,
        resolver: &R,
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
            resolver,
        ) {
            Ok((file_id, content, mod_dir)) => (file_id, content, mod_dir),
            Err(err) => {
                self.diagnostics.push(err);
                return;
            }
        };
        self.def_collector.def_map[module_id].file_id = file_id;

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
        let mut diagnostics =
            mod_collector.collect(item_tree.items(), file_cache, type_interner, resolver);
        self.def_collector
            .def_map
            .item_trees
            .insert(module_id, item_tree);
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
            &[(name, PerNs::from_def(def, self.module_id, visibility))],
            Some(visibility),
        );
        res
    }
}
