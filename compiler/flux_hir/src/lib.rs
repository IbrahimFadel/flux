use flux_diagnostics::Diagnostic;
use flux_span::{FileId, FileSpanned, Spanned};
use flux_syntax::{
    ast::{AstNode, Root},
    SyntaxNode,
};
use flux_typesystem::TypeId;
use hir::{
    ApplyDecl, EnumDecl, FnDecl, FnDeclFirstPass, GenericParamList, ModDecl, Name, ParamList,
    StructDecl, TraitDecl, TypeIdx, UseDecl,
};
use la_arena::{Arena, Idx};
use lasso::ThreadedRodeo;
use lower::LoweringCtx;
use tracing::{debug, info, instrument};

use crate::hir::WhereClause;

mod diagnostics;
mod hir;
mod lower;

#[derive(Debug)]
pub struct Module {
    pub uses: Vec<UseDecl>,
    pub mods: Vec<ModDecl>,
    pub functions: Vec<FnDecl>,
    pub enums: Vec<EnumDecl>,
    pub structs: Vec<StructDecl>,
    pub traits: Vec<TraitDecl>,
    pub applies: Vec<ApplyDecl>,
}

fn lower_functions(ctx: &mut LoweringCtx, root: Root, file_id: FileId) -> Vec<FnDecl> {
    let fn_signatures: Vec<(
        Name,
        Spanned<GenericParamList>,
        Spanned<ParamList>,
        TypeIdx,
        WhereClause,
    )> = root
        .fn_decls()
        .map(|fn_decl| ctx.lower_fn_signature(fn_decl))
        .collect();
    for (name, _, params, return_ty, _) in &fn_signatures {
        let param_types: Vec<TypeId> = params
            .iter()
            .map(|param| {
                ctx.tchk
                    .tenv
                    .insert(FileSpanned::new(ctx.to_ts_ty(param.ty), file_id))
            })
            .collect();
        let param_types = FileSpanned::new(Spanned::new(param_types, params.span), file_id);
        let return_type_id = ctx
            .tchk
            .tenv
            .insert(FileSpanned::new(ctx.to_ts_ty(*return_ty), file_id));
        let return_ty = FileSpanned::new(
            Spanned::new(return_type_id, ctx.types[*return_ty].span),
            file_id,
        );
        ctx.tchk
            .tenv
            .insert_function_signature(vec![name.inner], (param_types, return_ty));
    }

    root.fn_decls()
        .zip(fn_signatures.into_iter())
        .map(
            |(fn_decl, (name, generic_param_list, params, return_ty, where_clause))| {
                ctx.lower_fn_decl(
                    fn_decl,
                    name,
                    generic_param_list,
                    params,
                    return_ty,
                    where_clause,
                )
            },
        )
        .collect()
}

#[derive(Debug, Default)]
pub struct ItemTree {
    pub uses: Arena<UseDecl>,
    pub mods: Arena<ModDecl>,
    pub enums: Arena<EnumDecl>,
    pub structs: Arena<StructDecl>,
    pub functions: Arena<FnDeclFirstPass>,
}

/// Since we typecheck in the same pass as typechecking, we need access to all items across all modules before we start lowering item bodies.
/// This pass lowers items but not their bodies. ie. types, traits (with method signatures, **not** bodies), and function signatures.
/// Using this initial pass we can then lower item bodies and typecheck without missing any information (like what the return type of a function in another file is).
/// Furthermore, this pass will collect all the mod statements which will allow the driver to determine what files to parse and lower.
#[instrument(level = "info", skip(root, interner, item_tree))]
pub fn lower_items(
    root: SyntaxNode,
    file_id: FileId,
    interner: &'static ThreadedRodeo,
    item_tree: &mut ItemTree,
) -> Vec<Idx<ModDecl>> {
    let mut ctx = LoweringCtx::new(file_id, interner);
    let root = Root::cast(root).expect("internal compiler error: Root node should always cast");
    root.use_decls().for_each(|use_decl| {
        item_tree.uses.alloc(ctx.lower_use_decl(use_decl));
    });
    let mods = root
        .mod_decls()
        .map(|mod_decl| item_tree.mods.alloc(ctx.lower_mod_decl(mod_decl)))
        .collect();
    root.enum_decls().for_each(|enum_decl| {
        item_tree.enums.alloc(ctx.lower_enum_decl(enum_decl));
    });
    root.struct_decls().for_each(|struct_decl| {
        item_tree.structs.alloc(ctx.lower_struct_decl(struct_decl));
    });
    root.fn_decls().for_each(|fn_decl| {
        item_tree.functions.alloc(ctx.lower_fn_signature(fn_decl));
    });
    debug!("lowered items without bodies");
    mods
}

pub fn lower_to_hir(
    root: SyntaxNode,
    file_id: FileId,
    interner: &'static ThreadedRodeo,
) -> (Module, Vec<Diagnostic>) {
    let mut ctx = LoweringCtx::new(file_id, interner);
    let root = Root::cast(root).expect("internal compiler error: Root node should always cast");
    let uses: Vec<_> = root
        .use_decls()
        .map(|use_decl| ctx.lower_use_decl(use_decl))
        .collect();
    let mods: Vec<_> = root
        .mod_decls()
        .map(|mod_decl| ctx.lower_mod_decl(mod_decl))
        .collect();
    let enums: Vec<_> = root
        .enum_decls()
        .map(|enum_decl| ctx.lower_enum_decl(enum_decl))
        .collect();
    let structs: Vec<_> = root
        .struct_decls()
        .map(|struct_decl| ctx.lower_struct_decl(struct_decl))
        .collect();
    let traits: Vec<_> = root
        .trait_decls()
        .map(|trait_decl| ctx.lower_trait_decl(trait_decl))
        .collect();
    let applies: Vec<_> = root
        .apply_decls()
        .map(|apply_decl| ctx.lower_apply_decl(apply_decl))
        .collect();
    let functions = lower_functions(&mut ctx, root, file_id);
    let module = Module {
        uses,
        mods,
        functions,
        enums,
        structs,
        traits,
        applies,
    };
    (module, ctx.diagnostics)
}
