use flux_diagnostics::{ice, Diagnostic, ToDiagnostic};
use flux_id::{
    id::{self, WithMod, WithPackage},
    Map,
};
use flux_parser::{
    ast::{self, AstNode},
    syntax::SyntaxToken,
};
use flux_typesystem::{FnSignature, TEnv, ThisCtx, TraitRestriction, Type, Typed, WithType};
use flux_util::{FileId, FileSpanned, InFile, Interner, Span, Spanned, ToSpan, WithSpan};

use crate::{
    builtin,
    def::{
        expr::{BinOp, Cast, Expr, If, MemberAccess, Op, StructExpr},
        item::StructDecl,
        GenericParams, StructExprField, StructExprFieldList,
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
    exprs: &'a mut Map<id::Expr, Typed<Expr>>,
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
        exprs: &'a mut Map<id::Expr, Typed<Expr>>,
        packages: &'a Map<id::Pkg, Package>,
        tenv: &'a mut TEnv<'res>,
        item_resolver: &'a ItemResolver<'a>,
        interner: &'static Interner,
        diagnostics: &'a mut Vec<Diagnostic>,
    ) -> Self {
        Self {
            type_lowerer: r#type::LoweringCtx::new(ThisCtx::Function, interner),
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
    ) -> id::Expr {
        lower_node_mut(
            self,
            expr,
            |this, expr| {
                this.exprs.insert(
                    Expr::Poisoned
                        .with_type(this.tenv.insert(Type::unknown().at(expr.range().to_span()))),
                )
            },
            |this, expr| match expr {
                ast::Expr::PathExpr(path_expr) => this.lower_path_expr(path_expr, generic_params),
                ast::Expr::ParenExpr(_) => todo!(),
                ast::Expr::FloatExpr(_) => todo!(),
                ast::Expr::IntExpr(int_expr) => this.lower_int_expr(int_expr),
                ast::Expr::BinExpr(bin_expr) => this.lower_bin_expr(bin_expr, generic_params),
                ast::Expr::CallExpr(_) => todo!(),
                ast::Expr::StructExpr(struct_expr) => {
                    this.lower_struct_expr(struct_expr, generic_params)
                }
                ast::Expr::BlockExpr(block_expr) => {
                    this.lower_block_expr(block_expr, generic_params)
                }
                ast::Expr::TupleExpr(_) => todo!(),
                ast::Expr::AddressExpr(_) => todo!(),
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

    fn lower_stmt(&mut self, stmt: ast::Stmt, generic_params: &GenericParams) -> (bool, id::Expr) {
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
    ) -> id::Expr {
        let name = self.type_lowerer.lower_name(let_stmt.name());
        let ty = let_stmt
            .ty()
            .map(|ty| {
                let ty = self.type_lowerer.lower_type(Some(ty), generic_params);
                self.tenv.insert(ty)
            })
            .unwrap_or_else(|| self.tenv.insert(Type::unknown().at(name.span)));
        let val = self.lower(let_stmt.value(), generic_params);

        let val_tid = self.exprs.get(val).tid;
        self.tenv.add_equality(ty, val_tid);

        let ty = if let_stmt.ty().is_some() { ty } else { val_tid };
        let tid = self
            .tenv
            .insert(Type::unit().at(let_stmt.range().to_span()));
        self.tenv.insert_local(name.inner, ty);
        self.exprs.insert(Expr::unit().with_type(tid))
    }

    fn lower_path_expr(
        &mut self,
        path_expr: ast::PathExpr,
        generic_params: &GenericParams,
    ) -> id::Expr {
        let path = self
            .type_lowerer
            .lower_path(path_expr.path(), generic_params);
        let file_id = self.file_id;
        let span = path.span;
        let path = path.map(|path| path.map_args(|arg| self.tenv.insert(arg.at(span))));

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
                self.tenv.insert(Type::unknown().at(span))
            });

        self.exprs.insert(Expr::Path(path.inner).with_type(tid))
    }

    fn lower_int_expr(&mut self, int_expr: ast::IntExpr) -> id::Expr {
        let span = int_expr.range().to_span();
        let tid = self.tenv.insert(Type::int().at(span));
        let poisoned = |this: &mut Self| this.exprs.insert(Expr::Poisoned.with_type(tid));

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

        self.exprs.insert(Expr::Int(val).with_type(tid))
    }

    fn lower_bin_expr(
        &mut self,
        bin_expr: ast::BinExpr,
        generic_params: &GenericParams,
    ) -> id::Expr {
        let op = self.lower_op(bin_expr.op());
        let lhs = self.lower(bin_expr.lhs(), generic_params);
        let lhs_tid = self.exprs.get(lhs).tid;
        let rhs = self.lower(bin_expr.rhs(), generic_params);
        let rhs_tid = self.exprs.get(rhs).tid;
        let span = Span::combine(self.tenv.get_span(lhs_tid), self.tenv.get_span(rhs_tid));

        let tid = self.tenv.insert(Type::unknown().at(span));

        let (trait_path, method_name) = builtin::get_binop_trait(&op, self.interner);
        let trait_id = self
            .item_resolver
            .resolve_trait_ids(trait_path.in_mod(self.mod_id))
            .map_err(|err| {
                self.diagnostics
                    .push(err.to_diagnostic(self.file_id, span, self.interner))
            })
            .map(|(package_id, _, trait_id)| trait_id.in_pkg(package_id))
            .ok();

        if let Some(trait_id) = trait_id {
            self.tenv
                .add_trait_restriction(lhs_tid, TraitRestriction::new(trait_id, vec![rhs_tid]));
            self.tenv
                .add_trait_restriction(rhs_tid, TraitRestriction::new(trait_id, vec![lhs_tid]));

            let item_tree = &self.packages.get(trait_id.pkg_id).item_tree;
            let trait_decl = item_tree.traits.get(trait_id.inner);

            let method = trait_decl
                .get_method_in_item_tree(*method_name, item_tree)
                .unwrap_or_else(|| {
                    ice(format!(
                        "could not find builtin method `{}` for trait `{}`",
                        self.interner.resolve(method_name),
                        self.interner.resolve(&trait_decl.name)
                    ))
                });

            let method_return_ty = self.tenv.insert(method.return_ty.inner.clone().at(span));
            self.tenv.add_equality(tid, method_return_ty);
        }

        self.exprs
            .insert(Expr::BinOp(BinOp::new(lhs, rhs, op)).with_type(tid))
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

    fn lower_struct_expr(
        &mut self,
        struct_expr: ast::StructExpr,
        generic_params: &GenericParams,
    ) -> id::Expr {
        let span = struct_expr.range().to_span();
        let mut path = self
            .type_lowerer
            .lower_path(struct_expr.path(), generic_params);

        let struct_decl = self
            .item_resolver
            .resolve_struct(path.as_ref().inner.in_mod(self.mod_id))
            .map_err(|err| {
                self.diagnostics
                    .push(err.to_diagnostic(self.file_id, span, self.interner))
            })
            .ok();

        let expr = if let Some(struct_decl) = struct_decl {
            /*
               deal w args
            */
            let num_args = path.args.len();
            let num_params = struct_decl.generic_params.types.len();
            if num_args < num_params {
                for _ in num_args..num_params {
                    path.args.push(Type::unknown());
                }
            }

            let fields =
                self.lower_struct_fields(struct_expr.field_list(), generic_params, &struct_decl);
            Expr::Struct(StructExpr::new(path.clone(), fields))
        } else {
            Expr::Poisoned
        };

        let tid = self.tenv.insert(Type::path(path.inner).at(path.span));
        self.exprs.insert(expr.with_type(tid))
    }

    fn lower_struct_fields(
        &mut self,
        field_list: Option<ast::StructExprFieldList>,
        generic_params: &GenericParams,
        struct_decl: &InFile<&StructDecl>,
    ) -> StructExprFieldList {
        lower_node_mut(
            self,
            field_list,
            |_, _| StructExprFieldList::empty(),
            |this, field_list| {
                let mut all_fields = vec![];
                let mut unknown_fields = vec![];
                let fields = field_list
                    .fields()
                    .map(|field| {
                        let name = this.type_lowerer.lower_name(field.name());
                        let val = this.lower(field.val(), generic_params);

                        all_fields.push(name.inner);
                        match struct_decl.fields.find(*name) {
                            Some(field_decl) => {
                                let val_tid = this.exprs.get(val).tid;
                                let decl_tid = this.tenv.insert(field_decl.ty.clone());
                                this.tenv.add_equality(val_tid, decl_tid);
                            }
                            None => unknown_fields.push(name.inner),
                        };

                        StructExprField::new(name, val)
                    })
                    .collect();

                if !unknown_fields.is_empty() {
                    this.diagnostics.push(
                        LowerError::IncorrectStructFieldsInInitialization {
                            got_fields: all_fields
                                .iter()
                                .map(|name| this.interner.resolve(&name).to_string())
                                .collect(),
                            got_fields_file_span: field_list
                                .range()
                                .to_span()
                                .in_file(this.file_id),
                            struct_name: this.interner.resolve(&struct_decl.name).to_string(),
                            struct_name_file_span: struct_decl
                                .name
                                .span
                                .in_file(struct_decl.file_id),
                            expected_fields: struct_decl
                                .fields
                                .iter()
                                .map(|field| this.interner.resolve(&field.name).to_string())
                                .collect(),
                        }
                        .to_diagnostic(),
                    );
                }

                StructExprFieldList::new(fields)
            },
        )
    }

    fn lower_block_expr(
        &mut self,
        block_expr: ast::BlockExpr,
        generic_params: &GenericParams,
    ) -> id::Expr {
        let mut terminator: Option<id::Expr> = None;
        block_expr.stmts().for_each(|stmt| {
            if let Some(terminator) = &terminator {
                let span = self.tenv.get_span(self.exprs.get(*terminator).tid);
                self.diagnostics.push(
                    LowerError::StmtFollowingTerminatorExpr {
                        terminator: (),
                        terminator_file_span: span.in_file(self.file_id),
                        following_expr: (),
                        following_expr_file_span: stmt.range().to_span().in_file(self.file_id),
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
                .insert(Expr::unit().with_type(self.tenv.insert(Type::unit().at(span))))
        })
    }

    fn lower_member_access_expr(
        &mut self,
        member_access_expr: ast::MemberAccessExpr,
        generic_params: &GenericParams,
    ) -> id::Expr {
        let span = member_access_expr.range().to_span();
        let lhs = self.lower(member_access_expr.lhs(), generic_params);
        let lhs_tid = self.exprs.get(lhs).tid;
        let rhs = self.type_lowerer.lower_name(member_access_expr.rhs());
        self.tenv.add_field_requirement(lhs_tid, rhs.inner);

        self.exprs.insert(
            Expr::MemberAccess(MemberAccess::new(lhs, rhs))
                .with_type(self.tenv.insert(Type::unknown().at(span))),
        )
    }

    fn lower_if_expr(&mut self, if_expr: ast::IfExpr, generic_params: &GenericParams) -> id::Expr {
        let cond = self.lower(if_expr.condition(), generic_params);
        let then = self.lower_if_block_expr(if_expr.block(), generic_params);
        let tid = self.exprs.get(then).tid;
        let r#else = if_expr.else_block().map(|else_block| {
            let block = self.lower_if_block_expr(else_block.block(), generic_params);
            let block_tid = self.exprs.get(block).tid;
            self.tenv.add_equality(tid, block_tid);
            block
        });

        let else_ifs = if_expr.else_ifs().map(|else_if| {
            let cond = self.lower(else_if.condition(), generic_params);
            let block = self.lower_if_block_expr(else_if.block(), generic_params);
            let block_tid = self.exprs.get(block).tid;
            self.tenv.add_equality(tid, block_tid);
            (cond, block)
        });

        let if_expr = If::new(cond, then, else_ifs, r#else);
        self.exprs.insert(Expr::If(if_expr).with_type(tid))
    }

    fn lower_if_block_expr(
        &mut self,
        block_expr: Option<ast::BlockExpr>,
        generic_params: &GenericParams,
    ) -> id::Expr {
        lower_node_mut(
            self,
            block_expr,
            |this, block| {
                this.exprs.insert(
                    Expr::Poisoned.with_type(
                        this.tenv
                            .insert(Type::unknown().at(block.range().to_span())),
                    ),
                )
            },
            |this, block| this.lower_block_expr(block, generic_params),
        )
    }

    fn lower_intrinsic_expr(
        &mut self,
        intrinsic_expr: ast::IntrinsicExpr,
        generic_params: &GenericParams,
    ) -> id::Expr {
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

                let tid = self.tenv.insert(signature.return_ty().clone().at(span));
                self.exprs.insert(Expr::Intrinsic.with_type(tid))
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
                    .insert(Expr::Poisoned.with_type(self.tenv.insert(Type::unknown().at(span))))
            }
        }
    }

    fn lower_arg_list(
        &mut self,
        arg_list: Option<ast::ArgList>,
        generic_params: &GenericParams,
        expected_signature: FileSpanned<&FnSignature>,
    ) -> Spanned<Vec<id::Expr>> {
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
                        let expected_tid = this
                            .tenv
                            .insert(expected.clone().at(unification_span.inner));
                        this.tenv
                            .add_equality(this.exprs.get(expr).tid, expected_tid);
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

    fn lower_cast_expr(
        &mut self,
        cast_expr: ast::CastExpr,
        generic_params: &GenericParams,
    ) -> id::Expr {
        let val = self.lower(cast_expr.val(), generic_params);
        let mut to_ty = self
            .type_lowerer
            .lower_type(cast_expr.to_ty(), generic_params);
        to_ty.span = cast_expr.range().to_span();
        let tid = self.tenv.insert(to_ty);
        let cast = Cast::new(val, tid);
        self.exprs.insert(Expr::Cast(cast).with_type(tid))
    }
}
