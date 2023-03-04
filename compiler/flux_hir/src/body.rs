use flux_diagnostics::{ice, Diagnostic, ToDiagnostic};
use flux_span::{FileId, InFile, Spanned, ToSpan, WithSpan};
use flux_syntax::ast::{self, AstNode};
use flux_typesystem::{self as ts, ConcreteKind, TChecker, TEnv, TypeId, TypeKind};
use hashbrown::HashMap;
use la_arena::{Arena, Idx, RawIdx};
use lasso::ThreadedRodeo;

use crate::{
    diagnostics::LowerError,
    hir::{
        Apply, Block, Call, Expr, ExprIdx, Function, GenericParams, Let, Name, Path, Type,
        Visibility,
    },
    item_tree::ItemTree,
    lower_node,
    name_res::{path_res::ResolvePathError, DefMap, LocalModuleId},
    per_ns::PerNs,
    type_interner::TypeIdx,
    FunctionId, ModuleDefId, ModuleId, TypeInterner,
};

type ExprResult = (ExprIdx, TypeId);

pub struct LoweredBodies {
    pub exprs: Arena<Spanned<Expr>>,
    pub indices: HashMap<(ModuleId, ModuleDefId), ExprIdx>,
}

pub(crate) struct LowerCtx<'a> {
    def_map: Option<&'a DefMap>,
    cur_module_id: LocalModuleId,
    tchk: TChecker,
    type_interner: &'static TypeInterner,
    string_interner: &'static ThreadedRodeo,
    exprs: Arena<Spanned<Expr>>,
    diagnostics: Vec<Diagnostic>,
    indices: HashMap<(ModuleId, ModuleDefId), ExprIdx>,
}

impl<'a> LowerCtx<'a> {
    pub fn new(
        string_interner: &'static ThreadedRodeo,
        type_interner: &'static TypeInterner,
    ) -> Self {
        Self {
            def_map: None,
            cur_module_id: LocalModuleId::from_raw(RawIdx::from(0)),
            tchk: TChecker::new(TEnv::new(string_interner)),
            type_interner,
            string_interner,
            exprs: Arena::new(),
            diagnostics: Vec::new(),
            indices: HashMap::new(),
        }
    }

    pub fn finish(self) -> (LoweredBodies, Vec<Diagnostic>) {
        (
            LoweredBodies {
                exprs: self.exprs,
                indices: self.indices,
            },
            self.diagnostics,
        )
    }

    pub fn with_def_map(
        def_map: &'a DefMap,
        string_interner: &'static ThreadedRodeo,
        type_interner: &'static TypeInterner,
    ) -> Self {
        Self {
            def_map: Some(def_map),
            cur_module_id: LocalModuleId::from_raw(RawIdx::from(0)),
            tchk: TChecker::new(TEnv::new(string_interner)),
            type_interner,
            string_interner,
            exprs: Arena::new(),
            diagnostics: Vec::new(),
            indices: HashMap::new(),
        }
    }

    pub fn handle_item_tree(&mut self, item_tree: &ItemTree, module_id: LocalModuleId) {
        self.cur_module_id = module_id;
        for mod_item in &item_tree.top_level {
            match mod_item {
                crate::item_tree::ModItem::Apply(a) => self.handle_apply(a.index, item_tree),
                crate::item_tree::ModItem::Enum(_) => todo!(),
                crate::item_tree::ModItem::Function(f) => {
                    let body_idx = self.handle_function(f.index, item_tree);
                    self.indices
                        .insert((module_id, ModuleDefId::FunctionId(f.index)), body_idx);
                }
                crate::item_tree::ModItem::Mod(_) => {}
                crate::item_tree::ModItem::Struct(_) => todo!(),
                crate::item_tree::ModItem::Trait(_) => {}
                crate::item_tree::ModItem::Use(_) => {}
            }
        }
    }

