use flux_span::Span;
use flux_typesystem::{self as ts};

use crate::{
    diagnostics::LowerError,
    hir::{BinOp, Cast, If, MemberAccess, StructExpr, StructExprField},
    intrinsics, prelude,
};

use super::*;

mod verify;

impl<'a, 'pkgs> LowerCtx<'a, 'pkgs> {
    pub(super) fn alloc_expr(&mut self, expr: Expr) -> ExprIdx {
        ExprIdx::new(self.package_bodies.exprs.alloc(expr))
    }

    pub(super) fn lower_expr(
        &mut self,
        expr: Option<ast::Expr>,
        generic_params: &GenericParams,
    ) -> Typed<ExprIdx> {
        self.lower_node(
            expr,
            |this, expr| {
                let expr_idx = this.alloc_expr(Expr::Poisoned);
                expr_idx.with_type(
                    this.tckh
                        .tenv
                        .insert(Type::Unknown.file_span(this.file_id, expr.range().to_span())),
                )
            },
            |this, expr| match expr {
                ast::Expr::PathExpr(path_expr) => this.lower_path_expr(path_expr, generic_params),
                ast::Expr::ParenExpr(_) => todo!(),
                ast::Expr::FloatExpr(_) => todo!(),
                ast::Expr::IntExpr(int_expr) => this.lower_int_expr(int_expr),
                ast::Expr::BinExpr(bin_expr) => this.lower_bin_expr(bin_expr, generic_params),
                ast::Expr::CallExpr(call_expr) => this.lower_call_expr(call_expr, generic_params),
                ast::Expr::StructExpr(struct_expr) => {
                    this.lower_struct_expr(struct_expr, generic_params)
                }
                ast::Expr::BlockExpr(block_expr) => {
                    this.lower_block_expr(block_expr, generic_params)
                }
                ast::Expr::TupleExpr(tuple_expr) => {
                    this.lower_tuple_expr(tuple_expr, generic_params)
                }
                ast::Expr::AddressExpr(address_expr) => {
                    this.lower_address_expr(address_expr, generic_params)
                }
                ast::Expr::IdxExpr(_) => todo!(),
                ast::Expr::MemberAccessExpr(member_access_expr) => {
                    this.lower_member_access_expr(member_access_expr, generic_params)
                }
                ast::Expr::IfExpr(if_expr) => this.lower_if_expr(if_expr, generic_params),
                ast::Expr::IntrinsicExpr(intrinsic_expr) => {
                    this.lower_intrinsic_expr(intrinsic_expr, generic_params)
                }
                ast::Expr::StringExpr(_) => todo!(),
                ast::Expr::CastExpr(cast_expr) => this.lower_cast_expr(cast_expr, generic_params),
            },
        )
    }

    fn lower_path_expr(
        &mut self,
        path_expr: ast::PathExpr,
        generic_params: &GenericParams,
    ) -> Typed<ExprIdx> {
        let path = self.lower_path(path_expr.path(), generic_params);

        let tid = if path.len() == 1 {
            self.tckh
                .tenv
                .try_get_local(&(*path.get(0)).file_span(self.file_id, path.span))
        } else {
            None
        }
        .unwrap_or_else(|| {
            self.diagnostics.push(
                LowerError::UnknownLocal {
                    local: path.to_string(self.interner),
                    local_file_span: path.span.in_file(self.file_id),
                }
                .to_diagnostic(),
            );
            self.tckh
                .tenv
                .insert(Type::Unknown.file_span(self.file_id, path_expr.range().to_span()))
        });
        self.alloc_expr(Expr::Path(path.inner)).with_type(tid)
    }

    pub(crate) fn lower_int_expr(&mut self, int_expr: ast::IntExpr) -> Typed<ExprIdx> {
        let span = int_expr.range().to_span();
        let poisoned = |this: &mut Self| {
            this.alloc_expr(Expr::Poisoned)
                .with_type(this.insert_int_type(span))
        };

        let val_str = match int_expr.v() {
            Some(v) => self
                .interner
                .resolve(&v.text_key().unwrap_or_else(|| ice("genuinely not sure"))),
            None => return poisoned(self),
        }
        .replace("_", "");
        let val: u64 = match val_str.parse() {
            Ok(v) => v,
            Err(parse_err) => match parse_err.kind() {
                std::num::IntErrorKind::PosOverflow => {
                    self.diagnostics.push(
                        LowerError::PositiveIntegerOverflow {
                            val: val_str.to_string(),
                            val_file_span: span.in_file(self.file_id),
                        }
                        .to_diagnostic(),
                    );
                    return poisoned(self);
                }
                std::num::IntErrorKind::Empty
                | std::num::IntErrorKind::InvalidDigit
                | std::num::IntErrorKind::NegOverflow => {
                    ice("invalid int value reached hir lowering")
                }
                std::num::IntErrorKind::Zero => unreachable!(),
                _ => unimplemented!(),
            },
        };

        self.alloc_expr(Expr::Int(val))
            .with_type(self.insert_int_type(span))
    }

