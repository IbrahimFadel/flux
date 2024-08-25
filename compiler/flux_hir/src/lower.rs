use std::mem;

use flux_diagnostics::{ice, Diagnostic, SourceCache, ToDiagnostic};
use flux_id::{
    id::{self, InPkg},
    Map,
};
use flux_parser::{
    ast::{self, AstNode},
    syntax::SyntaxNode,
};
use flux_typesystem::{diagnostics::TypeError, TEnv, ThisCtx, TraitResolver, Typed};
use flux_util::{FileId, Interner, WithSpan};

use crate::{
    def::{expr::Expr, item::ApplyDecl},
    fmt::format_function_with_types,
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
    trait_resolver: &TraitResolver,
    packages: &Map<id::Pkg, Package>,
    exprs: &mut Map<id::Expr, Typed<Expr>>,
    interner: &'static Interner,
    diagnostics: &mut Vec<Diagnostic>,
    source_cache: &SourceCache,
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
            trait_resolver,
            exprs,
            interner,
            diagnostics,
            source_cache,
        ),
        // ItemTreeIdx::BuiltinType(_) => todo!(),
        // ItemTreeIdx::Enum(_) => todo!(),
        ItemTreeIdx::Function(function_id) => lower_function_body(
            None,
            *function_id,
            &ctx,
            &item_resolver,
            trait_resolver,
            exprs,
            interner,
            diagnostics,
            source_cache,
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
    trait_resolver: &TraitResolver,
    exprs: &mut Map<id::Expr, Typed<Expr>>,
    interner: &'static Interner,
    diagnostics: &mut Vec<Diagnostic>,
    source_cache: &SourceCache,
) {
    let apply_decl = ctx.item_tree.applies.get(apply_id);

    apply_decl.methods.iter().for_each(|method_id| {
        lower_function_body(
            Some(apply_decl),
            *method_id,
            ctx,
            item_resolver,
            trait_resolver,
            exprs,
            interner,
            diagnostics,
            source_cache,
        );
    });
}

fn lower_function_body(
    apply_decl: Option<&ApplyDecl>,
    function_id: id::FnDecl,
    ctx: &LoweringCtx,
    item_resolver: &ItemResolver,
    trait_resolver: &TraitResolver,
    exprs: &mut Map<id::Expr, Typed<Expr>>,
    interner: &'static Interner,
    diagnostics: &mut Vec<Diagnostic>,
    source_cache: &SourceCache,
) {
    let fn_decl = ctx.item_tree.functions.get(function_id);
    let mut tenv = TEnv::new(trait_resolver, interner);

    let this_ctx = apply_decl.map_or(ThisCtx::Function, |apply_decl| {
        let assoc_types: Vec<_> = apply_decl
            .assoc_types
            .iter()
            .map(|assoc_type| (assoc_type.name.inner, assoc_type.ty.kind.clone()))
            .collect();

        match apply_decl.trt {
            Some(_) => {
                ThisCtx::TraitApplication(Box::new(apply_decl.to_ty.kind.clone()), assoc_types)
            }
            None => ThisCtx::TypeApplication(Box::new(apply_decl.to_ty.kind.clone())),
        }
    });
    tenv.set_this_ctx(this_ctx.clone());

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
        this_ctx,
        item_resolver,
        interner,
        diagnostics,
    );
    let body = expr_lowerer.lower(ast.body(), &fn_decl.generic_params);
    let return_ty = fn_decl.return_ty.clone();
    let return_ty = expr_lowerer.tenv.insert(return_ty);

    let body_tid = exprs.get(body).tid;
    tenv.add_equality(body_tid, return_ty);

    let exprs_copy = exprs.clone();

    let unresolved_expressions = loop {
        let initial_unresolved_expressions = mem::take(exprs.as_mut());
        let num_initial_unresolved_expressions = initial_unresolved_expressions.len();

        let final_unresolved_expressions: Vec<_> = initial_unresolved_expressions
            .into_iter()
            .filter(|unresolved_expr| tenv.resolve(unresolved_expr.tid).is_err())
            .collect();

        // println!(
        //     "{} vs {}",
        //     final_unresolved_expressions.len(),
        //     num_initial_unresolved_expressions
        // );

        if final_unresolved_expressions.len() == 0
            || final_unresolved_expressions.len() == num_initial_unresolved_expressions
        {
            break final_unresolved_expressions;
        }

        *exprs.as_mut() = final_unresolved_expressions;
    };

    for expr in exprs_copy.values() {
        // println!("resolving {:?}", expr.inner);
        match tenv.resolve(expr.tid) {
            Ok(tkind) => {}
            Err(err) => {
                diagnostics.push(
                    TypeError::CouldNotInfer {
                        ty: (),
                        ty_file_span: tenv.get_span(expr.tid).in_file(ctx.file_id),
                    }
                    .to_diagnostic(),
                );
            }
        }
    }

    format_function_with_types(body, &exprs_copy, &mut tenv, source_cache, ctx.file_id);
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