    fn handle_apply(&mut self, a: Idx<Apply>, item_tree: &ItemTree) {
        let a = &item_tree[a];
        self.handle_apply_trait(a);
        for f in &a.methods.inner {
            self.handle_function(*f, item_tree);
        }
    }

    fn handle_apply_trait(&mut self, a: &Apply) {
        if let Some(trt_path) = &a.trt {
            let trt = self
                .def_map
                .unwrap()
                .resolve_path(trt_path, self.cur_module_id);
            match trt {
                Ok(per_ns) => match per_ns.types {
                    Some((def_id, m, vis)) => {
                        let item_tree = &self.def_map.unwrap().item_trees[m];
                        let file_id = self.def_map.unwrap().modules[m].file_id;
                        let trt = match def_id {
                            ModuleDefId::TraitId(trt_idx) => {
                                let trt = &item_tree[trt_idx];
                                if self.cur_module_id != m && vis == Visibility::Private {
                                    self.diagnostics.push(
                                        LowerError::TriedApplyingPrivateTrait {
                                            trt: self
                                                .string_interner
                                                .resolve(&trt.name.inner)
                                                .to_string(),
                                            declared_as_private: (),
                                            declared_as_private_file_span: trt
                                                .visibility
                                                .span
                                                .in_file(file_id),
                                            application: (),
                                            application_file_span: trt_path
                                                .span
                                                .in_file(self.file_id()),
                                        }
                                        .to_diagnostic(),
                                    );
                                }
                                trt
                            }
                            _ => unreachable!(),
                        };

                        self.check_trait_methods_with_apply_methods(&trt.methods, m, &a.methods);
                    }
                    None => self.diagnostics.push(
                        LowerError::UnresolvedTrait {
                            trt: trt_path.inner.to_string(self.string_interner),
                            trt_file_span: trt_path.span.in_file(self.file_id()),
                        }
                        .to_diagnostic(),
                    ),
                },
                Err(err) => self.diagnostics.push(
                    err.to_lower_error(self.file_id(), self.string_interner)
                        .to_diagnostic(),
                ),
            }
        }
    }

    fn check_trait_methods_with_apply_methods(
        &mut self,
        trait_methods: &Spanned<Vec<FunctionId>>,
        trait_module_id: ModuleId,
        apply_methods: &Spanned<Vec<FunctionId>>,
    ) {
        let def_map = self.def_map.unwrap();

        for apply_method in &apply_methods.inner {
            for trait_method in &trait_methods.inner {
                let apply_method = &def_map.item_trees[self.cur_module_id][*apply_method];
                let trait_file_id = def_map.modules[trait_module_id].file_id;
                let trait_method = &def_map.item_trees[trait_module_id][*trait_method];

                if apply_method.name == trait_method.name {
                    self.check_trait_method_with_apply_method(
                        trait_method,
                        trait_file_id,
                        trait_module_id,
                        apply_method,
                    );
                }
            }
        }

        // let apply_method_names: HashSet<Spur> = apply_methods
        //     .iter()
        //     .map(|method_id| {
        //         def_map.item_trees[self.cur_module_id][*method_id]
        //             .name
        //             .inner
        //     })
        //     .collect();
        // let trait_method_names: HashSet<Spur> = trait_methods
        //     .iter()
        //     .map(|method_id| {
        //         def_map.item_trees[trait_module_id][*method_id]
        //             .name
        //             .inner
        //     })
        //     .collect();
        // let methods_that_dont_belong: Vec<_> =
        //     apply_method_names.difference(&trait_method_names).collect();
        // let unimplemented_methods: Vec<_> =
        //     trait_method_names.difference(&apply_method_names).collect();

        // if unimplemented_methods.len() > 0 {
        //     let trait_file_id = def_map[trait_module_id].file_id;
        //     self.diagnostics.push(
        //         LowerError::UnimplementedTraitMethods {
        //             trait_methods_declared: trait_method_names
        //                 .iter()
        //                 .map(|spur| self.string_interner.resolve(spur).to_string())
        //                 .collect::<Vec<_>>()
        //                 .file_span(trait_file_id, trait_methods.span),
        //             unimplemented_methods: unimplemented_methods
        //                 .iter()
        //                 .map(|spur| self.string_interner.resolve(spur).to_string())
        //                 .collect::<Vec<_>>()
        //                 .file_span(self.file_id(), apply_methods.span),
        //         }
        //         .to_diagnostic(),
        //     );
        // }

        // if methods_that_dont_belong.len() > 0 {
        //     let trait_file_id = def_map[trait_module_id].file_id;
        //     self.diagnostics.push(
        //         LowerError::MethodsDontBelongInApply {
        //             trait_methods_declared: trait_method_names
        //                 .iter()
        //                 .map(|spur| self.string_interner.resolve(spur).to_string())
        //                 .collect::<Vec<_>>()
        //                 .file_span(trait_file_id, trait_methods.span),
        //             methods_that_dont_belond: methods_that_dont_belong
        //                 .iter()
        //                 .map(|spur| self.string_interner.resolve(spur).to_string())
        //                 .collect::<Vec<_>>()
        //                 .file_span(self.file_id(), apply_methods.span),
        //         }
        //         .to_diagnostic(),
        //     );
        // }
    }

