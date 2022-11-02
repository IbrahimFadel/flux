use tinyvec::tiny_vec;

use crate::hir::{GenericParamList, LetStmt, Stmt};

use super::*;

/// The lowered [`Stmt`], whether or not the [`ast::Stmt`] had a semicolon (needed for
/// typechecking), and its type
type StmtResult = (Stmt, bool, TypeId);

impl LoweringCtx {
    pub(crate) fn lower_stmt(
        &mut self,
        stmt: ast::Stmt,
        generic_param_list: &GenericParamList,
    ) -> StmtResult {
        match stmt {
            ast::Stmt::LetStmt(let_stmt) => self.lower_let_stmt(let_stmt, generic_param_list),
            ast::Stmt::ExprStmt(expr) => self.lower_expr_stmt(expr, generic_param_list),
        }
    }

    fn lower_let_stmt(
        &mut self,
        let_stmt: ast::LetStmt,
        generic_param_list: &GenericParamList,
    ) -> StmtResult {
        let name = self.lower_node(
            let_stmt.name(),
            |this, _| {
                Spanned::new(
                    this.interner.get_or_intern_static(POISONED_STRING_VALUE),
                    this.span_node(&let_stmt),
                )
            },
            |this, name| Spanned::new(name.ident().unwrap().text_key(), this.span_node(&let_stmt)),
        );

        let (ty_id, ty_idx) = if let Some(ty) = let_stmt.ty() {
            let ty = self.lower_type(ty, generic_param_list);
            (
                self.tchk.tenv.insert(self.file_spanned(self.to_ts_ty(ty))),
                ty,
            )
        } else {
            (
                self.tchk.tenv.insert(self.file_spanned(Spanned::new(
                    ts::Type::new(TypeKind::Unknown),
                    self.span_node(&let_stmt),
                ))),
                self.types.alloc(Spanned::new(
                    Type::Tuple(tiny_vec!()),
                    self.span_node(&let_stmt),
                )),
            )
        };

        let (value, value_ty_id) = self.lower_node(
            let_stmt.value(),
            |this, _| {
                (
                    this.exprs
                        .alloc(Spanned::new(Expr::Error, this.span_node(&let_stmt))),
                    this.tchk.tenv.insert(this.file_spanned(Spanned::new(
                        ts::Type::new(TypeKind::Unknown),
                        this.span_node(&let_stmt),
                    ))),
                )
            },
            |this, expr| this.lower_expr(expr, generic_param_list),
        );

        let result = self.tchk.unify(
            ty_id,
            value_ty_id,
            self.file_span(self.span_node(&let_stmt)),
        );
        self.maybe_emit_diagnostic(result);

        self.tchk.tenv.insert_local_to_scope(name.inner, ty_id);

        (
            Stmt::LetStmt(LetStmt {
                name,
                ty: ty_idx,
                value,
            }),
            true,
            ty_id,
        )
    }

    fn lower_expr_stmt(
        &mut self,
        expr: ast::ExprStmt,
        generic_param_list: &GenericParamList,
    ) -> StmtResult {
        let (idx, ty_id) = self.lower_node(
            expr.expr(),
            |this, _| {
                let expr = Spanned::new(Expr::Error, this.span_node(&expr));
                let ty_id = this.tchk.tenv.insert(
                    this.file_spanned(Spanned::new(ts::Type::new(TypeKind::Unknown), expr.span)),
                );
                let idx = this.exprs.alloc(expr);
                (idx, ty_id)
            },
            |this, expr| this.lower_expr(expr, generic_param_list),
        );
        let has_semicolon = expr.semicolon().is_some();
        (Stmt::ExprStmt(idx), has_semicolon, ty_id)
    }
}