    fn lower_bin_expr(
        &mut self,
        bin_expr: ast::BinExpr,
        generic_params: &GenericParams,
    ) -> Typed<ExprIdx> {
        let op = self.lower_op(bin_expr.op());
        let lhs = self.lower_expr(bin_expr.lhs(), generic_params);
        let rhs = self.lower_expr(bin_expr.rhs(), generic_params);
        let unification_span = Span::combine(
            self.tckh.tenv.get_span(&lhs.tid),
            self.tckh.tenv.get_span(&rhs.tid),
        );
        self.tckh
            .unify(lhs.tid, rhs.tid, unification_span.in_file(self.file_id))
            .unwrap_or_else(|err| self.diagnostics.push(err));

        let tid = if let Some(binop_trait) =
            prelude::bin_op_trait_map(self.prelude(), self.interner).get(&op)
        {
            tracing::warn!("assuming builtin binop traits have just one method, this probably won't work in the future");
            let package_id = self.package_id.unwrap();
            let trid = self
                .trait_map
                .get(&(package_id, binop_trait.idx.clone().into()))
                .unwrap_or_else(|| ice("prelude traits should always be in trait map"));
            let trait_idx: Idx<TraitDecl> = binop_trait.idx.clone().into();
            let trait_decl = &self.item_tree(package_id).traits[trait_idx];

            // I think we only need to check one side (either lhs or rhs) since we unified them already
            let signature = if let Some(aid) = self.tckh.valid_applications(lhs.tid, trid) {
                let application = self.tckh.get_trait_application(trid, &aid);
                Some(application.signatures[0].clone())
            } else {
                let lhs_filespan = self.tckh.tenv.get_filespan(&lhs.tid);
                self.diagnostics.push(
                    LowerError::TypeDoesNotImplementTrait {
                        ty: self.tckh.tenv.fmt_tid(&lhs.tid),
                        ty_file_span: lhs_filespan,
                        trt: self.interner.resolve(&trait_decl.name).to_string(),
                    }
                    .to_diagnostic(),
                );
                None
            };

            match signature {
                Some(signature) => *signature.return_ty(),
                None => self
                    .tckh
                    .tenv
                    .insert_unknown(self.file_id, unification_span),
            }
        } else {
            ice("this needs a diagnostic");
            // self.tckh
            //     .tenv
            //     .insert_unknown(self.file_id, unification_span)
        };

        self.alloc_expr(Expr::BinOp(BinOp::new(lhs.inner, rhs.inner, op)))
            .with_type(tid)
    }

    fn lower_call_expr(
        &mut self,
        call_expr: ast::CallExpr,
        generic_params: &GenericParams,
    ) -> Typed<ExprIdx> {
        self.lower_arg_list(call_expr.args(), generic_params);
        todo!()
    }

    fn lower_arg_list(
        &mut self,
        arg_list: Option<ast::ArgList>,
        generic_params: &GenericParams,
    ) -> Vec<Typed<ExprIdx>> {
        self.lower_node(
            arg_list,
            |_, _| vec![],
            |this, arg_list| {
                arg_list
                    .args()
                    .map(|arg| this.lower_expr(Some(arg), generic_params))
                    .collect()
            },
        )
    }

    fn lower_struct_expr(
        &mut self,
        struct_expr: ast::StructExpr,
        generic_params: &GenericParams,
    ) -> Typed<ExprIdx> {
        let span = struct_expr.range().to_span();
        let path = self.lower_path(struct_expr.path(), generic_params);
        let fields = self.lower_struct_expr_fields(struct_expr.field_list(), generic_params);
        let struct_expr = StructExpr::new(path.clone(), fields);

        self.resolve_path(&path).map(|(package_id, item_id)| {
            if let ItemTreeIdx::Struct(struct_id) = item_id.idx {
                let struct_decl = &self.item_tree(package_id).structs[struct_id];
                self.verify_struct_expr_fields(struct_decl, &struct_expr);
            } else {
                self.diagnostics.push(
                    LowerError::ExpectedDifferentItem {
                        path: path.to_string(self.interner),
                        path_file_span: path.span.in_file(self.file_id),
                        expected: "struct",
                        got: item_id.to_item_name(),
                    }
                    .to_diagnostic(),
                );
            }
        });

        let tid = self.tckh.tenv.insert(
            TypeKind::Concrete(ConcreteKind::Path(ts::Path::new(
                path.segments.clone(),
                path.generic_args.clone(),
            )))
            .file_span(self.file_id, span),
        );
        self.alloc_expr(Expr::Struct(struct_expr)).with_type(tid)
    }

