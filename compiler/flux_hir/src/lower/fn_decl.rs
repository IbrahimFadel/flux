use tinyvec::tiny_vec;

use crate::hir::{FnDecl, GenericParamList, Name, Param, ParamList, Type, WhereClause};

use super::*;

impl LoweringCtx {
    pub(crate) fn lower_fn_signature(
        &mut self,
        fn_decl: ast::FnDecl,
    ) -> (
        Name,
        GenericParamList,
        Spanned<ParamList>,
        Spanned<Type>,
        WhereClause,
    ) {
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
        let generic_param_list = fn_decl
            .generic_param_list()
            .map_or(GenericParamList::empty(), |generic_param_list| {
                self.lower_generic_param_list(generic_param_list)
            });
        let where_clause = fn_decl
            .where_clause()
            .map_or(WhereClause::EMPTY, |where_clause| {
                self.lower_where_clause(where_clause, &generic_param_list)
            });
        let param_list = self.lower_node(
            fn_decl.param_list(),
            |_, _| Spanned::new(ParamList::new(vec![]), name.span),
            |this, param_list| this.lower_param_list(param_list, &generic_param_list),
        );
        let return_ty = if let Some(return_ty) = fn_decl.return_type() {
            self.lower_type(return_ty, &generic_param_list)
        } else {
            Spanned::new(Type::Tuple(tiny_vec!()), param_list.span)
        };
        (
            name,
            generic_param_list,
            param_list,
            return_ty,
            where_clause,
        )
    }

    pub(crate) fn lower_fn_decl(
        &mut self,
        fn_decl: ast::FnDecl,
        name: Name,
        generic_param_list: GenericParamList,
        param_list: Spanned<ParamList>,
        return_ty: Spanned<Type>,
        where_clause: WhereClause,
    ) -> FnDecl {
        let return_ty_id = self
            .tchk
            .tenv
            .insert(self.file_spanned(self.to_ts_ty(&return_ty)));

        let (body, body_ty_id) = self.lower_node(
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
            |this, expr| this.lower_expr(expr, &generic_param_list),
        );

        let result = self
            .tchk
            .unify(return_ty_id, body_ty_id, self.file_span(return_ty.span));
        self.maybe_emit_diagnostic(result);

        FnDecl::new(
            name,
            generic_param_list,
            param_list,
            return_ty,
            where_clause,
            body,
        )
    }

    fn lower_param_list(
        &mut self,
        param_list: ast::ParamList,
        generic_param_list: &GenericParamList,
    ) -> Spanned<ParamList> {
        let mut params = vec![];
        for param in param_list.params() {
            params.push(self.lower_param(param, generic_param_list));
        }
        Spanned::new(ParamList::new(params), Span::new(param_list.range()))
    }

    fn lower_param(&mut self, param: ast::Param, generic_param_list: &GenericParamList) -> Param {
        let span = self.span_node(&param);
        let name = self.unwrap_token(
            param.name(),
            "missing parameter name".to_string(),
            param.range(),
        );
        let ty = match param.ty() {
            Some(ty) => self.lower_type(ty, generic_param_list),
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
