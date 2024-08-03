use flux_diagnostics::{ice, Diagnostic};
use flux_id::id;
use flux_util::{FileId, WithSpan};

use crate::{
    def::item::Visibility,
    item::{ItemId, ItemTreeIdx},
    name_res::{FileResolver, ModDir},
    package::PkgBuilder,
};

pub(crate) struct ModCollector<'a, 'b, R: FileResolver> {
    pub file_id: FileId,
    pub mod_dir: ModDir,
    pub mod_id: id::Mod,
    pub diagnostics: Vec<Diagnostic>,
    pub pkg_builder: &'a mut PkgBuilder<'b, R>,
}

impl<'a, 'b, R: FileResolver> ModCollector<'a, 'b, R> {
    pub fn collect(mut self, items: &[ItemId]) -> Vec<Diagnostic> {
        for item_id in items {
            match item_id.inner {
                ItemTreeIdx::Function(fn_id) => {
                    let f = self.pkg_builder.item_tree.functions.get(fn_id);
                    self.pkg_builder.module_tree[item_id.mod_id].scope.declare(
                        f.name.inner,
                        f.visibility.inner,
                        item_id.clone(),
                    );
                }
                ItemTreeIdx::BuiltinType(_) => {}
                ItemTreeIdx::Module(mod_id) => {
                    let m = self.pkg_builder.item_tree.mods.get(mod_id);
                    self.pkg_builder.module_tree[item_id.mod_id].scope.declare(
                        m.name.inner,
                        m.visibility.inner,
                        item_id.clone(),
                    );
                    self.collect_child_module(mod_id);
                }
                ItemTreeIdx::Struct(struct_id) => {
                    let s = self.pkg_builder.item_tree.structs.get(struct_id);
                    self.pkg_builder.module_tree[item_id.mod_id].scope.declare(
                        s.name.inner,
                        s.visibility.inner,
                        item_id.clone(),
                    );
                }
                ItemTreeIdx::Trait(trait_id) => {
                    let trt = self.pkg_builder.item_tree.traits.get(trait_id);
                    self.pkg_builder.module_tree[item_id.mod_id].scope.declare(
                        trt.name.inner,
                        trt.visibility.inner,
                        item_id.clone(),
                    );
                }
                ItemTreeIdx::Apply(_) => {}
                ItemTreeIdx::Enum(enum_id) => {
                    let e = self.pkg_builder.item_tree.enums.get(enum_id);
                    self.pkg_builder.module_tree[item_id.mod_id].scope.declare(
                        e.name.inner,
                        e.visibility.inner,
                        item_id.clone(),
                    );
                }
                ItemTreeIdx::Use(use_id) => {
                    let u = self.pkg_builder.item_tree.uses.get(use_id);
                    let name = u
                        .alias
                        .as_ref()
                        .map(|a| a.inner)
                        .or_else(|| u.path.last().copied())
                        .unwrap_or_else(|| ice("no name for import"));

                    self.pkg_builder.module_tree[item_id.mod_id].scope.declare(
                        name,
                        Visibility::Public,
                        item_id.clone(),
                    );
                }
            }
        }
        self.diagnostics
    }

    fn collect_child_module(&mut self, mod_decl_id: id::ModDecl) {
        let mod_decl = &self.pkg_builder.item_tree.mods.get(mod_decl_id);
        let name_str = mod_decl
            .name
            .as_ref()
            .map(|name| self.pkg_builder.interner.resolve(name))
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
