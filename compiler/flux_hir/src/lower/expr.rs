use lasso::Spur;
use ts::r#type::StructConcreteKind;

use crate::hir::{
    Block, Call, ExprIdx, Float, GenericParamList, Int, Struct, StructExprFieldAssignment,
};

use super::*;

type ExprResult = (ExprIdx, TypeId);

impl LoweringCtx {
    pub(crate) fn lower_expr(
        &mut self,
        expr: ast::Expr,
        generic_param_list: &GenericParamList,
    ) -> ExprResult {
        match expr {
            ast::Expr::BlockExpr(block) => self.lower_block_expr(block, generic_param_list),
            ast::Expr::CallExpr(call) => self.lower_call_expr(call, generic_param_list),
            ast::Expr::IntExpr(int) => self.lower_int_expr(int),
            ast::Expr::FloatExpr(float) => self.lower_float_expr(float),
            ast::Expr::PathExpr(path) => self.lower_path_expr(path),
            ast::Expr::StructExpr(strukt) => self.lower_struct_expr(strukt, generic_param_list),
            _ => todo!("{:#?}", expr),
        }
    }

    fn lower_exprs(
        &mut self,
        exprs: impl Iterator<Item = ast::Expr>,
        generic_param_list: &GenericParamList,
    ) -> Vec<ExprResult> {
        exprs
            .map(|expr| self.lower_expr(expr, generic_param_list))
            .collect()
    }

