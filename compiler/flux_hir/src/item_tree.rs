use flux_span::{FileId, Interner};
use flux_syntax::{
    ast::{self, AstNode},
    SyntaxNode,
};
use flux_typesystem::TEnv;
use la_arena::Arena;

use crate::{
    hir::{ApplyDecl, FnDecl, ModDecl, TraitDecl},
    item::ItemId,
    module::ModuleId,
};

pub(crate) mod lower;

#[derive(Debug, Default)]
pub(crate) struct ItemTree {
    pub top_level: Vec<ItemId>,
    pub applies: Arena<ApplyDecl>,
    // pub enums: Arena<Enum>,
    pub functions: Arena<FnDecl>,
    pub mods: Arena<ModDecl>,
    // pub structs: Arena<Struct>,
    pub traits: Arena<TraitDecl>,
    // pub uses: Arena<Use>,
}

impl ItemTree {
    pub fn new() -> Self {
        Self {
            top_level: vec![],
            applies: Arena::new(),
            functions: Arena::new(),
            mods: Arena::new(),
            traits: Arena::new(),
        }
    }
}

pub(crate) fn lower_cst_to_item_tree(
    root: SyntaxNode,
    file_id: FileId,
    item_tree: &mut ItemTree,
    module_id: ModuleId,
    interner: &'static Interner,
    tenv: &mut TEnv,
) -> Vec<ItemId> {
    let root = ast::Root::cast(root)
        .unwrap_or_else(|| flux_diagnostics::ice("root syntax node should always cast"));
    let ctx = lower::Ctx::new(item_tree, module_id, interner, file_id, tenv);
    ctx.lower_module_items(&root)
}
