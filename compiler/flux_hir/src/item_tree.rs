use flux_diagnostics::{ice, Diagnostic};
use flux_id::{id, Map};
use flux_span::{FileId, Interner};
use flux_syntax::{
    ast::{self, AstNode},
    SyntaxNode,
};

use crate::{
    hir::{ApplyDecl, EnumDecl, FnDecl, ModDecl, StructDecl, TraitDecl, UseDecl},
    item::ItemId,
};

pub(crate) mod lower;

#[derive(Debug, Default, Clone)]
pub struct ItemTree {
    pub top_level: Vec<ItemId>,
    pub applies: Map<id::ApplyDecl, ApplyDecl>,
    pub enums: Map<id::EnumDecl, EnumDecl>,
    pub functions: Map<id::FnDecl, FnDecl>,
    pub mods: Map<id::ModDecl, ModDecl>,
    pub structs: Map<id::StructDecl, StructDecl>,
    pub traits: Map<id::TraitDecl, TraitDecl>,
    pub uses: Map<id::UseDecl, UseDecl>,
}

impl ItemTree {
    pub fn new() -> Self {
        Self {
            top_level: vec![],
            applies: Map::new(),
            enums: Map::new(),
            functions: Map::new(),
            mods: Map::new(),
            structs: Map::new(),
            traits: Map::new(),
            uses: Map::new(),
        }
    }
}

pub(crate) fn lower_cst_to_item_tree(
    root: SyntaxNode,
    file_id: FileId,
    module_id: id::Mod,
    item_tree: &mut ItemTree,
    diagnostics: &mut Vec<Diagnostic>,
    interner: &'static Interner,
) -> Vec<ItemId> {
    let root = ast::Root::cast(root).unwrap_or_else(|| ice("root syntax node should always cast"));
    let ctx = lower::LoweringCtx::new(item_tree, interner, file_id, module_id, diagnostics);
    ctx.lower_module_items(&root)
}
