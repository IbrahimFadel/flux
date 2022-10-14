use std::convert::identity;

use flux_diagnostics::{Diagnostic, ToDiagnostic};
use flux_span::FileId;
use flux_syntax::{
    ast::{AstNode, Root},
    SyntaxNode,
};
use hir::FnDecl;
use lasso::ThreadedRodeo;
use lower::LoweringCtx;

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

    let fn_decls: Vec<FnDecl> = root
        .fn_decls()
        .map(|fn_decl| ctx.lower_fn_decl(fn_decl))
        .filter_map(identity)
        .collect();

    let module = Module {
        functions: fn_decls,
    };

    (
        module,
        ctx.diagnostics
            .iter()
            .map(|diagnostic| diagnostic.to_diagnostic())
            .collect(),
    )
}