    fn check_trait_method_with_apply_method(
        &mut self,
        trait_method: &Function,
        trait_file_id: FileId,
        trait_module_id: ModuleId,
        apply_method: &Function,
    ) {
        self.check_trait_method_generic_params_with_apply_method_generic_params(
            InFile::new(&trait_method.generic_params, trait_file_id),
            trait_module_id,
            &apply_method.generic_params,
        );
    }

    fn check_trait_method_generic_params_with_apply_method_generic_params(
        &mut self,
        trait_generic_params: InFile<&Spanned<GenericParams>>,
        trait_module_id: ModuleId,
        apply_generic_params: &Spanned<GenericParams>,
    ) {
        let trait_params_len = trait_generic_params.types.len();
        let apply_params_len = apply_generic_params.types.len();
        if trait_params_len != apply_params_len {
            self.diagnostics.push(
                LowerError::IncorrectNumGenericParamsInApplyMethod {
                    got_num: apply_params_len,
                    got_num_file_span: apply_generic_params.span.in_file(self.file_id()),
                    expected_num: trait_params_len,
                    expected_num_file_span: trait_generic_params
                        .span
                        .in_file(trait_generic_params.file_id),
                }
                .to_diagnostic(),
            );
        }

        // let def_map = self.def_map.unwrap();
        // for trait_where_predicate in &trait_generic_params.where_predicates {
        //     let apply_where_predicate = apply_generic_params
        //         .where_predicates
        //         .iter()
        //         .find(|predicate| predicate.ty == trait_where_predicate.ty);
        //     match apply_where_predicate {
        //         Some(apply_where_predicate) => {
        //             let a_trait = def_map.resolve_path(a_bound, trait_module_id).map_or_else(
        //                 |_| {
        //                     todo!();
        //                 },
        //                 |res| {
        //                     res.types.map(|(def_id, mod_id, _)| {
        //                         let item_tree = &self.def_map.unwrap().item_trees[mod_id];
        //                         match def_id {
        //                             crate::ModuleDefId::TraitId(t) => &item_tree[t],
        //                             _ => todo!(),
        //                         }
        //                     })
        //                 },
        //             );
        //         }
        //         None => todo!(),
        //     }
        // }

        // trait_generic_params
        //     .where_predicates
        //     .iter()
        //     .zip(apply_generic_params.where_predicates.iter())
        //     .for_each(|(a, b)| {
        //         let a_bound = &a.bound;
        //         let b_bound = &b.bound;

        //         let def_map = self.def_map.unwrap();
        //         let a_trait = def_map.resolve_path(a_bound, trait_module_id).map_or_else(
        //             |_| {
        //                 todo!();
        //             },
        //             |res| {
        //                 res.types.map(|(def_id, mod_id, _)| {
        //                     let item_tree = &self.def_map.unwrap().item_trees[mod_id];
        //                     match def_id {
        //                         crate::ModuleDefId::TraitId(t) => &item_tree[t],
        //                         _ => todo!(),
        //                     }
        //                 })
        //             },
        //         );
        //         let b_trait = def_map.resolve_path(b_bound, trait_module_id).map_or_else(
        //             |_| {
        //                 todo!();
        //             },
        //             |res| {
        //                 res.types.map(|(def_id, mod_id, _)| {
        //                     let item_tree = &self.def_map.unwrap().item_trees[mod_id];
        //                     match def_id {
        //                         crate::ModuleDefId::TraitId(t) => &item_tree[t],
        //                         _ => todo!(),
        //                     }
        //                 })
        //             },
        //         );

        //         if let Some(a_trait) = a_trait {
        //             if let Some(b_trait) = b_trait {
        //                 if a_trait.name.inner != b_trait.name.inner {

        //                 }
        //             }
        //         }
        //     });
    }

