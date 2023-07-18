use std::collections::HashMap;

use flux_diagnostics::{ice, reporting::FileCache, Diagnostic, ToDiagnostic};
use flux_span::{FileId, Spanned};
use la_arena::{Arena, ArenaMap, Idx};
use lasso::{Spur, ThreadedRodeo};

use crate::{
    builtin::BuiltinType,
    diagnostics::LowerError,
    hir::{Mod, Type, Use, Visibility},
    item_scope::{ItemScope, ModuleItemWithVis},
    item_tree::{lower_ast_to_item_tree, FileItemTreeId, ItemTree, ModItem},
    ModuleDefId, ModuleId,
};

use self::{
    mod_res::{FileResolver, ModDir},
    path_res::{PathResolutionResultKind, ResolvePathError},
};

pub(crate) mod mod_res;
pub(crate) mod path_res;

#[derive(Debug, Clone)]
pub struct DefMap {
    // name: Spur,
    pub modules: Arena<ModuleData>,
    pub item_trees: ArenaMap<ModuleId, ItemTree>,
    // pub items: ArenaMap<ModuleId, Vec<ModItem>>,
    // packages: Arena<Arc<DefMap>>,
    // pub dependencies: Vec<PackageId>,
    root: ModuleId,
    pub prelude: ModuleId,
    builtin_scope: HashMap<Spur, ModuleItemWithVis>,
}

#[derive(Debug, Clone)]
pub struct PackageData {
    // root_file_id: FileId,
    pub def_map: DefMap,
    name: Spur,
    dependencies: Vec<PackageDependency>,
}

