use itertools::Itertools;

use crate::hir::{FnDecl, Param, ParamList, Type};

use super::*;

impl LoweringCtx {
    pub(crate) fn lower_fn_decl(&mut self, fn_decl: ast::FnDecl) -> Option<FnDecl> {
        let name = self.lower_node(
            fn_decl.name(),
            |this, _| {
                Spanned::new(
                    this.interner.get_or_intern(POISONED_STRING_VALUE),
                    this.span_node(&fn_decl),
                )
            },
            |this, name| Spanned::new(name.ident().unwrap().text_key(), this.span_node(&fn_decl)),
        );
        let param_list = self.lower_node(
            fn_decl.param_list(),
            |_, _| Spanned::new(ParamList::new(vec![]), name.span),
            |this, param_list| this.lower_param_list(param_list),
        );

        let return_ty = if let Some(return_ty) = fn_decl.return_type() {
            self.lower_type(return_ty)
        } else {
            Spanned::new(Type::Error, param_list.span)
        };
        self.tenv
            .insert(self.to_ts_ty(&return_ty.inner, return_ty.span));
        // let return_ty = self.lower_node(
        //     fn_decl.return_type(),
        //     |_, _| Spanned::new(Type::Error, param_list.span),
        //     |this, return_ty| this.lower_type(return_ty),
        // );

        let body = self.lower_node(
            fn_decl.body(),
            |this, _| {
                (
                    Spanned::new(Expr::Error, return_ty.span),
                    this.tenv
                        .insert(ts::Type::new(TypeKind::Unknown, return_ty.span)),
                )
            },
            |this, expr| this.lower_expr(expr),
        );

        println!("{}", self.tenv);
        None
    }

    fn lower_param_list(&mut self, param_list: ast::ParamList) -> Spanned<ParamList> {
        let mut params = vec![];
        for param in param_list.params() {
            params.push(self.lower_param(param));
        }
        Spanned::new(ParamList::new(params), Span::new(param_list.range()))
    }

    fn lower_param(&mut self, param: ast::Param) -> Param {
        let span = self.span_node(&param);
        let name = self.unwrap_token(
            param.name(),
            format!("missing parameter name"),
            param.range(),
        );
        let ty = match param.ty() {
            Some(ty) => self.lower_type(ty),
            None => {
                self.emit_diagnostic(LoweringDiagnostic::Missing {
                    msg: FileSpanned::new(
                        Spanned::new(format!("missing parameter type"), span),
                        self.file_id,
                    ),
                });
                Spanned::new(Type::Error, span)
            }
        };
        Param { name, ty }
    }
}
