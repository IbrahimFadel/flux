use tinyvec::tiny_vec;

use crate::hir::{FnDecl, Name, Param, ParamList, Type};

use super::*;

impl LoweringCtx {
    pub(crate) fn lower_fn_signature(
        &mut self,
        fn_decl: ast::FnDecl,
    ) -> (Name, Spanned<ParamList>, Spanned<Type>) {
        let name = self.lower_node(
            fn_decl.name(),
            |this, _| {
                Spanned::new(
                    this.interner.get_or_intern_static(POISONED_STRING_VALUE),
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
            Spanned::new(Type::Tuple(tiny_vec!()), param_list.span)
        };
        (name, param_list, return_ty)
    }

    pub(crate) fn lower_fn_decl(
        &mut self,
        fn_decl: ast::FnDecl,
        name: Name,
        param_list: Spanned<ParamList>,
        return_ty: Spanned<Type>,
    ) -> Option<FnDecl> {
        // let name = self.lower_node(
        //     fn_decl.name(),
        //     |this, _| {
        //         Spanned::new(
        //             this.interner.get_or_intern_static(POISONED_STRING_VALUE),
        //             this.span_node(&fn_decl),
        //         )
        //     },
        //     |this, name| Spanned::new(name.ident().unwrap().text_key(), this.span_node(&fn_decl)),
        // );
        // let param_list = self.lower_node(
        //     fn_decl.param_list(),
        //     |_, _| Spanned::new(ParamList::new(vec![]), name.span),
        //     |this, param_list| this.lower_param_list(param_list),
        // );

        // let return_ty = if let Some(return_ty) = fn_decl.return_type() {
        //     self.lower_type(return_ty)
        // } else {
        //     Spanned::new(Type::Error, param_list.span)
        // };
        self.tchk
            .tenv
            .insert(self.file_spanned(self.to_ts_ty(&return_ty)));
        // let return_ty = self.lower_node(
        //     fn_decl.return_type(),
        //     |_, _| Spanned::new(Type::Error, param_list.span),
        //     |this, return_ty| this.lower_type(return_ty),
        // );

        let body = self.lower_node(
            fn_decl.body(),
            |this, _| {
                (
                    this.exprs.alloc(Spanned::new(Expr::Error, return_ty.span)),
                    this.tchk.tenv.insert(this.file_spanned(Spanned::new(
                        ts::Type::new(TypeKind::Unknown),
                        return_ty.span,
                    ))),
                )
            },
            |this, expr| this.lower_expr(expr),
        );

        println!("{}", self.tchk.tenv);
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
            "missing parameter name".to_string(),
            param.range(),
        );
        let ty = match param.ty() {
            Some(ty) => self.lower_type(ty),
            None => {
                self.emit_diagnostic(
                    LoweringDiagnostic::Missing {
                        msg: FileSpanned::new(
                            Spanned::new("missing parameter type".to_string(), span),
                            self.file_id,
                        ),
                    }
                    .to_diagnostic(),
                );
                Spanned::new(Type::Error, span)
            }
        };
        Param { name, ty }
    }
}
