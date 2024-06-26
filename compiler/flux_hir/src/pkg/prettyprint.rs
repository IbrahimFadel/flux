use ascii_tree::{write_tree, Tree};
use cstree::interning::TokenKey;
use flux_span::Interner;
use itertools::Itertools;
use pretty::RcDoc;

use crate::{
    hir::{FnDecl, GenericParams, Param, ParamList, TraitDecl, Visibility},
    item_tree::ItemTree,
    module::{ModuleData, ModuleTree},
};

use super::Package;

impl Package {
    pub fn to_doc(&self, interner: &'static Interner) -> RcDoc {
        RcDoc::text("+-------------+\n| Module Tree |\n+-------------+\n")
            .append(self.module_tree.to_doc(interner))
            .append(RcDoc::text(
                "+-------------+\n|  Item Tree  |\n+-------------+\n",
            ))
            .append(self.item_tree.to_doc(&self.module_tree, interner))
    }

    pub fn to_pretty(&self, width: usize, interner: &'static Interner) -> String {
        let mut w = Vec::new();
        self.to_doc(interner).render(width, &mut w).unwrap();
        String::from_utf8(w).unwrap()
    }
}

impl ItemTree {
    pub fn to_doc<'a>(&'a self, module_tree: &'a ModuleTree, interner: &'static Interner) -> RcDoc {
        let root = module_tree
            .get()
            .values()
            .find(|module| module.parent.is_none())
            .expect("missing root module");
        let name = interner.get_or_intern_static("main");
        root.to_doc(&[name], self, module_tree, interner)
        // RcDoc::intersperse(self.top_level.iter().map(|item_id| RcDoc::nil()), "\n")
    }
}

impl Visibility {
    pub fn to_doc(&self) -> RcDoc {
        match self {
            Self::Private => RcDoc::nil(),
            Self::Public => RcDoc::text("pub").append(RcDoc::space()),
        }
    }
}

impl FnDecl {
    pub fn to_doc(&self, interner: &'static Interner) -> RcDoc {
        self.visibility
            .to_doc()
            .append(RcDoc::text("fn"))
            .append(RcDoc::space())
            .append(RcDoc::text(interner.resolve(&self.name)))
            .append(self.generic_params.params_to_doc(interner))
            .append(self.params.to_doc(interner))
    }
}

impl GenericParams {
    pub fn params_to_doc(&self, interner: &'static Interner) -> RcDoc {
        if self.types.is_empty() {
            RcDoc::nil()
        } else {
            let names = self
                .types
                .iter()
                .map(|(_, name)| RcDoc::text(interner.resolve(&name)));
            RcDoc::text("<")
                .append(RcDoc::intersperse(names, RcDoc::text(", ")))
                .append(">")
        }
    }

    pub fn where_clause_to_doc(&self, interner: &'static Interner) -> RcDoc {
        RcDoc::nil()
    }
}

impl ParamList {
    pub fn to_doc(&self, interner: &'static Interner) -> RcDoc {
        let params = if self.is_empty() {
            RcDoc::nil()
        } else {
            RcDoc::intersperse(
                self.iter().map(|param| param.to_doc(interner)),
                RcDoc::text(", "),
            )
        };

        RcDoc::text("(").append(params).append(RcDoc::text(")"))
    }
}

impl Param {
    pub fn to_doc(&self, interner: &'static Interner) -> RcDoc {
        RcDoc::text(interner.resolve(&self.name))
            .append(RcDoc::space())
            .append(RcDoc::text("TODO"))
    }
}

impl TraitDecl {
    pub fn to_doc(&self, interner: &'static Interner) -> RcDoc {
        RcDoc::text("trait")
    }
}

impl ModuleTree {
    pub fn to_doc(&self, interner: &'static Interner) -> RcDoc {
        RcDoc::text(self.to_string(interner))
    }

    pub fn to_string(&self, interner: &'static Interner) -> String {
        let root = self
            .get()
            .values()
            .find(|module| module.parent.is_none())
            .expect("missing root module");

        let tree = root.to_tree(self, &interner.get_or_intern_static("main"), interner);

        let mut s = String::new();
        write_tree(&mut s, &tree).unwrap();
        s
    }
}

impl ModuleData {
    fn to_tree(
        &self,
        module_tree: &ModuleTree,
        name: &TokenKey,
        interner: &'static Interner,
    ) -> Tree {
        let name = interner.resolve(name).to_string();
        if self.children.is_empty() {
            Tree::Leaf(vec![name])
        } else {
            Tree::Node(
                name,
                self.children
                    .iter()
                    .map(|(name, mod_id)| module_tree[*mod_id].to_tree(module_tree, name, interner))
                    .collect(),
            )
        }
    }

    pub fn to_doc<'a>(
        &'a self,
        path: &[TokenKey],
        item_tree: &'a ItemTree,
        module_tree: &'a ModuleTree,
        interner: &'static Interner,
    ) -> RcDoc {
        let name = path.iter().map(|key| interner.resolve(key)).join("::");
        let items = RcDoc::intersperse(
            self.scope
                .items
                .iter()
                .map(|(_, (_, item_id))| match item_id.idx {
                    crate::item::ItemTreeIdx::Apply(_) => todo!(),
                    crate::item::ItemTreeIdx::Function(fn_decl) => {
                        item_tree.functions[fn_decl].to_doc(interner)
                    }
                    crate::item::ItemTreeIdx::Module(_) => todo!(),
                    crate::item::ItemTreeIdx::Trait(trait_decl) => {
                        item_tree.traits[trait_decl].to_doc(interner)
                    }
                }),
            RcDoc::hardline().append(RcDoc::hardline()),
        );
        let children = self.children.iter().map(|(name, mod_id)| {
            let path = &[path, &[*name]].concat();
            module_tree[*mod_id].to_doc(path, item_tree, module_tree, interner)
        });
        RcDoc::text(name)
            .append(RcDoc::line())
            .append(items)
            .append(RcDoc::intersperse(children, RcDoc::hardline()))
    }
}
