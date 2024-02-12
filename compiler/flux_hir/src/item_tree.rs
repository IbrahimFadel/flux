use flux_span::InputFile;
use flux_syntax::{
    ast::{self, AstNode},
    SyntaxNode,
};
use la_arena::Arena;

use crate::{hir::Function, item::ItemId};

mod lower;

#[derive(Debug, Default)]
pub(crate) struct ItemTree {
    pub top_level: Vec<ItemId>,
    // pub applies: Arena<Apply>,
    // pub enums: Arena<Enum>,
    pub functions: Arena<Function>,
    // pub mods: Arena<Mod>,
    // pub structs: Arena<Struct>,
    // pub traits: Arena<Trait>,
    // pub uses: Arena<Use>,
}

impl ItemTree {
    pub fn new() -> Self {
        Self {
            top_level: vec![],
            functions: Arena::new(),
        }
    }
}

enum Item {
    Function(Function),
}

pub(crate) fn lower_cst_to_item_tree(
    root: SyntaxNode,
    file: InputFile,
    item_tree: &mut ItemTree,
    // tenv: &mut TEnv,
    // module_xid: ModuleId,
) -> Vec<ItemId> {
    let root = ast::Root::cast(root)
        .unwrap_or_else(|| flux_diagnostics::ice("root syntax node should always cast"));
    // let packages = &Arena::new();
    // let ctx = lower::Ctx::new(
    //     file_id,
    //     string_interner,
    //     packages,
    //     tenv,
    //     item_tree,
    //     module_id,
    // );
    // ctx.lower_module_items(&root)
    vec![]
}
