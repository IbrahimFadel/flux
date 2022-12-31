use crate::hir::{Block, ExprIdx, Let};

use super::*;

impl Context {
    pub(super) fn lower_expr(&mut self, expr: Option<ast::Expr>) -> ExprIdx {
        self.lower_node(
            expr,
            |this, expr| this.exprs.alloc(Expr::Poisoned.at(expr.range().to_span())),
            |this, expr| match expr {
                ast::Expr::BlockExpr(block) => this.lower_block_expr(block),
                ast::Expr::IntExpr(int) => this.lower_int_expr(int),
                _ => todo!(
                    "internal compiler error: unhandled expression type: {:#?}",
                    expr
                ),
            },
        )
    }

    fn lower_block_expr(&mut self, block: ast::BlockExpr) -> ExprIdx {
        let span = block.range().to_span();
        let exprs: Vec<_> = block
            .stmts()
            .map(|stmt| match stmt {
                ast::Stmt::ExprStmt(expr) => self.lower_expr(expr.expr()),
                ast::Stmt::LetStmt(let_expr) => self.lower_let_expr(let_expr),
            })
            .collect();
        let block = self.exprs.alloc(Expr::Block(Block::new(exprs)).at(span));
        block
    }

    fn lower_int_expr(&mut self, int: ast::IntExpr) -> ExprIdx {
        let span = int.range().to_span();
        let value_str = match int.v() {
            Some(v) => self
                .interner
                .resolve(&v.text_key())
                .at(v.text_range().to_span()),
            None => return self.exprs.alloc(Expr::Poisoned.at(span)),
        };
        let value: Spanned<u64> = value_str.map(|v| match v.parse() {
            Ok(v) => v,
            Err(_) => todo!(),
        });
        self.exprs.alloc(Expr::Int(value.inner).at(span))
    }

    fn lower_let_expr(&mut self, let_expr: ast::LetStmt) -> ExprIdx {
        let span = let_expr.range().to_span();
        let name = self.lower_name(let_expr.name());
        let lhs_ty = let_expr.ty().map_or(
            self.type_interner.intern(Type::Tuple(vec![])).at(span),
            |ty| self.lower_type(Some(ty), &GenericParamList::empty(), span),
        );
        let expr = self.lower_expr(let_expr.value());
        let let_expr = self
            .exprs
            .alloc(Expr::Let(Let::new(name, lhs_ty, expr)).at(span));
        let_expr
    }
}