    fn handle_function(&mut self, f: Idx<Function>, item_tree: &ItemTree) -> ExprIdx {
        let f = &item_tree[f];
        let (body_idx, body_tid) = self.lower_expr(
            f.ast
                .as_ref()
                .unwrap_or_else(|| {
                    ice("function ast should only be `None` for trait method declarations")
                })
                .body(),
        );
        let ret_tid = self.insert_type_to_tenv(&f.ret_ty, self.file_id());
        self.tchk
            .unify(
                body_tid,
                ret_tid,
                self.exprs[body_idx.raw()].span.in_file(self.file_id()),
            )
            .unwrap_or_else(|err| {
                self.diagnostics.push(err);
            });
        body_idx
    }

    fn file_id(&self) -> FileId {
        self.def_map.unwrap()[self.cur_module_id].file_id
    }

    fn insert_type_to_tenv(&mut self, idx: &Spanned<TypeIdx>, file_id: FileId) -> TypeId {
        let ty = match self.type_interner.resolve(idx.inner) {
            // Type::Array(ty, n) => {
            //     let ty = self.hir_ty_to_ts_ty(ty, where_clause, file_id);
            //     TypeKind::Concrete(ConcreteKind::Array(ty, n.inner))
            // }
            // Type::Generic(generic_name) => TypeKind::Generic(*generic_name),
            Type::Path(path) => {
                TypeKind::Concrete(ConcreteKind::Path(path.to_spur(self.string_interner)))
            }
            Type::Ptr(ty) => {
                TypeKind::Concrete(ConcreteKind::Ptr(self.insert_type_to_tenv(ty, file_id)))
            }
            Type::Tuple(types) => TypeKind::Concrete(ConcreteKind::Tuple(
                types
                    .iter()
                    .map(|idx| self.insert_type_to_tenv(idx, file_id))
                    .collect(),
            )),
            Type::Unknown => TypeKind::Unknown,
        };
        let ty = ts::Type::new(ty, &mut self.tchk.tenv.type_interner);
        self.tchk.tenv.insert(ty.file_span(file_id, idx.span))
    }

    fn lower_expr(&mut self, expr: Option<ast::Expr>) -> ExprResult {
        let (idx, tid) = lower_node(
            expr,
            |_| todo!(),
            |expr| match expr {
                ast::Expr::PathExpr(path) => {
                    let (idx, tid, _) = self.lower_path_expr(Some(path));
                    (idx, tid)
                }
                ast::Expr::ParenExpr(_) => todo!(),
                ast::Expr::FloatExpr(float) => self.lower_float_expr(&float),
                ast::Expr::IntExpr(int) => self.lower_int_expr(&int),
                ast::Expr::BinExpr(_) => todo!(),
                ast::Expr::CallExpr(call) => self.lower_call_expr(&call),
                ast::Expr::StructExpr(_) => todo!(),
                ast::Expr::BlockExpr(block) => self.lower_block_expr(&block),
                ast::Expr::TupleExpr(_) => todo!(),
                ast::Expr::AddressExpr(_) => todo!(),
                ast::Expr::IdxExpr(_) => todo!(),
            },
        );
        (idx, tid)
    }

