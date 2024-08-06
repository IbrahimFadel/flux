use flux_diagnostics::{ice, Diagnostic};
use flux_id::{
    id::{self, InPkg},
    Map,
};
use flux_parser::{
    ast::{self, AstNode},
    syntax::SyntaxNode,
};
use flux_typesystem::{TEnv, ThisCtx, Type, TypeKind};
use flux_util::{FileId, Interner};

use crate::{
    def::{expr::Expr, item::ApplyDecl},
    item::{ItemId, ItemTreeIdx},
    name_res::item::ItemResolver,
    Package,
};

use self::item_tree::ItemTree;

mod expr;
pub mod item_tree;
mod r#type;

struct LoweringCtx<'a> {
    package_id: id::Pkg,
    packages: &'a Map<id::Pkg, Package>,
    item_tree: &'a ItemTree,
    mod_id: id::Mod,
    file_id: FileId,
}

pub(super) fn lower_cst_to_item_tree(
    root: SyntaxNode,
    file_id: FileId,
    module_id: id::Mod,
    item_tree: &mut ItemTree,
    interner: &'static Interner,
    diagnostics: &mut Vec<Diagnostic>,
) -> Vec<ItemId> {
    let root = ast::Root::cast(root).unwrap_or_else(|| ice("root syntax node should always cast"));
    item_tree::LoweringCtx::new(file_id, module_id, item_tree, interner, diagnostics)
        .lower_module_items(&root)
}

pub(super) fn lower_item_bodies(
    mod_id: InPkg<id::Mod>,
    item_id: &ItemId,
    packages: &Map<id::Pkg, Package>,
    exprs: &mut Map<id::Expr, Expr>,
    interner: &'static Interner,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let item_tree = &packages.get(mod_id.pkg_id).item_tree;
    let file_id = packages.get(mod_id.pkg_id).module_tree[*mod_id].file_id;
    let ctx = LoweringCtx {
        package_id: mod_id.pkg_id,
        packages,
        item_tree,
        mod_id: *mod_id,
        file_id,
    };

    let item_resolver = ItemResolver::new(mod_id.pkg_id, packages, interner);
    match &item_id.inner {
        ItemTreeIdx::Apply(apply_id) => lower_apply_bodies(
            *apply_id,
            &ctx,
            &item_resolver,
            exprs,
            interner,
            diagnostics,
        ),
        // ItemTreeIdx::BuiltinType(_) => todo!(),
        // ItemTreeIdx::Enum(_) => todo!(),
        ItemTreeIdx::Function(function_id) => lower_function_body(
            None,
            *function_id,
            &ctx,
            &item_resolver,
            exprs,
            interner,
            diagnostics,
        ),
        // ItemTreeIdx::Module(_) => todo!(),
        // ItemTreeIdx::Struct(_) => todo!(),
        // ItemTreeIdx::Trait(_) => todo!(),
        // ItemTreeIdx::Use(_) => todo!(),
        _ => {}
    }
}

fn lower_apply_bodies(
    apply_id: id::ApplyDecl,
    ctx: &LoweringCtx,
    item_resolver: &ItemResolver,
    exprs: &mut Map<id::Expr, Expr>,
    interner: &'static Interner,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let apply_decl = ctx.item_tree.applies.get(apply_id);

    apply_decl.methods.iter().for_each(|method_id| {
        lower_function_body(
            Some(apply_decl),
            *method_id,
            ctx,
            item_resolver,
            exprs,
            interner,
            diagnostics,
        );
    });
}

fn lower_function_body(
    apply_decl: Option<&ApplyDecl>,
    function_id: id::FnDecl,
    ctx: &LoweringCtx,
    item_resolver: &ItemResolver,
    exprs: &mut Map<id::Expr, Expr>,
    interner: &'static Interner,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let fn_decl = ctx.item_tree.functions.get(function_id);
    let mut tenv = TEnv::new(interner);

    if let Some(apply_decl) = apply_decl {
        let this = tenv.insert(apply_decl.to_ty.clone());

        let assoc_types = apply_decl
            .assoc_types
            .iter()
            .map(|assoc_type| (assoc_type.name.inner, tenv.insert(assoc_type.ty.clone())))
            .collect();

        tenv.set_this_ctx(ThisCtx::new(this, assoc_types));
    }

    fn_decl.params.iter().for_each(|param| {
        let tid = tenv.insert(param.ty.clone());
        tenv.insert_local(param.name.inner, tid);
    });

    let ast = fn_decl
        .ast
        .as_ref()
        .unwrap_or_else(|| ice("`FnDecl` should have an `ast` field"));

    let mut expr_lowerer = expr::LoweringCtx::new(
        ctx.file_id,
        ctx.mod_id,
        exprs,
        ctx.packages,
        &mut tenv,
        item_resolver,
        interner,
        diagnostics,
    );
    let body = expr_lowerer.lower(ast.body(), &fn_decl.generic_params);

    let return_ty = fn_decl.return_ty.clone();
    let return_ty = expr_lowerer.tenv.insert(return_ty);

    tenv.add_equality(body.tid, return_ty);

    // println!("Function: `{}`\n{tenv}", interner.resolve(&fn_decl.name));

    let body_ty = tenv
        .resolve(body.tid, ctx.file_id)
        .map_err(|errs| diagnostics.extend(errs))
        .ok();

    if let Some(body_ty) = body_ty {
        println!("Resolved body to: {}", tenv.fmt_concrete_kind(&body_ty));
    }
}

/*
    During parsing, when there is an error we generate a diagnostic, and poison the AST node
    To avoid duplicating such errors, we assert that anything that produced an error will be poisoned, or else it is an ICE
    This utility function will lower the node, or return a default value if the node is poisoned
*/
fn lower_node<C, N, T, P, F>(ctx: &C, node: Option<N>, poison: P, normal: F) -> T
where
    N: AstNode,
    P: FnOnce(&C, N) -> T,
    F: FnOnce(&C, N) -> T,
{
    let n = node.unwrap_or_else(|| ice("missing node that should always be emitted"));
    if n.is_poisoned() {
        poison(ctx, n)
    } else {
        normal(ctx, n)
    }
}

fn lower_node_mut<C, N, T, P, F>(ctx: &mut C, node: Option<N>, poison: P, normal: F) -> T
where
    N: AstNode,
    P: FnOnce(&mut C, N) -> T,
    F: FnOnce(&mut C, N) -> T,
{
    let n = node.unwrap_or_else(|| ice("missing node that should always be emitted"));
    if n.is_poisoned() {
        poison(ctx, n)
    } else {
        normal(ctx, n)
    }
}

fn lower_optional_node_mut<C, N, T, P, F>(
    ctx: &mut C,
    node: Option<N>,
    poison_function: P,
    normal_function: F,
) -> T
where
    N: AstNode,
    P: FnOnce(&mut C) -> T,
    F: FnOnce(&mut C, N) -> T,
{
    match node {
        Some(n) => {
            if n.is_poisoned() {
                poison_function(ctx)
            } else {
                normal_function(ctx, n)
            }
        }
        None => poison_function(ctx),
    }
}
