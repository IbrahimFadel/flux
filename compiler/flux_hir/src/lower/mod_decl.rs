use crate::hir::{ModDecl, Visibility};

use super::*;

impl LoweringCtx {
    pub(crate) fn lower_mod_decl(&mut self, mod_decl: ast::ModDecl) -> ModDecl {
        let span = mod_decl.range().to_span();
        let visibility = mod_decl
            .visibility()
            .map_or(Visibility::Private, |vis|self.lower_visibility(vis));
        let name = self.lower_node(
            mod_decl.name(),
            |this, _| {
                this.interner
                    .get_or_intern_static(POISONED_STRING_VALUE)
                    .at(span)
            },
            |_, name| name.ident().unwrap().text_key().at(name.range().to_span()),
        );
        ModDecl::new(visibility, name)
    }
}
