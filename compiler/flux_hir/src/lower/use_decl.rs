use crate::hir::{Path, UseDecl};

use super::*;

impl LoweringCtx {
    pub(crate) fn lower_use_decl(&mut self, use_decl: ast::UseDecl) -> UseDecl {
        // let span = use_decl.range().to_span();
        let path = self.lower_node(
            use_decl.path(),
            |_, _| Path::poisoned(),
            |this, path| this.lower_path(path.segments()),
        );
        let alias = use_decl
            .alias()
            .map(|name| name.ident().unwrap().text_key().at(name.range().to_span()));
        UseDecl::new(path, alias)
    }
}
