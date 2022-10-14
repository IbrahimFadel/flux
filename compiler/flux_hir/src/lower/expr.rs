use super::*;

type ExprResult = (Spanned<Expr>, TypeId);

impl LoweringCtx {
    pub(crate) fn lower_expr(&mut self, expr: ast::Expr) -> ExprResult {
        match expr {
            ast::Expr::BlockExpr(_) => (
                Spanned::new(
                    Expr::Path(Path::from_segments(std::iter::once(Spanned::new(
                        Spur::default(),
                        self.span_node(&expr),
                    )))),
                    self.span_node(&expr),
                ),
                TypeId::new(0),
            ),
            ast::Expr::PathExpr(path) => self.lower_path_expr(path),
            _ => todo!(),
        }
    }

    fn lower_path_expr(&mut self, path: ast::PathExpr) -> ExprResult {
        let hir_path = self.lower_path(path.segments());
        let ty_id = self.tenv.insert(ts::Type::new(
            TypeKind::Concrete(ConcreteKind::Path(hir_path.get_spurs())),
            self.span_node(&path),
        ));
        (
            Spanned::new(Expr::Path(hir_path), self.span_node(&path)),
            ty_id,
        )
    }
}
