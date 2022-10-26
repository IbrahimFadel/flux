use flux_diagnostics::Diagnostic;
use flux_span::{FileId, FileSpanned, Spanned};
use flux_syntax::{
    ast::{AstNode, Root},
    SyntaxNode,
};
use flux_typesystem::TypeId;
use hir::{FnDecl, GenericParamList, Name, ParamList, StructDecl, Type};
use lasso::ThreadedRodeo;
use lower::LoweringCtx;
use tinyvec::{tiny_vec, TinyVec};

use crate::hir::WhereClause;

mod diagnostics;
mod hir;
mod lower;

#[derive(Debug)]
pub struct Module {
    pub functions: Vec<FnDecl>,
    pub structs: Vec<StructDecl>,
}

pub fn lower_to_hir(
    root: SyntaxNode,
    file_id: FileId,
    interner: &'static ThreadedRodeo,
) -> (Module, Vec<Diagnostic>) {
    let mut ctx = LoweringCtx::new(file_id, interner);
    let root = Root::cast(root).expect("internal compiler error: Root node should always cast");

    let structs: Vec<_> = root
        .struct_decls()
        .map(|struct_decl| ctx.lower_struct_decl(struct_decl))
        .collect();

    let fn_signatures: Vec<(
        Name,
        GenericParamList,
        Spanned<ParamList>,
        Spanned<Type>,
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
                    .insert(FileSpanned::new(ctx.to_ts_ty(&param.ty), file_id))
            })
            .collect();
        let param_types = FileSpanned::new(Spanned::new(param_types, params.span), file_id);
        let return_type_id = ctx
            .tchk
            .tenv
            .insert(FileSpanned::new(ctx.to_ts_ty(return_ty), file_id));
        let return_ty = FileSpanned::new(Spanned::new(return_type_id, return_ty.span), file_id);
        ctx.tchk
            .tenv
            .insert_function_signature(vec![name.inner], (param_types, return_ty));
    }

    let functions: Vec<FnDecl> = root
        .fn_decls()
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
        .collect();

    let module = Module { functions, structs };

    // println!("{:#?}", module);

    (module, ctx.diagnostics)
}
