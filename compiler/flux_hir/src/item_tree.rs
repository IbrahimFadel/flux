use flux_span::{FileId, Interner};
use flux_syntax::{
    ast::{self, AstNode},
    SyntaxNode,
};
use flux_typesystem::TEnv;
use la_arena::Arena;

use crate::{
    hir::{ApplyDecl, EnumDecl, FnDecl, ModDecl, StructDecl, TraitDecl, UseDecl},
    item::ItemId,
    module::{ModuleId, ModuleTree},
};

pub(crate) mod lower;

#[derive(Debug, Default)]
pub(crate) struct ItemTree {
    pub top_level: Vec<ItemId>,
    pub applies: Arena<ApplyDecl>,
    pub enums: Arena<EnumDecl>,
    pub functions: Arena<FnDecl>,
    pub mods: Arena<ModDecl>,
    pub structs: Arena<StructDecl>,
    pub traits: Arena<TraitDecl>,
    pub uses: Arena<UseDecl>,
}

impl ItemTree {
    pub fn new() -> Self {
        Self {
            top_level: vec![],
            applies: Arena::new(),
            enums: Arena::new(),
            functions: Arena::new(),
            mods: Arena::new(),
            structs: Arena::new(),
            traits: Arena::new(),
            uses: Arena::new(),
        }
    }
}

pub(crate) fn lower_cst_to_item_tree(
    root: SyntaxNode,
    file_id: FileId,
    item_tree: &mut ItemTree,
    module_tree: &mut ModuleTree,
    module_id: ModuleId,
    interner: &'static Interner,
    tenv: &mut TEnv,
) -> Vec<ItemId> {
    let root = ast::Root::cast(root)
        .unwrap_or_else(|| flux_diagnostics::ice("root syntax node should always cast"));
    let ctx = lower::Ctx::new(item_tree, module_tree, module_id, tenv, interner, file_id);
    ctx.lower_module_items(&root)
}
