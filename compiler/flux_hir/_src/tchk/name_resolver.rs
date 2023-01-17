use flux_span::{Span, WithSpan};
use hashbrown::HashMap;
use itertools::Itertools;
use la_arena::Idx;
use lasso::{Spur, ThreadedRodeo};
use tracing::info;

use crate::{
    hir::{ItemDefinitionId, Path},
    package_defs::{ModuleData, ModuleID, PackageDefs},
    Module,
};

#[derive(Debug)]
pub struct NameResolver {
    interner: &'static ThreadedRodeo,
    // Vec<(ModuleID, Spur)> or Vec<Vec<Spur>> indexed by ModuleID?
    pub use_paths: Vec<(ModuleID, Spur)>,
    cached_resolved_paths: HashMap<Spur, ItemDefinitionId>,
    pub absolute_path_map: HashMap<Spur, (ModuleID, Option<ItemDefinitionId>)>,
}

impl NameResolver {
    pub fn new(interner: &'static ThreadedRodeo) -> Self {
        Self {
            interner,
            use_paths: vec![],
            cached_resolved_paths: HashMap::new(),
            absolute_path_map: HashMap::new(),
        }
    }

    pub fn from_package_defs(package_defs: &PackageDefs, interner: &'static ThreadedRodeo) -> Self {
        let mut absolute_path_map = HashMap::new();
        let mut use_paths = vec![];

        for (m, item_scope) in package_defs.modules.iter() {
            let path = package_defs
                .module_tree
                .get_absolute_path_of_module(m, interner);
            // let spur = interner.get_or_intern(&path.iter().join("::"));
            let spur = path.to_spur(interner);

            absolute_path_map.insert(spur, (m, None));

            for u_id in item_scope.uses.iter() {
                let module_data = package_defs.get_module_data(m);
                let u_id = match u_id {
                    ItemDefinitionId::UseId(id) => id,
                    _ => unreachable!(),
                };
                let path = &module_data.module.uses[*u_id].path;
                // let path = relative_path_in_module_to_absolute_path(
                //     package_defs,
                //     &use_paths,
                //     m,
                //     path,
                //     interner,
                // );
                use_paths.push((m, path.to_spur(interner)));
            }

            for (name, f) in item_scope.functions.iter() {
                let mut path = path.clone();
                path.push(name.at(Span::new(0..0)));
                // let spur = interner.get_or_intern(&path.iter().join("::"));
                absolute_path_map.insert(spur, (m, Some(*f)));
            }
        }

        Self {
            interner,
            use_paths,
            cached_resolved_paths: HashMap::new(),
            absolute_path_map,
        }
    }

    pub fn resolve_path_in_module(
        &self,
        package_defs: &PackageDefs,
        module: ModuleID,
        path: &Path,
        interner: &'static ThreadedRodeo,
    ) -> Option<ItemDefinitionId> {
        info!(
            path = interner.resolve(&path.to_spur(interner)),
            "resolving path"
        );

        let absolute_path_of_module = package_defs
            .module_tree
            .get_absolute_path_of_module(module, interner);

        for (module_id, use_path) in &self.use_paths {
            if *module_id != module {
                continue;
            }
            if *use_path == path.to_spur(interner) {
                if path.nth(0).unwrap().inner == interner.get_or_intern_static("pkg") {
                    return package_defs.get_item_definition_id_with_absolute_path(path);
                } else {
                    // absolute_path_of_module
                }
            }
        }

        todo!()
    }
}

#[cfg(test)]
mod test {
    use flux_diagnostics::reporting::FileCache;
    use flux_parser::parse;
    use flux_span::{FileId, Span, WithSpan};
    use lasso::ThreadedRodeo;
    use once_cell::sync::Lazy;

    use crate::{hir::Path, lower_ast_to_hir, package_defs::PackageDefs, Module};

    use super::NameResolver;

    static INTERNER: Lazy<ThreadedRodeo> = Lazy::new(ThreadedRodeo::new);

    const root_content: &str = r#"
fn main() {
}    
"#;

    const foo_content: &str = r#"
fn foo() -> i32 {
    0
}
"#;

    fn generate_hir_module(
        content: &str,
        path: &str,
        file_cache: &mut FileCache,
    ) -> (FileId, Module) {
        let file_id = file_cache.add_file(path, root_content);
        let result = parse(&root_content, file_id, &INTERNER);
        let (root, diagnostics) = (result.syntax(), result.diagnostics);
        assert_eq!(diagnostics.len(), 0);
        (file_id, lower_ast_to_hir(root, &INTERNER))
    }

    #[test]
    fn foo() {
        let mut file_cache = FileCache::new(&INTERNER);
        let mut package_defs = PackageDefs::new();

        let (root_file_id, root_module) =
            generate_hir_module(root_content, "./main.flx", &mut file_cache);
        let root_module_id = package_defs.add_root_module(
            root_module,
            INTERNER.get_or_intern_static("pkg"),
            root_file_id,
        );

        let (foo_file_id, foo_module) =
            generate_hir_module(foo_content, "./foo.flx", &mut file_cache);
        let foo_module_id = package_defs.add_module(
            foo_module,
            INTERNER.get_or_intern_static("foo"),
            root_module_id,
            foo_file_id,
        );
        package_defs.add_child_module(
            root_module_id,
            INTERNER.get_or_intern_static("foo"),
            foo_module_id,
        );

        println!("{}", package_defs.fmt(&INTERNER));

        let name_resolver = NameResolver::from_package_defs(&package_defs, &INTERNER);
        println!("{:#?}", name_resolver);

        let res = name_resolver.resolve_absolute_path_in_module(
            foo_module_id,
            &Path::from_str_static("pkg::foo::foo".at(Span::new(0..0)), &INTERNER),
        );
        assert!(res.is_some());
        let res = name_resolver.resolve_absolute_path_in_module(
            foo_module_id,
            &Path::from_str_static("pkg::foo".at(Span::new(0..0)), &INTERNER),
        );
        assert!(res.is_none());
        // let name_resolver = NameResolver::new(&INTERNER);
        // let res = name_resolver.resolve_absolute_path_in_module(
        //     root_module_id,
        //     &Path::from_str_static("pkg::foo::foo".at(Span::new(0..0)), &INTERNER),
        // );
        // println!("{:?}", res);

        todo!()
    }
}