    fn lower_block_expr(
        &mut self,
        block: ast::BlockExpr,
        generic_param_list: &GenericParamList,
    ) -> ExprResult {
        let mut stmts = vec![];
        let mut ty = self
            .tchk
            .tenv
            .insert_unit(self.file_span(self.span_node(&block)));
        for stmt in block.stmts() {
            let (s, had_semicolon, stmt_ty) = self.lower_stmt(stmt, generic_param_list);
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

    fn lower_call_expr(
        &mut self,
        call: ast::CallExpr,
        generic_param_list: &GenericParamList,
    ) -> ExprResult {
        let span = self.span_node(&call);
        let path = self.lower_node(
            call.path(),
            |_, _| Spanned::new(Path::poisoned(span), span),
            |this, path| {
                let span = Span::new(path.range());
                let path = this.lower_path(path.segments());
                Spanned::new(path, span)
            },
        );
        let args = self.lower_node(
            call.args(),
            |_, _| vec![],
            |this, arg_list| this.lower_exprs(arg_list.args(), generic_param_list),
        );
        let signature_result = self.tchk.tenv.get_function_signature(
            &self.file_spanned(path.map_ref(|path| path.get_unspanned_spurs())),
        );
        let (param_ty_ids, return_ty_id) = self.maybe_emit_diagnostic_with(
            signature_result,
            |this| {
                (
                    this.file_spanned(Spanned::new(vec![], span)),
                    FileSpanned::new(
                        Spanned::new(this.tchk.tenv.insert_unknown(this.file_span(span)), span),
                        this.file_id,
                    ),
                )
            },
            |_, signature| signature,
        );

        let lparen = call.lparen();
        let rparen = call.rparen();
        let args_span: Option<Span> =
            Span::span_iter_of_spanned(args.iter().map(|(idx, _)| &self.exprs[*idx]));
        let args_span = match (lparen, rparen) {
            (Some(lparen), Some(rparen)) => Span::new(TextRange::new(
                lparen.text_range().start(),
                rparen.text_range().end(),
            )),
            (Some(lparen), _) => {
                if let Some(args_span) = args_span {
                    Span::new(TextRange::new(
                        lparen.text_range().start(),
                        args_span.range.end(),
                    ))
                } else {
                    Span::new(lparen.text_range())
                }
            }
            (_, Some(rparen)) => {
                if let Some(args_span) = args_span {
                    Span::new(TextRange::new(
                        args_span.range.start(),
                        rparen.text_range().end(),
                    ))
                } else {
                    Span::new(rparen.text_range())
                }
            }
            (_, _) => args_span.unwrap_or(path.span),
        };

        self.tychk_call_expr_args(&path, &args, args_span, &param_ty_ids, param_ty_ids.span);

        let call = Call {
            path,
            args: args.into_iter().map(|(idx, _)| idx).collect(),
        };
        let expr = Spanned::new(Expr::Call(call), span);
        let idx = self.exprs.alloc(expr);
        (idx, return_ty_id.inner.inner)
    }

    fn tychk_call_expr_args(
        &mut self,
        path: &Spanned<Path>,
        args: &[(ExprIdx, TypeId)],
        args_span: Span,
        params: &[TypeId],
        params_span: Span,
    ) {
        let args_len = args.len();
        let params_len = params.len();

        let expected_number = self.file_spanned(Spanned::new(params_len, params_span));
        let got_number = self.file_spanned(Spanned::new(args_len, args_span));

        if args_len != params_len {
            self.emit_diagnostic(
                LoweringDiagnostic::IncorrectNumberOfArgsInCall {
                    call_path: self
                        .file_spanned(path.map_ref(|path| path.to_string(self.interner))),
                    expected_number,
                    got_number,
                }
                .to_diagnostic(),
            );
        }

        args.iter()
            .zip(params.iter())
            .for_each(|((arg_idx, arg_id), param_id)| {
                let unification_span = self.file_span(self.exprs[*arg_idx].span);
                let result = self.tchk.unify(*param_id, *arg_id, unification_span);
                self.maybe_emit_diagnostic(result);
            });
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
        let spurs = hir_path.get_unspanned_spurs();
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

    fn lower_struct_expr(
        &mut self,
        strukt: ast::StructExpr,
        generic_param_list: &GenericParamList,
    ) -> ExprResult {
        let (path, generic_arg_list) = self.lower_node(
            strukt.path(),
            |_, _| (Path::poisoned(Span::new(strukt.range())), None),
            |this, path| (this.lower_path(path.segments()), path.generic_arg_list()),
        );
        let args =
            generic_arg_list.map_or(Spanned::new(vec![], self.span_node(&strukt)), |arg_list| {
                Spanned::new(
                    arg_list
                        .args()
                        .map(|arg| self.lower_type(arg, generic_param_list))
                        .collect(),
                    Span::new(arg_list.range()),
                )
            });
        let struct_span = self.span_node(&strukt);

        let struct_field_types_result =
            self.tchk
                .tenv
                .get_struct_field_types(self.file_spanned(Spanned::new(
                    path.iter().map(|spur| spur.inner),
                    struct_span,
                )));
        let (struct_concrete_kind, struct_decl_span) = self.maybe_emit_diagnostic_with(
            struct_field_types_result,
            |this| (StructConcreteKind::EMPTY, this.file_span(struct_span)),
            |_, strukt| strukt,
        );

        let type_name = FileSpanned::new(
            Spanned::new(path.to_string(self.interner), struct_decl_span.inner),
            struct_decl_span.file_id,
        );

        let decl_generic_params_span = struct_concrete_kind
            .generic_params
            .iter()
            .map(|id| self.tchk.tenv.get_type_filespan(*id).inner);
        let decl_generic_params_span =
            Span::span_iter_of_span(decl_generic_params_span).unwrap_or(type_name.span);
        let decl_generic_params_len = struct_concrete_kind.generic_params.len();
        let generic_args_len = args.len();
        if decl_generic_params_len != generic_args_len {
            // People can have 0 args if they want to rely on type inference
            if !(decl_generic_params_len > 0 && generic_args_len == 0) {
                self.emit_diagnostic(
                    LoweringDiagnostic::IncorrectNumberOfTypeArgs {
                        type_decl: type_name.clone(),
                        num_params: FileSpanned::new(
                            Spanned::new(decl_generic_params_len, decl_generic_params_span),
                            struct_decl_span.file_id,
                        ),
                        num_args: self.file_spanned(Spanned::new(generic_args_len, args.span)),
                    }
                    .to_diagnostic(),
                )
            }
        }

        let fields = self.lower_node(
            strukt.field_list(),
            |_, _| vec![],
            |this, fields| {
                this.lower_struct_expr_field_list(
                    fields,
                    generic_param_list,
                    struct_concrete_kind.fields.iter(),
                    type_name,
                )
            },
        );

        let ty = ts::Type::with_params(
            TypeKind::Concrete(ConcreteKind::Path(path.get_unspanned_spurs())),
            args.iter().map(|arg| self.to_ts_tykind(*arg).inner),
        );
        let ty_id = self
            .tchk
            .tenv
            .insert(self.file_spanned(Spanned::new(ty, struct_span)));

        let strukt = Struct::new(path, args.inner, fields);
        let expr = Spanned::new(Expr::Struct(strukt), struct_span);
        let expr_id = self.exprs.alloc(expr);

        (expr_id, ty_id)
    }

    fn lower_struct_expr_field_list<'a>(
        &mut self,
        field_list: ast::StructExprFieldList,
        generic_param_list: &GenericParamList,
        mut struct_field_types: impl Iterator<Item = &'a (Spur, TypeId)>,
        struct_decl: FileSpanned<String>,
    ) -> Vec<StructExprFieldAssignment> {
        field_list
            .fields()
            .map(|field| {
                let name = self.unwrap_token(
                    field.name(),
                    "expected name in struct expression field assignment",
                    field.range(),
                );
                let (val, val_id) = self.lower_node(
                    field.val(),
                    |this, _| {
                        let field_span = this.span_node(&field);
                        (
                            this.exprs.alloc(Spanned::new(Expr::Error, field_span)),
                            this.tchk.tenv.insert(this.file_spanned(Spanned::new(
                                ts::Type::new(TypeKind::Unknown),
                                field_span,
                            ))),
                        )
                    },
                    |this, expr| this.lower_expr(expr, generic_param_list),
                );

                let expected_field_ty =
                    struct_field_types.find(|(field_name, _)| *field_name == name.inner);
                match expected_field_ty {
                    Some((_, expected_field_ty)) => {
                        let unification_result =
                            self.tchk
                                .unify(*expected_field_ty, val_id, self.file_span(name.span));
                        self.maybe_emit_diagnostic(unification_result);
                    }
                    None => self.emit_diagnostic(
                        LoweringDiagnostic::UnknownFieldInStructExpr {
                            unknown_field: self.file_spanned(
                                name.map_ref(|spur| self.interner.resolve(spur).to_string()),
                            ),
                            struct_definition: struct_decl.clone(),
                        }
                        .to_diagnostic(),
                    ),
                };

                StructExprFieldAssignment::new(name, val)
            })
            .collect()
    }
}