    fn lower_struct_expr_fields(
        &mut self,
        field_list: Option<ast::StructExprFieldList>,
        generic_params: &GenericParams,
    ) -> Spanned<Vec<StructExprField>> {
        self.lower_node(
            field_list,
            |_, field_list| vec![].at(field_list.range().to_span()),
            |this: &mut LowerCtx<'_, '_>, field_list| {
                field_list
                    .fields()
                    .map(|field| {
                        let name = this.lower_name(field.name());
                        let val = this.lower_expr(field.val(), generic_params);
                        StructExprField::new(name, val)
                    })
                    .collect::<Vec<_>>()
                    .at(field_list.range().to_span())
            },
        )
    }

    fn lower_block_expr(
        &mut self,
        block_expr: ast::BlockExpr,
        generic_params: &GenericParams,
    ) -> Typed<ExprIdx> {
        let mut terminator: Option<Typed<ExprIdx>> = None;
        block_expr.stmts().for_each(|stmt| {
            if let Some(terminator) = &terminator {
                self.diagnostics.push(
                    LowerError::StmtFollowingTerminatorExpr {
                        terminator: (),
                        terminator_file_span: self.tckh.tenv.get_filespan(&terminator.tid),
                        following_expr: (),
                        following_expr_file_span: stmt.range().to_span().in_file(self.file_id),
                    }
                    .to_diagnostic(),
                );
            }
            let (was_terminator, expr) = self.lower_stmt(stmt, generic_params);
            if was_terminator {
                terminator = Some(expr);
            }
        });
        let span = block_expr
            .rbrace()
            .map(|rbrace| rbrace.text_range())
            .unwrap_or_else(|| block_expr.range())
            .to_span();
        terminator.unwrap_or_else(|| {
            self.alloc_expr(Expr::Tuple(vec![]))
                .with_type(self.insert_unit(span))
        })
    }

    fn lower_tuple_expr(
        &mut self,
        tuple_expr: ast::TupleExpr,
        generic_params: &GenericParams,
    ) -> Typed<ExprIdx> {
        let (vals, types): (Vec<_>, Vec<_>) = tuple_expr
            .vals()
            .map(|expr| {
                let expr = self.lower_expr(Some(expr), generic_params);
                (expr.inner, expr.tid)
            })
            .unzip();
        let tid = self
            .tckh
            .tenv
            .insert(Type::Tuple(types).file_span(self.file_id, tuple_expr.range().to_span()));
        self.alloc_expr(Expr::Tuple(vals)).with_type(tid)
    }

    fn lower_address_expr(
        &mut self,
        address_expr: ast::AddressExpr,
        generic_params: &GenericParams,
    ) -> Typed<ExprIdx> {
        let v = self.lower_expr(address_expr.of(), generic_params);
        let tid = self
            .tckh
            .tenv
            .insert(Type::Ptr(v.tid).file_span(self.file_id, address_expr.range().to_span()));
        self.alloc_expr(Expr::Address(v.inner)).with_type(tid)
    }

    fn lower_member_access_expr(
        &mut self,
        member_access_expr: ast::MemberAccessExpr,
        generic_params: &GenericParams,
    ) -> Typed<ExprIdx> {
        let lhs = self.lower_expr(member_access_expr.lhs(), generic_params);
        let lhs_span = self.tckh.tenv.get_span(&lhs.tid);
        let lhs_inner = self.tckh.tenv.get_inner_tid(&lhs.tid);
        let rhs = self.lower_name(member_access_expr.rhs());

        let tid = if let TypeKind::Concrete(ConcreteKind::Path(path)) =
            &self.tckh.tenv.get(&lhs_inner).inner.inner
        {
            let path = Path::new(path.segments.clone(), path.generic_args.clone());
            self.resolve_path(&path.at(lhs_span))
                .map(|(package_id, item_id)| match item_id.idx {
                    ItemTreeIdx::Struct(struct_idx) => {
                        let strukt = &self.item_tree(package_id).structs[struct_idx];
                        let file_id = self.file_id(package_id, item_id.mod_id);
                        strukt
                            .fields
                            .iter()
                            .find_map(|field| {
                                if field.name.inner == rhs.inner {
                                    Some(field.ty.inner)
                                } else {
                                    None
                                }
                            })
                            .unwrap_or_else(|| {
                                self.diagnostics.push(
                                    LowerError::UnknownStructField {
                                        field: self.interner.resolve(&rhs).to_string(),
                                        field_file_span: rhs.span.in_file(self.file_id),
                                        strukt: self.interner.resolve(&strukt.name).to_string(),
                                        strukt_file_span: strukt.name.span.in_file(file_id),
                                    }
                                    .to_diagnostic(),
                                );
                                self.tckh.tenv.insert_unknown(self.file_id, rhs.span)
                            })
                    }
                    _ => {
                        todo!("diagnostic. This can only happen in structs")
                        // self.tckh.tenv.insert_unknown(self.file_id, rhs.span)
                    }
                })
                .unwrap_or_else(|| self.tckh.tenv.insert_unknown(self.file_id, rhs.span))
        } else {
            todo!("diagnostic. This can only happen in structs")
        };

        self.alloc_expr(Expr::MemberAccess(MemberAccess::new(lhs, rhs)))
            .with_type(tid)
    }