    fn lower_path_expr(
        &mut self,
        path: Option<ast::PathExpr>,
    ) -> (ExprIdx, TypeId, Result<PerNs, ResolvePathError>) {
        let path = lower_node(
            path,
            |path| Path::poisoned().at(path.range().to_span()),
            |path| {
                let segments = path.segments().map(|segment| segment.text_key()).collect();
                Path::new(segments, vec![]).at(path.range().to_span())
            },
        );

        let span = path.span;
        // Path expressions should never be lowered by item tree, so we should always have a def map
        let resolved_path = self
            .def_map
            .unwrap()
            .resolve_path(&path, self.cur_module_id);
        let path = Expr::Path(path.inner).at(span);
        let idx = self.exprs.alloc(path);
        let tid = match &resolved_path {
            Ok(per_ns) => match per_ns.values {
                Some((def_id, m, _)) => {
                    let item_tree = &self.def_map.unwrap().item_trees[m];
                    let file_id = self.def_map.unwrap().modules[m].file_id;
                    match def_id {
                        crate::ModuleDefId::ApplyId(_) => todo!(),
                        crate::ModuleDefId::EnumId(_) => todo!(),
                        crate::ModuleDefId::FunctionId(f) => {
                            self.insert_type_to_tenv(&item_tree[f].ret_ty, file_id)
                        }
                        crate::ModuleDefId::ModuleId(_) => todo!(),
                        crate::ModuleDefId::StructId(_) => todo!(),
                        crate::ModuleDefId::TraitId(_) => todo!(),
                        crate::ModuleDefId::UseId(_) => todo!(),
                    }
                }
                None => self.tchk.tenv.insert_unknown(span.in_file(self.file_id())),
            },
            Err(_) => self.tchk.tenv.insert_unknown(span.in_file(self.file_id())),
        };
        (idx.into(), tid, resolved_path)
    }

    fn lower_float_expr(&mut self, float: &ast::FloatExpr) -> ExprResult {
        let span = float.range().to_span();
        let float_ty = self.tchk.tenv.insert_float(span.in_file(self.file_id()));
        let value_str = match float.v() {
            Some(v) => self
                .string_interner
                .resolve(&v.text_key())
                .at(v.text_range().to_span()),
            None => return (self.exprs.alloc(Expr::Poisoned.at(span)).into(), float_ty),
        };
        let value: Spanned<f64> = value_str.map(|v| match v.parse() {
            Ok(v) => v,
            Err(_) => todo!(),
        });
        (
            self.exprs.alloc(Expr::Float(value.inner).at(span)).into(),
            float_ty,
        )
    }

    fn lower_int_expr(&mut self, int: &ast::IntExpr) -> ExprResult {
        let span = int.range().to_span();
        let int_ty = self.tchk.tenv.insert_int(span.in_file(self.file_id()));
        let value_str = match int.v() {
            Some(v) => self
                .string_interner
                .resolve(&v.text_key())
                .at(v.text_range().to_span()),
            None => return (self.exprs.alloc(Expr::Poisoned.at(span)).into(), int_ty),
        };
        let value: Spanned<u64> = value_str.map(|v| match v.parse() {
            Ok(v) => v,
            Err(_) => todo!(),
        });
        (
            self.exprs.alloc(Expr::Int(value.inner).at(span)).into(),
            int_ty,
        )
    }

