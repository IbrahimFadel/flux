use flux_span::Spanned;
use flux_syntax::ast;
use la_arena::Arena;

use crate::hir::{Expr, ExprIdx, Path};

pub(crate) struct LowerCtx {
    exprs: Arena<Spanned<Expr>>,
}

impl LowerCtx {
    pub fn new() -> Self {
        Self {
            exprs: Arena::new(),
        }
    }

    pub fn lower_expr(&self, expr: ast::Expr) -> ExprIdx {
        match expr {
            ast::Expr::PathExpr(path) => self.lower_path_expr(&path),
            ast::Expr::ParenExpr(_) => todo!(),
            ast::Expr::FloatExpr(_) => todo!(),
            ast::Expr::IntExpr(_) => todo!(),
            ast::Expr::BinExpr(_) => todo!(),
            ast::Expr::CallExpr(_) => todo!(),
            ast::Expr::StructExpr(_) => todo!(),
            ast::Expr::BlockExpr(_) => todo!(),
            ast::Expr::TupleExpr(_) => todo!(),
            ast::Expr::AddressExpr(_) => todo!(),
            ast::Expr::IdxExpr(_) => todo!(),
        }
    }

    fn lower_path_expr(&self, path: &ast::PathExpr) -> ExprIdx {
        todo!()
    }

    pub fn lower_path(&self, path: Option<ast::Path>) -> Path {
        todo!()
    }
}