    fn lower_if_expr(
        &mut self,
        if_expr: ast::IfExpr,
        generic_params: &GenericParams,
    ) -> Typed<ExprIdx> {
        let span = if_expr.range().to_span();
        let cond = self.lower_expr(if_expr.condition(), generic_params);
        let cond_filespan = self.tckh.tenv.get_filespan(&cond.tid);
        let bool = self
            .tckh
            .tenv
            .insert_bool(self.file_id, cond_filespan.inner);
        tracing::debug!(
            "if unifying {} and {}",
            self.tckh.tenv.fmt_tid(&cond.tid),
            self.tckh.tenv.fmt_tid(&bool)
        );
        self.tckh
            .unify(cond.tid, bool, cond_filespan)
            .unwrap_or_else(|err| self.diagnostics.push(err));
        let then = self.lower_if_block_expr(if_expr.block(), generic_params);
        let r#else = if_expr
            .else_block()
            .map(|else_block| self.lower_if_block_expr(else_block.block(), generic_params));
        let else_ifs = if_expr.else_ifs().map(|else_if| {
            let cond = self.lower_expr(else_if.condition(), generic_params);
            let block = self.lower_if_block_expr(else_if.block(), generic_params);
            (cond, block)
        });
        let tid = then.tid;
        let if_expr = If::new(cond, then, else_ifs, r#else);

        for block in if_expr.blocks().skip(1) {
            self.tckh
                .unify(tid, block.tid, span.in_file(self.file_id))
                .unwrap_or_else(|err| self.diagnostics.push(err));
        }

        self.alloc_expr(Expr::If(if_expr)).with_type(tid)
    }

    fn lower_if_block_expr(
        &mut self,
        block_expr: Option<ast::BlockExpr>,
        generic_params: &GenericParams,
    ) -> Typed<ExprIdx> {
        self.lower_node(
            block_expr,
            |this, block| {
                this.alloc_expr(Expr::Poisoned)
                    .with_type(this.insert_unknown(block.range().to_span()))
            },
            |this, block| this.lower_block_expr(block, generic_params),
        )
    }

    fn lower_intrinsic_expr(
        &mut self,
        intrinsic_expr: ast::IntrinsicExpr,
        generic_params: &GenericParams,
    ) -> Typed<ExprIdx> {
        let name_syntax = intrinsic_expr
            .name()
            .unwrap_or_else(|| ice("intrinsic missing name"));
        let name = name_syntax
            .text_key()
            .unwrap_or_else(|| ice("intrinsic missing name"));
        let span = name_syntax.text_range().to_span();
        let args = self.lower_arg_list(intrinsic_expr.arg_list(), generic_params);
        let arg_tids = args.iter().map(|arg| arg.tid).collect::<Vec<_>>();

        if let Some(signature) =
            intrinsics::get_signature(name, self.file_id, span, self.tckh.tenv, self.interner)
        {
            self.verify_call_args(
                arg_tids.as_slice().file_span(self.file_id, span),
                signature.parameters().file_span(self.file_id, span),
            );
            self.alloc_expr(Expr::Intrinsic)
                .with_type(*signature.return_ty())
        } else {
            self.diagnostics.push(
                LowerError::UnknownIntrinsic {
                    intrinsic: self.interner.resolve(&name).to_string(),
                    intrinsic_file_span: span.in_file(self.file_id),
                }
                .to_diagnostic(),
            );
            self.alloc_expr(Expr::Poisoned)
                .with_type(self.insert_unknown(intrinsic_expr.range().to_span()))
        }
    }

    fn lower_cast_expr(
        &mut self,
        cast_expr: ast::CastExpr,
        generic_params: &GenericParams,
    ) -> Typed<ExprIdx> {
        let val = self.lower_expr(cast_expr.val(), generic_params);
        let to_ty = self.lower_type(cast_expr.to_ty(), generic_params);
        let tid = to_ty.inner;
        let cast = Cast::new(val, tid);
        self.alloc_expr(Expr::Cast(cast)).with_type(tid)
    }
}
