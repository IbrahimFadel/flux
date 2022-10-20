use flux_diagnostics::Diagnostic;
use flux_span::{FileId, FileSpanned, Spanned};
use flux_syntax::{
    ast::{AstNode, Root},
    SyntaxNode,
};
use flux_typesystem::TypeId;
use hir::{FnDecl, Name, ParamList, Type};
use lasso::ThreadedRodeo;
use lower::LoweringCtx;
use tinyvec::{tiny_vec, TinyVec};

mod diagnostics;
mod hir;
mod lower;

#[derive(Debug)]
pub struct Module {
    pub functions: Vec<FnDecl>,
}

pub fn lower_to_hir(
    root: SyntaxNode,
    file_id: FileId,
    interner: &'static ThreadedRodeo,
) -> (Module, Vec<Diagnostic>) {
    let mut ctx = LoweringCtx::new(file_id, interner);
    let root = Root::cast(root).expect("internal compiler error: Root node should always cast");

    let fn_signatures: Vec<(Name, Spanned<ParamList>, Spanned<Type>)> = root
        .fn_decls()
        .map(|fn_decl| ctx.lower_fn_signature(fn_decl))
        .collect();
    for (name, params, return_ty) in &fn_signatures {
        let param_types: TinyVec<[TypeId; 2]> = params
            .iter()
            .map(|param| {
                ctx.tchk
                    .tenv
                    .insert(FileSpanned::new(ctx.to_ts_ty(&param.ty), file_id))
            })
            .collect();
        let return_ty = ctx
            .tchk
            .tenv
            .insert(FileSpanned::new(ctx.to_ts_ty(return_ty), file_id));
        ctx.tchk
            .tenv
            .insert_function_signature(tiny_vec!(name.inner), (param_types, return_ty));
    }

    let functions: Vec<FnDecl> = root
        .fn_decls()
        .zip(fn_signatures.into_iter())
        .filter_map(|(fn_decl, (name, params, return_ty))| {
            ctx.lower_fn_decl(fn_decl, name, params, return_ty)
        })
        .collect();

    let module = Module { functions };

    (module, ctx.diagnostics)
}
