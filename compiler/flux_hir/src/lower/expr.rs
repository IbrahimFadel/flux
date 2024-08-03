use flux_diagnostics::{ice, Diagnostic, ToDiagnostic};
use flux_id::{
    id::{self, InMod},
    Map,
};
use flux_parser::{
    ast::{self, AstNode},
    syntax::SyntaxToken,
};
use flux_typesystem::{FnSignature, TEnv, ThisCtx, Typed, WithType};
use flux_util::{FileId, FileSpanned, Interner, Span, Spanned, ToSpan, WithSpan};

use crate::{
    builtin,
    def::{
        expr::{BinOp, Expr, Op},
        GenericParams,
    },
    diagnostics::LowerError,
    intrinsics,
    name_res::item::ItemResolver,
    Package,
};

use super::{lower_node_mut, r#type};

pub(super) struct LoweringCtx<'a, 'res> {
    type_lowerer: r#type::LoweringCtx,
    file_id: FileId,
    mod_id: id::Mod,
    exprs: &'a mut Map<id::Expr, Expr>,
    packages: &'a Map<id::Pkg, Package>,
    pub(super) tenv: &'a mut TEnv<'res>,
    item_resolver: &'a ItemResolver<'a>,
    interner: &'static Interner,
    diagnostics: &'a mut Vec<Diagnostic>,
}

impl<'a, 'res> LoweringCtx<'a, 'res> {
    pub(super) fn new(
        file_id: FileId,
        mod_id: id::Mod,
        exprs: &'a mut Map<id::Expr, Expr>,
        packages: &'a Map<id::Pkg, Package>,
        tenv: &'a mut TEnv<'res>,
        item_resolver: &'a ItemResolver<'a>,
        this_ctx: ThisCtx,
        interner: &'static Interner,
        diagnostics: &'a mut Vec<Diagnostic>,
    ) -> Self {
        Self {
            type_lowerer: r#type::LoweringCtx::new(this_ctx, interner),
            file_id,
            mod_id,
            exprs,
            tenv,
            packages,
            item_resolver,
            interner,
            diagnostics,
        }
    }

    pub(super) fn lower(
        &mut self,
        expr: Option<ast::Expr>,
        generic_params: &GenericParams,
    ) -> Typed<id::Expr> {
        lower_node_mut(
            self,
            expr,
            |this, expr| {
                this.exprs.insert(Expr::Poisoned).with_type(
                    this.tenv
                        .insert_unknown(this.file_id, expr.range().to_span()),
                )
            },
            |this, expr| match expr {
                ast::Expr::PathExpr(path_expr) => this.lower_path_expr(path_expr, generic_params),
                ast::Expr::ParenExpr(_) => todo!(),
                ast::Expr::FloatExpr(_) => todo!(),
                ast::Expr::IntExpr(int_expr) => this.lower_int_expr(int_expr),
                ast::Expr::BinExpr(bin_expr) => this.lower_bin_expr(bin_expr, generic_params),
                ast::Expr::CallExpr(_) => todo!(),
                ast::Expr::StructExpr(_) => todo!(),
                ast::Expr::BlockExpr(block_expr) => {
                    this.lower_block_expr(block_expr, generic_params)
                }
                ast::Expr::TupleExpr(_) => todo!(),
                ast::Expr::AddressExpr(_) => todo!(),
                ast::Expr::IdxExpr(_) => todo!(),
                ast::Expr::MemberAccessExpr(_) => todo!(),
                ast::Expr::IfExpr(_) => todo!(),
                ast::Expr::IntrinsicExpr(intrinsic_expr) => {
                    this.lower_intrinsic_expr(intrinsic_expr, generic_params)
                }
                ast::Expr::StringExpr(_) => todo!(),
                ast::Expr::CastExpr(_) => todo!(),
            },
        )
    }

    fn lower_stmt(
        &mut self,
        stmt: ast::Stmt,
        generic_params: &GenericParams,
    ) -> (bool, Typed<id::Expr>) {
        match stmt {
            ast::Stmt::LetStmt(let_stmt) => (false, self.lower_let_expr(let_stmt, generic_params)),
            ast::Stmt::ExprStmt(expr_stmt) => (false, self.lower(expr_stmt.expr(), generic_params)),
            ast::Stmt::TerminatorExprStmt(terminator_expr_stmt) => (
                true,
                self.lower(terminator_expr_stmt.expr(), generic_params),
            ),
        }
    }

    fn lower_let_expr(
        &mut self,
        let_stmt: ast::LetStmt,
        generic_params: &GenericParams,
    ) -> Typed<id::Expr> {
        let name = self.type_lowerer.lower_name(let_stmt.name());
        let ty = let_stmt
            .ty()
            .map(|ty| {
                let ty = self.type_lowerer.lower_type(Some(ty), generic_params);
                self.tenv.insert(ty.in_file(self.file_id))
            })
            .unwrap_or_else(|| self.tenv.insert_unknown(self.file_id, name.span));
        let val = self.lower(let_stmt.value(), generic_params);
        let unification_span = self.tenv.get_filespan(val.tid);

        self.tenv
            .unify(ty, val.tid, unification_span)
            .unwrap_or_else(|err| self.diagnostics.push(err));

        let ty = if let_stmt.ty().is_some() { ty } else { val.tid };
        self.tenv.insert_local(name.inner, ty);
        self.exprs.insert(Expr::unit()).with_type(ty)
    }

    fn lower_path_expr(
        &mut self,
        path_expr: ast::PathExpr,
        generic_params: &GenericParams,
    ) -> Typed<id::Expr> {
        let path = self
            .type_lowerer
            .lower_path(path_expr.path(), generic_params);
        let file_id = self.file_id;
        let span = path.span;
        let path =
            path.map(|path| path.map_args(|arg| self.tenv.insert(arg.file_span(file_id, span))));

        let tid = (path.len() == 1)
            .then(|| self.tenv.try_get_local(path.get_nth(0)).cloned())
            .flatten()
            .unwrap_or_else(|| {
                self.diagnostics.push(
                    LowerError::UnknownLocal {
                        local: path.to_string(self.interner),
                        local_file_span: path.span.in_file(file_id),
                    }
                    .to_diagnostic(),
                );
                self.tenv.insert_unknown(file_id, span)
            });

        self.exprs.insert(Expr::Path(path.inner)).with_type(tid)
    }

    fn lower_int_expr(&mut self, int_expr: ast::IntExpr) -> Typed<id::Expr> {
        let span = int_expr.range().to_span();
        let tid = self.tenv.insert_int(self.file_id, span);
        let poisoned = |this: &mut Self| this.exprs.insert(Expr::Poisoned).with_type(tid);

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

        self.exprs.insert(Expr::Int(val)).with_type(tid)
    }

    fn lower_bin_expr(
        &mut self,
        bin_expr: ast::BinExpr,
        generic_params: &GenericParams,
    ) -> Typed<id::Expr> {
        let op = self.lower_op(bin_expr.op());
        let lhs = self.lower(bin_expr.lhs(), generic_params);
        let rhs = self.lower(bin_expr.rhs(), generic_params);
        let span = Span::combine(self.tenv.get_span(lhs.tid), self.tenv.get_span(rhs.tid));

        let (trait_path, method_name) = builtin::get_binop_trait(&op, self.interner);
        let tid = match self
            .item_resolver
            .resolve_trait_ids(trait_path.in_mod(self.mod_id))
        {
            Ok(trait_id) => {
                let item_tree = &self.packages.get(trait_id.pkg_id).item_tree;
                let trait_decl = item_tree.traits.get(**trait_id);
                let method = &trait_decl
                    .methods
                    .iter()
                    .find_map(|method| {
                        let fn_decl = item_tree.functions.get(*method);
                        if fn_decl.name.inner == *method_name {
                            Some(fn_decl)
                        } else {
                            None
                        }
                    })
                    .unwrap_or_else(|| {
                        ice(format!(
                            "could not find method associated with binop `{}`",
                            op.inner
                        ))
                    });
                self.tenv
                    .insert(method.return_ty.clone().in_file(self.file_id))
            }
            Err(err) => {
                self.diagnostics
                    .push(err.to_diagnostic(self.file_id, span, self.interner));
                self.tenv.insert_unknown(self.file_id, span)
            }
        };

        self.exprs
            .insert(Expr::BinOp(BinOp::new(lhs, rhs, op)))
            .with_type(tid)
    }

    fn lower_op(&mut self, op: Option<&SyntaxToken>) -> Spanned<Op> {
        use flux_parser::syntax::SyntaxKind::*;
        let op = op.unwrap_or_else(|| ice("there should always be an op token"));
        match op.kind() {
            Eq => Op::Eq,
            Plus => Op::Add,
            Minus => Op::Sub,
            Star => Op::Mul,
            Slash => Op::Div,
            CmpAnd => Op::CmpAnd,
            CmpEq => Op::CmpEq,
            CmpGt => Op::CmpGt,
            CmpGte => Op::CmpGte,
            CmpLt => Op::CmpLt,
            CmpLte => Op::CmpLte,
            CmpNeq => Op::CmpNeq,
            CmpOr => Op::CmpOr,
            _ => ice("invalid op token encountered"),
        }
        .at(op.text_range().to_span())
    }

    fn lower_block_expr(
        &mut self,
        block_expr: ast::BlockExpr,
        generic_params: &GenericParams,
    ) -> Typed<id::Expr> {
        let mut terminator: Option<Typed<id::Expr>> = None;
        let file_id = self.file_id;
        block_expr.stmts().for_each(|stmt| {
            if let Some(terminator) = &terminator {
                let file_span = self.tenv.get_filespan(terminator.tid);
                self.diagnostics.push(
                    LowerError::StmtFollowingTerminatorExpr {
                        terminator: (),
                        terminator_file_span: file_span,
                        following_expr: (),
                        following_expr_file_span: stmt.range().to_span().in_file(file_id),
                    }
                    .to_diagnostic(),
                );
            } else {
                let (was_terminator, expr) = self.lower_stmt(stmt, generic_params);
                if was_terminator {
                    terminator = Some(expr);
                }
            }
        });
        let span = block_expr
            .rbrace()
            .map(|rbrace| rbrace.text_range())
            .unwrap_or_else(|| block_expr.range())
            .to_span();
        terminator.unwrap_or_else(|| {
            self.exprs
                .insert(Expr::unit())
                .with_type(self.tenv.insert_unit(file_id, span))
        })
    }

    fn lower_intrinsic_expr(
        &mut self,
        intrinsic_expr: ast::IntrinsicExpr,
        generic_params: &GenericParams,
    ) -> Typed<id::Expr> {
        let span = intrinsic_expr.range().to_span();
        let name_syntax = intrinsic_expr
            .name()
            .unwrap_or_else(|| ice("intrinsic missing name"));
        let name_span = name_syntax.text_range().to_span();
        let name = name_syntax
            .text_key()
            .unwrap_or_else(|| ice("intrinsic missing name"));

        match intrinsics::get_signature(&name, self.interner) {
            Some(signature) => {
                let _args = self.lower_arg_list(
                    intrinsic_expr.arg_list(),
                    generic_params,
                    (&signature).file_span(self.file_id, span),
                );

                let tid = self
                    .tenv
                    .insert(signature.return_ty().clone().file_span(self.file_id, span));
                self.exprs.insert(Expr::Intrinsic).with_type(tid)
            }
            None => {
                self.diagnostics.push(
                    LowerError::UnknownIntrinsic {
                        intrinsic: self.interner.resolve(&name).to_string(),
                        intrinsic_file_span: name_span.in_file(self.file_id),
                    }
                    .to_diagnostic(),
                );
                self.exprs
                    .insert(Expr::Poisoned)
                    .with_type(self.tenv.insert_unknown(self.file_id, span))
            }
        }
    }

    fn lower_arg_list(
        &mut self,
        arg_list: Option<ast::ArgList>,
        generic_params: &GenericParams,
        expected_signature: FileSpanned<&FnSignature>,
    ) -> Spanned<Vec<Typed<id::Expr>>> {
        let params = expected_signature.parameters();
        let result = lower_node_mut(
            self,
            arg_list,
            |_, arg_list| vec![].at(arg_list.range().to_span()),
            |this, arg_list| {
                let arg_list = arg_list
                    .args()
                    .zip(params.iter())
                    .map(|(arg, expected)| {
                        let unification_span = arg.range().to_span().in_file(this.file_id);
                        let expr = this.lower(Some(arg), generic_params);
                        let expected_tid = this.tenv.insert(
                            expected
                                .clone()
                                .file_span(this.file_id, unification_span.inner),
                        );

                        this.tenv
                            .unify(expected_tid, expr.tid, unification_span)
                            .unwrap_or_else(|err| this.diagnostics.push(err));

                        expr
                    })
                    .collect::<Vec<_>>()
                    .at(arg_list.range().to_span());

                let args_count = arg_list.len();
                let sig_count = params.len();

                if args_count != sig_count {
                    this.diagnostics.push(
                        LowerError::IncorrectNumberOfArgs {
                            got_num: args_count,
                            got_num_file_span: arg_list.span.in_file(this.file_id),
                            expected_num: sig_count,
                            expected_num_file_span: expected_signature.to_file_span(),
                        }
                        .to_diagnostic(),
                    );
                }

                arg_list
            },
        );

        result
    }
}
