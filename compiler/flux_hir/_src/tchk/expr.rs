use super::*;

impl<'a> TChecker<'a> {
    pub(super) fn check_expr(&mut self, idx: ExprIdx) -> Type {
        match &self.module.module.exprs[idx].inner {
            Expr::Block(block) => self.check_block_expr(block),
            Expr::Float(_) => Type::new(TypeKind::Float(None)),
            Expr::Int(_) => Type::new(TypeKind::Int(None)),
            Expr::Let(let_expr) => self.check_let_expr(let_expr),
            Expr::Path(path) => self.check_path_expr(path),
            _ => todo!(),
        }
    }

    fn check_block_expr(&mut self, block: &Block) -> Type {
        block
            .iter()
            .map(|expr| self.check_expr(*expr))
            .last()
            .unwrap_or(Type::new(TypeKind::Concrete(ConcreteKind::Tuple(vec![]))))
    }

    fn check_let_expr(&mut self, let_expr: &Let) -> Type {
        let lhs_ty = self.hir_ty_to_ts_ty(&let_expr.ty);
        let lhs_span = lhs_ty.span;
        let lhs_tyid = self.env.insert(lhs_ty.in_file(self.module.file_id));
        let rhs_ty = self.check_expr(let_expr.val.inner);
        let rhs_span = self.module.module.exprs[let_expr.val.inner].span;
        let rhs_tyid = self
            .env
            .insert(rhs_ty.in_file(self.module.file_id, rhs_span));
        self.env.push_constraint(Constraint::TypeEq(
            lhs_tyid,
            rhs_tyid,
            lhs_span.in_file(self.module.file_id),
        ));
        self.env.insert_var(let_expr.name.inner, lhs_tyid);
        Type::new(TypeKind::Concrete(ConcreteKind::Tuple(vec![])))
    }

    fn check_path_expr(&mut self, path: &Path) -> Type {
        // this is last resort if not found in function scope

        if path.len() == 1 {
            todo!()
        } else {
            let item_id = self
                .name_resolver
                .resolve_path_in_module(path, self.string_interner);

            todo!()
        }
    }
}
