use super::*;

impl<'a, 'pkgs> LowerCtx<'a, 'pkgs> {
    pub(crate) fn lower_stmt(
        &mut self,
        stmt: ast::Stmt,
        generic_params: &GenericParams,
    ) -> (bool, Typed<ExprIdx>) {
        match stmt {
            ast::Stmt::LetStmt(let_stmt) => (false, self.lower_let_expr(let_stmt, generic_params)),
            ast::Stmt::ExprStmt(expr_stmt) => {
                (false, self.lower_expr_stmt(expr_stmt, generic_params))
            }
            ast::Stmt::TerminatorExprStmt(terminator_expr_stmt) => (
                true,
                self.lower_terminator_expr_stmt(terminator_expr_stmt, generic_params),
            ),
        }
    }

    // Always return unit `()`
    fn lower_let_expr(
        &mut self,
        let_stmt: ast::LetStmt,
        generic_params: &GenericParams,
    ) -> Typed<ExprIdx> {
        let name = self.lower_name(let_stmt.name());
        let ty = let_stmt
            .ty()
            .map(|ty| self.lower_type(Some(ty), generic_params).inner)
            .unwrap_or_else(|| {
                self.tckh
                    .tenv
                    .insert(Type::Unknown.file_span(self.file_id, name.span))
            });
        let val = self.lower_expr(let_stmt.value(), generic_params);
        let unification_span = self.tckh.tenv.get_filespan(&val.tid);
        self.tckh
            .unify(ty, val.tid, unification_span)
            .unwrap_or_else(|err| self.diagnostics.push(err));

        let tid = let_stmt.ty().map_or(val.tid, |_| ty);

        self.tckh.tenv.insert_local(name.inner, tid);
        self.alloc_expr(Expr::Tuple(vec![])).with_type(val.tid)
    }

    fn lower_expr_stmt(
        &mut self,
        expr_stmt: ast::ExprStmt,
        generic_params: &GenericParams,
    ) -> Typed<ExprIdx> {
        self.lower_expr(expr_stmt.expr(), generic_params)
    }

    fn lower_terminator_expr_stmt(
        &mut self,
        terminator_expr_stmt: ast::TerminatorExprStmt,
        generic_params: &GenericParams,
    ) -> Typed<ExprIdx> {
        self.lower_expr(terminator_expr_stmt.expr(), generic_params)
    }
}
