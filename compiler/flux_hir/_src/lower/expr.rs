use std::convert::identity;

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
            .insert_unit(block.range().to_span().in_file(self.file_id));
        for stmt in block.stmts() {
            let (s, had_semicolon, stmt_ty) = self.lower_stmt(stmt, generic_param_list);
            stmts.push(s);
            if !had_semicolon {
                ty = stmt_ty;
                break;
            }
        }
        let expr = Expr::Block(Block::new(stmts)).at(block.range().to_span());
        let block_id = self.exprs.alloc(expr);
        (block_id, ty)
    }

    fn lower_call_expr(
        &mut self,
        call: ast::CallExpr,
        generic_param_list: &GenericParamList,
    ) -> ExprResult {
        let span = call.range().to_span();
        let path = self.lower_node(
            call.path(),
            |_, _| Path::poisoned().at(span),
            |this, path| {
                let span = path.range().to_span();
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
            &path
                .map_ref(|path| path.get_unspanned_spurs())
                .in_file(self.file_id),
        );
        let (param_ty_ids, return_ty_id) = self.maybe_emit_diagnostic_with(
            signature_result,
            |this| {
                (
                    vec![].in_file(this.file_id, span),
                    this.tchk
                        .tenv
                        .insert_unknown(span.in_file(this.file_id))
                        .in_file(this.file_id, span),
                )
            },
            |_, signature| signature,
        );

        let lparen = call.lparen();
        let rparen = call.rparen();
        let args_span: Option<Span> =
            Span::span_iter_of_spanned(args.iter().map(|(idx, _)| &self.exprs[*idx]));
        let args_span = match (lparen, rparen) {
            (Some(lparen), Some(rparen)) => {
                TextRange::new(lparen.text_range().start(), rparen.text_range().end()).to_span()
            }
            (Some(lparen), _) => {
                if let Some(args_span) = args_span {
                    TextRange::new(lparen.text_range().start(), args_span.range.end()).to_span()
                } else {
                    lparen.text_range().to_span()
                }
            }
            (_, Some(rparen)) => {
                if let Some(args_span) = args_span {
                    TextRange::new(args_span.range.start(), rparen.text_range().end()).to_span()
                } else {
                    rparen.text_range().to_span()
                }
            }
            (_, _) => args_span.unwrap_or(path.span),
        };

        self.tychk_call_expr_args(&path, &args, args_span, &param_ty_ids, param_ty_ids.span);

        let call = Call {
            path,
            args: args.into_iter().map(|(idx, _)| idx).collect(),
        };
        let expr = Expr::Call(call).at(span);
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

        let expected_number = params_len.in_file(self.file_id, params_span);
        let got_number = args_len.in_file(self.file_id, args_span);

        if args_len != params_len {
            self.emit_diagnostic(
                LoweringDiagnostic::IncorrectNumberOfArgsInCall {
                    call_path: path
                        .map_ref(|path| path.to_string(self.interner))
                        .in_file(self.file_id),
                    expected_number,
                    got_number,
                }
                .to_diagnostic(),
            );
        }

        args.iter()
            .zip(params.iter())
            .for_each(|((arg_idx, arg_id), param_id)| {
                let unification_span = self.exprs[*arg_idx].span.in_file(self.file_id);
                let result = self.tchk.unify(*param_id, *arg_id, unification_span);
                self.maybe_emit_diagnostic(result);
            });
    }

    fn lower_int_expr(&mut self, int: ast::IntExpr) -> ExprResult {
        let v = int
            .v()
            .expect("internal compiler error: missing value token in int expression");
        let num_str = self.interner.resolve(&v.text_key());
        let int_span = int.range().to_span();
        let num_int = match num_str.parse::<u64>() {
            Ok(v) => v,
            Err(err) => {
                self.emit_diagnostic(
                    LoweringDiagnostic::CouldNotParseInt {
                        span: int_span.in_file(self.file_id),
                        msg: err.to_string(),
                    }
                    .to_diagnostic(),
                );
                u64::MAX
            }
        };
        let expr = Expr::Int(Int::new(num_int)).at(int.range().to_span());
        let expr_id = self.exprs.alloc(expr);
        let ty_id = self
            .tchk
            .tenv
            .insert(ts::Type::new(TypeKind::Int(None)).in_file(self.file_id, int_span));
        (expr_id, ty_id)
    }

    fn lower_float_expr(&mut self, float: ast::FloatExpr) -> ExprResult {
        let span = float.range().to_span();
        let v = float
            .v()
            .expect("internal compiler error: missing value token in float expression");
        let num_str = self.interner.resolve(&v.text_key());
        let num_float = match num_str.parse::<f64>() {
            Ok(v) => v,
            Err(err) => {
                self.emit_diagnostic(
                    LoweringDiagnostic::CouldNotParseInt {
                        span: span.in_file(self.file_id),
                        msg: err.to_string(),
                    }
                    .to_diagnostic(),
                );
                f64::MAX
            }
        };
        let expr = Expr::Float(Float::new(num_float)).at(span);
        let expr_id = self.exprs.alloc(expr);
        let ty_id = self
            .tchk
            .tenv
            .insert(ts::Type::new(TypeKind::Float(None)).in_file(self.file_id, span));
        (expr_id, ty_id)
    }

    fn lower_path_expr(&mut self, path: ast::PathExpr) -> ExprResult {
        let hir_path = self.lower_path(path.segments());
        let spurs = hir_path.get_unspanned_spurs();
        let path_span = path.range().to_span();
        let ty_id = {
            self.tchk
                .tenv
                .get_path_typeid(
                    spurs
                        .in_file(self.file_id, path_span)
                        .map_inner(|x| x.into_iter()),
                )
                .map_or_else(
                    |diagnostic| {
                        self.emit_diagnostic(diagnostic);
                        self.tchk
                            .tenv
                            .insert_unknown(path_span.in_file(self.file_id))
                    },
                    identity,
                )
        };
        let expr = Expr::Path(hir_path).at(path_span);
        let expr_id = self.exprs.alloc(expr);
        (expr_id, ty_id)
    }

    fn lower_struct_expr(
        &mut self,
        strukt: ast::StructExpr,
        generic_param_list: &GenericParamList,
    ) -> ExprResult {
        let struct_span = strukt.range().to_span();
        let (path, generic_arg_list) = self.lower_node(
            strukt.path(),
            |_, _| (Path::poisoned(), None),
            |this, path| (this.lower_path(path.segments()), path.generic_arg_list()),
        );
        let args = generic_arg_list.map_or(vec![].at(struct_span), |arg_list| {
            arg_list
                .args()
                .map(|arg| self.lower_type(arg, generic_param_list))
                .collect::<Vec<_>>()
                .at(arg_list.range().to_span())
        });

        let struct_field_types_result = self.tchk.tenv.get_struct_field_types(
            path.iter()
                .map(|spur| spur.inner)
                .in_file(self.file_id, struct_span),
        );
        let (struct_concrete_kind, struct_decl_span) = self.maybe_emit_diagnostic_with(
            struct_field_types_result,
            |this| (StructConcreteKind::EMPTY, this.file_span(struct_span)),
            |_, strukt| strukt,
        );

        let type_name = path
            .to_string(self.interner)
            .in_file(struct_decl_span.file_id, struct_decl_span.inner);

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
                        num_params: decl_generic_params_len
                            .in_file(struct_decl_span.file_id, decl_generic_params_span),
                        num_args: generic_args_len.in_file(self.file_id, args.span),
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
        let ty_id = self.tchk.tenv.insert(ty.in_file(self.file_id, struct_span));

        let strukt = Struct::new(path, args.inner, fields);
        let expr = Expr::Struct(strukt).at(struct_span);
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
                            this.exprs.alloc(Expr::Error.at(field_span)),
                            this.tchk.tenv.insert(
                                ts::Type::new(TypeKind::Unknown).in_file(this.file_id, field_span),
                            ),
                        )
                    },
                    |this, expr| this.lower_expr(expr, generic_param_list),
                );

                let expected_field_ty =
                    struct_field_types.find(|(field_name, _)| *field_name == name.inner);
                match expected_field_ty {
                    Some((_, expected_field_ty)) => {
                        let unification_result = self.tchk.unify(
                            *expected_field_ty,
                            val_id,
                            name.span.in_file(self.file_id),
                        );
                        self.maybe_emit_diagnostic(unification_result);
                    }
                    None => self.emit_diagnostic(
                        LoweringDiagnostic::UnknownFieldInStructExpr {
                            unknown_field: name
                                .map_ref(|spur| self.interner.resolve(spur).to_string())
                                .in_file(self.file_id),
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