    fn lower_call_expr(&mut self, call: &ast::CallExpr) -> ExprResult {
        let (path, ret_tid, resolved_path) = self.lower_path_expr(call.path());

        let args = lower_node(
            call.args(),
            |arg_list| vec![].at(arg_list.range().to_span()),
            |arg_list| {
                arg_list
                    .args()
                    .map(|arg| self.lower_expr(Some(arg)))
                    .collect::<Vec<_>>()
                    .at(arg_list.range().to_span())
            },
        );

        let function = match resolved_path {
            Ok(per_ns) => per_ns
                .values
                .map(|(def_id, m, vis)| {
                    let item_tree = &self.def_map.unwrap().item_trees[m];
                    let file_id = self.def_map.unwrap().modules[m].file_id;
                    match def_id {
                        crate::ModuleDefId::FunctionId(f) => {
                            let f = &item_tree[f];
                            if vis == Visibility::Private {
                                self.diagnostics.push(
                                    LowerError::TriedCallingPrivateFunction {
                                        function: self
                                            .string_interner
                                            .resolve(&f.name.inner)
                                            .to_string(),
                                        declared_as_private: (),
                                        declared_as_private_file_span: f
                                            .visibility
                                            .span
                                            .in_file(file_id),
                                        call: (),
                                        call_file_span: self.exprs[path.raw()]
                                            .span
                                            .in_file(self.file_id()),
                                    }
                                    .to_diagnostic(),
                                );
                            }
                            Some(f.in_file_ref(file_id))
                        }
                        _ => unreachable!(),
                    }
                })
                .unwrap_or_else(|| {
                    let function_span = self.exprs[path.raw()].span;
                    let path_string = match &self.exprs[path.raw()].inner {
                        Expr::Path(path) => path.to_string(self.string_interner),
                        _ => unreachable!(),
                    };
                    self.diagnostics.push(
                        LowerError::UnresolvedFunction {
                            function: path_string,
                            function_file_span: function_span.in_file(self.file_id()),
                        }
                        .to_diagnostic(),
                    );
                    None
                }),
            Err(err) => {
                self.diagnostics.push(
                    err.to_lower_error(self.file_id(), self.string_interner)
                        .to_diagnostic(),
                );
                None
            }
        };

        if let Some(function) = function {
            self.check_call_args_with_function_decl(&args, function);
        }

        let args = args.map(|args| args.into_iter().map(|(idx, _)| idx).collect());
        let call = Expr::Call(Call {
            path,
            args: args.inner,
        })
        .at(call.range().to_span());
        let idx = self.exprs.alloc(call);

        (idx.into(), ret_tid)
    }

    fn check_call_args_with_function_decl(
        &mut self,
        args: &Spanned<Vec<(ExprIdx, TypeId)>>,
        function: InFile<&Function>,
    ) {
        args.iter()
            .zip(function.params.iter())
            .for_each(|((idx, tid), param)| {
                let param_tid = self.insert_type_to_tenv(&param.ty, function.file_id);
                self.tchk
                    .unify(
                        *tid,
                        param_tid,
                        self.exprs[(*idx).raw()].span.in_file(self.file_id()),
                    )
                    .unwrap_or_else(|err| {
                        self.diagnostics.push(err);
                    });
            });
        let args_len = args.len();
        let params_len = function.params.inner.len();
        if args_len != params_len {
            self.diagnostics.push(
                LowerError::IncorrectNumArgsInCall {
                    expected_number: params_len,
                    expected_number_file_span: function.params.span.in_file(function.file_id),
                    got_number: args_len,
                    got_number_file_span: args.span.in_file(self.file_id()),
                    function: self
                        .string_interner
                        .resolve(&function.name.inner)
                        .to_string(),
                }
                .to_diagnostic(),
            );
        }
    }

