use crate::hir::{Block, Call, ExprIdx, Float, Int};

use super::*;

type ExprResult = (ExprIdx, TypeId);

impl LoweringCtx {
    pub(crate) fn lower_expr(&mut self, expr: ast::Expr) -> ExprResult {
        match expr {
            ast::Expr::BlockExpr(block) => self.lower_block_expr(block),
            ast::Expr::CallExpr(call) => self.lower_call_expr(call),
            ast::Expr::IntExpr(int) => self.lower_int_expr(int),
            ast::Expr::FloatExpr(float) => self.lower_float_expr(float),
            ast::Expr::PathExpr(path) => self.lower_path_expr(path),
            _ => todo!("{:#?}", expr),
        }
    }

    fn lower_exprs(&mut self, exprs: impl Iterator<Item = ast::Expr>) -> Vec<ExprResult> {
        exprs.map(|expr| self.lower_expr(expr)).collect()
    }

    fn lower_block_expr(&mut self, block: ast::BlockExpr) -> ExprResult {
        let mut stmts = vec![];
        let mut ty = self
            .tchk
            .tenv
            .insert_unit(self.file_span(self.span_node(&block)));
        for stmt in block.stmts() {
            let (s, had_semicolon, stmt_ty) = self.lower_stmt(stmt);
            stmts.push(s);
            if !had_semicolon {
                ty = stmt_ty;
                break;
            }
        }
        let expr = Spanned::new(Expr::Block(Block::new(stmts)), self.span_node(&block));
        let block_id = self.exprs.alloc(expr);
        (block_id, ty)
    }

    fn lower_call_expr(&mut self, call: ast::CallExpr) -> ExprResult {
        let span = self.span_node(&call);
        println!("PATH: {:?}", call.path());
        let path = self.lower_node(
            call.path(),
            |_, _| Spanned::new(Path::poisoned(span), span),
            |this, path| {
                let span = Span::new(path.range());
                let path = this.lower_path(path.segments());
                println!("path hehe: {:?}", path);
                Spanned::new(path, span)
            },
        );
        println!("path: {:?}", path);
        let args = self.lower_node(
            call.args(),
            |_, _| vec![],
            |this, arg_list| {
                this.lower_exprs(arg_list.args())
                    .into_iter()
                    .map(|(idx, _)| idx)
                    .collect()
            },
        );
        let signature_result = self
            .tchk
            .tenv
            .get_function_signature(&self.file_spanned(path.map_ref(|path| path.get_spurs())));
        let return_ty_id = self.maybe_emit_diagnostic_with(
            signature_result,
            |this| this.tchk.tenv.insert_unknown(this.file_span(span)),
            |_, (_, return_ty_id)| return_ty_id,
        );
        let call = Call { path, args };
        let expr = Spanned::new(Expr::Call(call), span);
        let idx = self.exprs.alloc(expr);
        (idx, return_ty_id)
    }

    fn lower_int_expr(&mut self, int: ast::IntExpr) -> ExprResult {
        let v = int
            .v()
            .expect("internal compiler error: missing value token in int expression");
        let num_str = self.interner.resolve(&v.text_key());
        let num_int = match num_str.parse::<u64>() {
            Ok(v) => v,
            Err(err) => {
                self.emit_diagnostic(
                    LoweringDiagnostic::CouldNotParseInt {
                        span: self.file_span(self.span_node(&int)),
                        msg: err.to_string(),
                    }
                    .to_diagnostic(),
                );
                u64::MAX
            }
        };
        let expr = Spanned::new(Expr::Int(Int::new(num_int)), self.span_node(&int));
        let expr_id = self.exprs.alloc(expr);
        let ty_id = self.tchk.tenv.insert(self.file_spanned(Spanned::new(
            ts::Type::new(TypeKind::Int(None)),
            self.span_node(&int),
        )));
        (expr_id, ty_id)
    }

    fn lower_float_expr(&mut self, float: ast::FloatExpr) -> ExprResult {
        let span = self.span_node(&float);
        let v = float
            .v()
            .expect("internal compiler error: missing value token in float expression");
        let num_str = self.interner.resolve(&v.text_key());
        let num_float = match num_str.parse::<f64>() {
            Ok(v) => v,
            Err(err) => {
                self.emit_diagnostic(
                    LoweringDiagnostic::CouldNotParseInt {
                        span: self.file_span(span),
                        msg: err.to_string(),
                    }
                    .to_diagnostic(),
                );
                f64::MAX
            }
        };
        let expr = Spanned::new(Expr::Float(Float::new(num_float)), span);
        let expr_id = self.exprs.alloc(expr);
        let ty_id = self
            .tchk
            .tenv
            .insert(self.file_spanned(Spanned::new(ts::Type::new(TypeKind::Float(None)), span)));
        (expr_id, ty_id)
    }

    fn lower_path_expr(&mut self, path: ast::PathExpr) -> ExprResult {
        let hir_path = self.lower_path(path.segments());
        let spurs = hir_path.get_spurs();
        let ty_id = if spurs.len() > 1 {
            unimplemented!()
        } else {
            self.tchk
                .tenv
                .get_path_typeid(
                    self.file_spanned(Spanned::new(*spurs.first().unwrap(), self.span_node(&path))),
                )
                .map_or_else(
                    |diagnostic| {
                        self.emit_diagnostic(diagnostic);
                        self.tchk
                            .tenv
                            .insert_unknown(self.file_span(self.span_node(&path)))
                    },
                    |result| result,
                )
        };
        let expr = Spanned::new(Expr::Path(hir_path), self.span_node(&path));
        let expr_id = self.exprs.alloc(expr);
        (expr_id, ty_id)
    }
}
