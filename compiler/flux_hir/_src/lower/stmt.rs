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
        let span = let_stmt.range().to_span();
        let name = self.lower_node(
            let_stmt.name(),
            |this, _| {
                this.interner
                    .get_or_intern_static(POISONED_STRING_VALUE)
                    .at(span)
            },
            |this, name| name.ident().unwrap().text_key().at(span),
        );

        let (ty_id, ty_idx) = if let Some(ty) = let_stmt.ty() {
            let ty = self.lower_type(ty, generic_param_list);
            (
                self.tchk
                    .tenv
                    .insert(self.to_ts_ty(ty).in_file(self.file_id)),
                ty,
            )
        } else {
            (
                self.tchk.tenv.insert(
                    ts::Type::new(TypeKind::Unknown)
                        .in_file(self.file_id, self.span_node(&let_stmt)),
                ),
                self.types.alloc(Type::Tuple(tiny_vec!()).at(span)),
            )
        };

        let (value, value_ty_id) = self.lower_node(
            let_stmt.value(),
            |this, _| {
                (
                    this.exprs.alloc(Expr::Error.at(span)),
                    this.tchk
                        .tenv
                        .insert(ts::Type::new(TypeKind::Unknown).in_file(this.file_id, span)),
                )
            },
            |this, expr| this.lower_expr(expr, generic_param_list),
        );

        let result = self
            .tchk
            .unify(ty_id, value_ty_id, span.in_file(self.file_id));
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
        let span = expr.range().to_span();
        let (idx, ty_id) = self.lower_node(
            expr.expr(),
            |this, _| {
                let expr = Expr::Error.at(span);
                let ty_id = this
                    .tchk
                    .tenv
                    .insert(ts::Type::new(TypeKind::Unknown).in_file(this.file_id, span));
                let idx = this.exprs.alloc(expr);
                (idx, ty_id)
            },
            |this, expr| this.lower_expr(expr, generic_param_list),
        );
        let has_semicolon = expr.semicolon().is_some();
        (Stmt::ExprStmt(idx), has_semicolon, ty_id)
    }
}