    fn lower_block_expr(&mut self, block: &ast::BlockExpr) -> ExprResult {
        let file_id = self.file_id();
        let span = block.range().to_span();
        let mut block_tid = self.tchk.tenv.insert_unit(span.in_file(file_id));
        let mut exprs = vec![];
        let stmts = block.stmts().collect::<Vec<_>>();
        let stmts_len = stmts.len();
        for i in 0..stmts_len {
            let (expr, _) = match &stmts[i] {
                ast::Stmt::ExprStmt(expr) => self.lower_expr(expr.expr()),
                ast::Stmt::LetStmt(let_expr) => self.lower_let_expr(let_expr),
                ast::Stmt::TerminatorExprStmt(expr) => {
                    let (e, tid) = self.lower_expr(expr.expr());
                    block_tid = tid;
                    if i < stmts_len - 1 {
                        self.diagnostics.push(
                            LowerError::StmtFollowingTerminatorExpr {
                                terminator: (),
                                terminator_file_span: expr.range().to_span().in_file(file_id),
                                following_expr: (),
                                following_expr_file_span: stmts[i + 1]
                                    .range()
                                    .to_span()
                                    .in_file(file_id),
                            }
                            .to_diagnostic(),
                        );
                    }
                    exprs.push(e);
                    break;
                }
            };
            exprs.push(expr);
        }
        let block = Expr::Block(Block { exprs }).at(span);
        let idx = self.exprs.alloc(block);
        (idx.into(), block_tid)
    }

    fn lower_let_expr(&mut self, let_expr: &ast::LetStmt) -> ExprResult {
        let name = self.lower_name(let_expr.name());
        let ty = match let_expr.ty() {
            Some(ty) => self.lower_type(Some(ty)),
            None => self.type_interner.intern(Type::Unknown).at(name.span),
        };
        let declared_tid = self.insert_type_to_tenv(&ty, self.file_id());
        let (val, val_tid) = self.lower_expr(let_expr.value());
        self.tchk
            .unify(
                declared_tid,
                val_tid,
                self.exprs[val.raw()].span.in_file(self.file_id()),
            )
            .unwrap_or_else(|err| {
                self.diagnostics.push(err);
            });
        let l = Expr::Let(Let { name, ty, val }).at(let_expr.range().to_span());
        let idx = self.exprs.alloc(l);
        (idx.into(), declared_tid)
    }

    pub(crate) fn lower_path(&self, path: Option<ast::Path>) -> Spanned<Path> {
        lower_node(
            path,
            |path| Path::poisoned().at(path.range().to_span()),
            |path| {
                let segments = path.segments().map(|segment| segment.text_key()).collect();
                let generic_args = path
                    .generic_arg_list()
                    .map(|arg_list| {
                        arg_list
                            .args()
                            .map(|arg| self.lower_type(Some(arg)))
                            .collect()
                    })
                    .unwrap_or(vec![]);
                Path::new(segments, generic_args).at(path.range().to_span())
            },
        )
    }

    pub(crate) fn lower_name(&self, name: Option<ast::Name>) -> Name {
        lower_node(
            name,
            |name| {
                self.string_interner
                    .get_or_intern_static("poisoned_name")
                    .at(name.range().to_span())
            },
            |name| {
                let name = name
                    .ident()
                    .unwrap_or_else(|| ice("name parsed without identifier token"));
                name.text_key().at(name.text_range().to_span())
            },
        )
    }

    pub(crate) fn lower_type(&self, ty: Option<ast::Type>) -> Spanned<TypeIdx> {
        let ty = lower_node(
            ty,
            |ty| Type::Unknown.at(ty.range().to_span()),
            |ty| {
                let span = ty.range().to_span();
                match ty {
                    ast::Type::PathType(path) => Type::Path(self.lower_path(path.path()).inner),
                    ast::Type::TupleType(_) => todo!(),
                    ast::Type::ArrayType(_) => todo!(),
                    ast::Type::PtrType(_) => todo!(),
                }
                .at(span)
            },
        );
        self.type_interner.intern(ty.inner).at(ty.span)
    }
}

pub fn lower_def_map_bodies(
    def_map: &DefMap,
    string_interner: &'static ThreadedRodeo,
    type_interner: &'static TypeInterner,
) -> (LoweredBodies, Vec<Diagnostic>) {
    tracing::info!("lowering definition map bodies");
    let mut ctx = LowerCtx::with_def_map(def_map, string_interner, type_interner);
    for (module_id, item_tree) in def_map.item_trees.iter() {
        ctx.handle_item_tree(item_tree, module_id);
    }
    ctx.finish()
}