impl PackageData {
    pub fn new(name: Spur, def_map: DefMap, dependencies: Vec<PackageDependency>) -> Self {
        Self {
            def_map,
            name,
            dependencies,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PackageDependency {
    pub name: Spur,
    pub package_id: PackageId,
}

pub type PackageId = Idx<PackageData>;

impl std::ops::Index<ModuleId> for DefMap {
    type Output = ModuleData;
    fn index(&self, index: ModuleId) -> &ModuleData {
        &self.modules[index]
    }
}

impl std::ops::IndexMut<ModuleId> for DefMap {
    fn index_mut(&mut self, index: ModuleId) -> &mut Self::Output {
        &mut self.modules[index]
    }
}

#[derive(Debug, Clone)]
pub struct ModuleData {
    pub(crate) parent: Option<ModuleId>,
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

    pub fn prelude() -> Self {
        let scope = ItemScope::default();
        ModuleData {
            parent: None,
            children: HashMap::default(),
            scope,
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
    pub fn empty(module_data: ModuleData, string_interner: &'static ThreadedRodeo) -> Self {
        let mut modules = Arena::new();
        let root = modules.alloc(module_data);

        let prelude_data = ModuleData::prelude();
        let prelude = modules.alloc(prelude_data);

        let builtin_scope = BuiltinType::all(string_interner)
            .iter()
            .map(|(name, ty)| {
                (
                    name.clone(),
                    ModuleItemWithVis::from(((*ty).into(), prelude, Visibility::Public)),
                )
            })
            .collect();

        Self {
            modules,
            item_trees: ArenaMap::default(),
            root,
            prelude,
            builtin_scope,
        }
    }
}

#[tracing::instrument(skip_all, name = "hir::build_def_map")]
pub fn build_def_map<'a, R: FileResolver>(
    name: Spur,
    entry_path: &str,
    file_cache: &mut FileCache,
    packages: &Arena<PackageData>,
    dependencies: &[PackageDependency],
    string_interner: &'static ThreadedRodeo,
    resolver: &R,
) -> (DefMap, Arena<Spanned<Type>>, Vec<Diagnostic>) {
    tracing::info!("building definition map for project");
    let root = ModuleData::new();
    let def_map = DefMap::empty(root, string_interner);
    let mut types = Arena::new();
    let mut collector = DefCollector {
        def_map,
        unresolved_imports: vec![],
        diagnostics: vec![],
        string_interner,
    };
    collector.seed_with_entry(
        entry_path,
        file_cache,
        string_interner,
        &mut types,
        resolver,
    );
    collector.resolve_imports(&name, dependencies, packages);
    (collector.def_map, types, collector.diagnostics)
}

#[derive(Debug, Eq, PartialEq)]
struct Import {
    module_id: ModuleId,
    use_decl: FileItemTreeId<Use>,
    status: PartialResolvedImport,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum PartialResolvedImport {
    Unresolved(ResolvePathError),
    Resolved(ModuleItemWithVis),
}

impl PartialResolvedImport {
    fn namespaces(self) -> Option<ModuleItemWithVis> {
        match self {
            PartialResolvedImport::Unresolved(_) => None,
            PartialResolvedImport::Resolved(ns) => Some(ns),
        }
    }
}

#[derive(Debug)]
struct DefCollector {
    def_map: DefMap,
    // global_item_tree: &'a mut ItemTree,
    unresolved_imports: Vec<Import>,
    diagnostics: Vec<Diagnostic>,
    string_interner: &'static ThreadedRodeo,
}

impl DefCollector {
    fn update(
        &mut self,
        module_id: ModuleId,
        resolutions: &[(Spur, Option<ModuleItemWithVis>)],
        vis: Option<Visibility>,
    ) {
        for (name, res) in resolutions {
            if let Some((mod_def_id, mod_id, og_vis)) = res {
                let scope = &mut self.def_map.modules[module_id].scope;
                let res = match vis {
                    Some(vis) => (*mod_def_id, *mod_id, vis),
                    None => (*mod_def_id, *mod_id, *og_vis),
                };
                scope.define_item(*name, res);
            }
        }
    }

    fn resolve_imports(
        &mut self,
        package_name: &Spur,
        dependencies: &[PackageDependency],
        packages: &Arena<PackageData>,
    ) {
        let imports = std::mem::take(&mut self.unresolved_imports);

        self.unresolved_imports = imports
            .into_iter()
            .filter_map(|mut import| {
                import.status = self.resolve_import(
                    import.module_id,
                    &import,
                    package_name,
                    dependencies,
                    packages,
                );
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
                                PathResolutionResultKind::Use,
                            )
                            .to_diagnostic(),
                        );
                        Some(import)
                    }
                }
            })
            .collect();
    }

    fn resolve_import(
        &self,
        module_id: ModuleId,
        import: &Import,
        package_name: &Spur,
        dependencies: &[PackageDependency],
        packages: &Arena<PackageData>,
    ) -> PartialResolvedImport {
        let u = &self.def_map.item_trees.get(module_id).unwrap()[import.use_decl];
        // let res = self.def_map.resolve_path(&u.path, module_id);
        let res =
            self.def_map
                .resolve_path(package_name, &u.path, module_id, dependencies, packages);

        match res {
            Err(err) => PartialResolvedImport::Unresolved(err),
            Ok((_, def)) => {
                if let Some(def) = def {
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
        types: &mut Arena<Spanned<Type>>,
        resolver: &R,
    ) {
        let (file_id, content) = resolver
            .resolve_absolute_path(entry_path, file_cache)
            .unwrap();
        let (item_tree, diagnostics) =
            self.build_item_tree(file_id, &content, string_interner, types);
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
        let mut diagnostics = mod_collector.collect(item_tree.items(), file_cache, types, resolver);
        self.def_map.item_trees.insert(module_id, item_tree);
        self.diagnostics.append(&mut diagnostics);
    }

    fn build_item_tree(
        &mut self,
        file_id: FileId,
        content: &str,
        string_interner: &'static ThreadedRodeo,
        types: &mut Arena<Spanned<Type>>,
    ) -> (ItemTree, Vec<Diagnostic>) {
        let parse = flux_parser::parse(content, file_id, string_interner);
        let (root, diagnostics) = (parse.syntax(), parse.diagnostics);
        // println!("{}", root.debug(self.string_interner, true));
        let item_tree = lower_ast_to_item_tree(root, file_id, string_interner, types);
        (item_tree, diagnostics)
    }
}

struct ModCollector<'a> {
    def_collector: &'a mut DefCollector,
    module_id: ModuleId,
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
        types: &mut Arena<Spanned<Type>>,
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
                    &[(
                        name,
                        Some(ModuleItemWithVis::from((id, self.module_id, vis))),
                    )],
                    Some(vis),
                );
            };
            match item {
                crate::item_tree::ModItem::Apply(_) => {}
                crate::item_tree::ModItem::Enum(id) => {
                    let e = &self.item_tree[id];
                    update_def(
                        self.def_collector,
                        id.into(),
                        e.name.inner,
                        e.visibility.inner,
                    );
                }
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
                    self.collect_module(id, file_cache, types, resolver);
                }
                crate::item_tree::ModItem::Struct(id) => {
                    let s = &self.item_tree[id];
                    update_def(
                        self.def_collector,
                        id.into(),
                        s.name.inner,
                        s.visibility.inner,
                    );
                }
                crate::item_tree::ModItem::Trait(id) => {
                    let t = &self.item_tree[id];
                    update_def(self.def_collector, id.into(), t.name.inner, *t.visibility);
                }
                crate::item_tree::ModItem::Use(id) => {
                    self.def_collector.unresolved_imports.push(Import {
                        module_id: self.module_id,
                        use_decl: id,
                        status: PartialResolvedImport::Unresolved(
                            ResolvePathError::UnresolvedPath {
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
        types: &mut Arena<Spanned<Type>>,
        resolver: &R,
    ) {
        let (name, visibility) = {
            let module = &self.item_tree[module_id];
            (module.name.clone(), module.visibility)
        };
        let module_id = self.push_child_module(name.inner, visibility);
        let (file_id, content, mod_dir) = match self.mod_dir.resolve_declaration(
            self.file_id,
            name.map_ref(|name| self.string_interner.resolve(name))
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

        let (item_tree, mut diagnostics) =
            self.def_collector
                .build_item_tree(file_id, &content, self.string_interner, types);
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
        let mut diagnostics = mod_collector.collect(item_tree.items(), file_cache, types, resolver);
        self.def_collector
            .def_map
            .item_trees
            .insert(module_id, item_tree);
        self.diagnostics.append(&mut diagnostics);
    }

    fn push_child_module(&mut self, name: Spur, visibility: Visibility) -> ModuleId {
        let def_map = &mut self.def_collector.def_map;
        let res = def_map.modules.alloc(ModuleData::new());
        def_map.modules[res].parent = Some(self.module_id);
        def_map.modules[self.module_id].children.insert(name, res);
        let def = ModuleDefId::ModuleId(res);
        def_map.modules[self.module_id].scope.declare(def);
        self.def_collector.update(
            self.module_id,
            &[(
                name,
                Some(ModuleItemWithVis::from((def, self.module_id, visibility))),
            )],
            Some(visibility),
        );
        res
    }
}
