use flux_diagnostics::Diagnostic;
use flux_span::{FileId, WithSpan};
use la_arena::Idx;

use crate::{
    hir::ModDecl,
    item::{ItemId, ItemTreeIdx},
    name_res::{FileResolver, ModDir},
    pkg::PkgBuilder,
};

use super::ModuleId;

pub(crate) struct ModCollector<'a, 'b, R: FileResolver> {
    pub file_id: FileId,
    pub mod_dir: ModDir,
    pub mod_id: ModuleId,
    pub diagnostics: Vec<Diagnostic>,
    pub pkg_builder: &'a mut PkgBuilder<'b, R>,
}

impl<'a, 'b, R: FileResolver> ModCollector<'a, 'b, R> {
    pub fn collect(mut self, items: &[ItemId]) -> Vec<Diagnostic> {
        for item_id in items {
            match item_id.idx {
                ItemTreeIdx::Function(fn_id) => {
                    let f = &self.pkg_builder.item_tree.functions[fn_id];
                    self.pkg_builder.module_tree[item_id.mod_id].scope.declare(
                        f.name.inner,
                        f.visibility.inner,
                        item_id.clone(),
                    );
                }
                ItemTreeIdx::Module(mod_id) => {
                    self.collect_child_module(mod_id);
                }
                ItemTreeIdx::Trait(trait_id) => {
                    let trt = &self.pkg_builder.item_tree.traits[trait_id];
                    self.pkg_builder.module_tree[item_id.mod_id].scope.declare(
                        trt.name.inner,
                        trt.visibility.inner,
                        item_id.clone(),
                    );
                }
                ItemTreeIdx::Apply(_) => {}
            }
        }
        self.diagnostics
    }

    fn collect_child_module(&mut self, mod_decl_id: Idx<ModDecl>) {
        let mod_decl = &self.pkg_builder.item_tree.mods[mod_decl_id];
        let name_str = mod_decl
            .name
            .map_ref(|name| self.pkg_builder.interner.resolve(name))
            .in_file(self.file_id);
        let name_key = mod_decl.name.inner;

        let mod_resolution_result = self.mod_dir.resolve_declaration(
            self.file_id,
            &name_str,
            self.pkg_builder.source_cache,
            &self.pkg_builder.resolver,
        );
        let (file_id, file_content, mod_dir) = match mod_resolution_result {
            Ok(resolution) => resolution,
            Err(e) => {
                self.diagnostics.push(e);
                return;
            }
        };

        let (child_mod_id, items) =
            self.pkg_builder
                .new_module(file_id, &file_content, Some(self.mod_id));

        self.pkg_builder.module_tree[self.mod_id]
            .children
            .insert(name_key, child_mod_id);

        let mod_collector = ModCollector {
            file_id,
            mod_dir,
            mod_id: child_mod_id,
            diagnostics: vec![],
            pkg_builder: self.pkg_builder,
        };
        let mut diagnostics = mod_collector.collect(&items);
        self.diagnostics.append(&mut diagnostics);
    }
}
