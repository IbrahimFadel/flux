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
        Apply, Block, Call, Expr, ExprIdx, Function, GenericParams, Let, Name, Path, Struct,
        StructExpr, StructExprField, StructFields, Trait, Type, TypeIdx, Visibility,
        WherePredicate,
    },
    item_tree::ItemTree,
    name_res::{path_res::ResolvePathError, DefMap, LocalModuleId},
    per_ns::PerNs,
    FunctionId, ModuleDefId, ModuleId, StructId, TraitId,
};

mod apply;
mod resolve;

type ExprResult = (ExprIdx, TypeId);

#[derive(Debug)]
pub struct LoweredBodies {
    pub exprs: Arena<Spanned<Expr>>,
    pub types: Arena<Spanned<Type>>,
    pub indices: HashMap<(ModuleId, ModuleDefId), ExprIdx>,
}

pub(crate) struct LowerCtx<'a> {
    def_map: Option<&'a DefMap>,
    cur_module_id: LocalModuleId,
    tchk: TChecker,
    string_interner: &'static ThreadedRodeo,
    exprs: Arena<Spanned<Expr>>,
    pub types: &'a mut Arena<Spanned<Type>>,
    diagnostics: Vec<Diagnostic>,
    indices: HashMap<(ModuleId, ModuleDefId), ExprIdx>,
}

impl<'a> LowerCtx<'a> {
    pub fn new(
        string_interner: &'static ThreadedRodeo,
        types: &'a mut Arena<Spanned<Type>>,
    ) -> Self {
        Self {
            def_map: None,
            cur_module_id: LocalModuleId::from_raw(RawIdx::from(0)),
            tchk: TChecker::new(TEnv::new(string_interner)),
            string_interner,
            exprs: Arena::new(),
            types,
            diagnostics: Vec::new(),
            indices: HashMap::new(),
        }
    }

    /// Lower an AST node to its HIR equivalent
    ///
    /// This exists to help clean up the lowering process due to the optional nature of the AST layer.
    /// We want certain nodes to **ALWAYS** be emitted even when there's a parsing error, but be marked as poisoned.
    /// For this reason, we can `unwrap`/`expect` safely (panics are ICEs), then carry on.
    ///
    /// If the node is poisoned, use the supplied closure to provide a poisoned value.
    /// If the node is not poisoned, use the supplied closure to carry out the regular lowering process.
    ///
    /// This method can be quite verbose and clog up code, so generally this should be used in generalizable methods such as `lower_name` or `lower_generic_param_list`, not in unique methods such as `lower_fn_decl`.
    pub fn lower_node<N, T, P, F>(
        &mut self,
        node: Option<N>,
        poison_function: P,
        normal_function: F,
    ) -> T
    where
        N: AstNode,
        P: FnOnce(&mut Self, N) -> T,
        F: FnOnce(&mut Self, N) -> T,
    {
        let n = node.expect("internal compiler error: missing node that should always be emitted");
        if n.is_poisoned() {
            poison_function(self, n)
        } else {
            normal_function(self, n)
        }
    }
    // pub fn lower_node<N, T, P, F>(node: Option<N>, poison_function: P, normal_function: F) -> T
    // where
    //     N: AstNode,
    //     P: FnOnce(&mut S, N) -> T,
    //     F: FnOnce(&mut S, N) -> T,
    // {
    //     let n = node.unwrap_or_else(|| ice("missing node that should always be emitted"));
    //     if n.is_poisoned() {
    //         poison_function(this, n)
    //     } else {
    //         normal_function(this, n)
    //     }
    // }

    pub fn finish(self) -> (LoweredBodies, Vec<Diagnostic>) {
        (
            LoweredBodies {
                exprs: self.exprs,
                types: std::mem::take(self.types),
                indices: self.indices,
            },
            self.diagnostics,
        )
    }

    pub fn with_def_map(
        def_map: &'a DefMap,
        string_interner: &'static ThreadedRodeo,
        types: &'a mut Arena<Spanned<Type>>,
    ) -> Self {
        Self {
            def_map: Some(def_map),
            cur_module_id: LocalModuleId::from_raw(RawIdx::from(0)),
            tchk: TChecker::new(TEnv::new(string_interner)),
            string_interner,
            exprs: Arena::new(),
            types,
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
                    self.handle_function(f.index, item_tree);
                }
                crate::item_tree::ModItem::Mod(_) => {}
                crate::item_tree::ModItem::Struct(s) => self.handle_struct(s.index, item_tree),
                crate::item_tree::ModItem::Trait(trt) => self.handle_trait(trt.index, item_tree),
                crate::item_tree::ModItem::Use(_) => {}
            }
        }
    }

    fn handle_function(&mut self, f_idx: FunctionId, item_tree: &ItemTree) -> ExprIdx {
        let f = &item_tree[f_idx];

        self.check_where_predicates(
            &f.generic_params,
            f.generic_params.span.in_file(self.file_id()),
            self.cur_module_id,
        );

        let mut used_generics = vec![];

        for param in f.params.inner.iter() {
            if let Type::Generic(name) = &self.types[param.ty.raw()].inner {
                used_generics.push(*name);
            }
        }

        let (body_idx, body_tid) = self.lower_expr(
            f.ast
                .as_ref()
                .unwrap_or_else(|| {
                    ice("function ast should only be `None` for trait method declarations")
                })
                .body(),
            &f.generic_params,
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

        // TODO: once generics get developed more, we'll have to check if they get used inside the function body
        let unusued_generic_params = f.generic_params.unused(used_generics.iter());
        if !unusued_generic_params.is_empty() {
            self.diagnostics.push(
                LowerError::UnusedGenericParams {
                    unused_generic_params: unusued_generic_params
                        .iter()
                        .map(|spur| self.string_interner.resolve(spur).to_string())
                        .collect(),
                    unused_generic_params_file_span: f.generic_params.span.in_file(self.file_id()),
                }
                .to_diagnostic(),
            );
        }

        self.indices.insert(
            (self.cur_module_id, ModuleDefId::FunctionId(f_idx)),
            body_idx.clone(),
        );

        body_idx
    }

    fn handle_struct(&mut self, s: StructId, item_tree: &ItemTree) {
        let s = &item_tree[s];

        let mut used_generics = vec![];

        for field in &s.fields.fields {
            if let Type::Generic(name) = &self.types[field.ty.raw()].inner {
                used_generics.push(*name);
            }
        }

        let unused_generic_params = s.generic_params.unused(used_generics.iter());

        if !unused_generic_params.is_empty() {
            self.diagnostics.push(
                LowerError::UnusedGenericParams {
                    unused_generic_params: unused_generic_params
                        .iter()
                        .map(|spur| self.string_interner.resolve(&spur).to_string())
                        .collect(),
                    unused_generic_params_file_span: s.generic_params.span.in_file(self.file_id()),
                }
                .to_diagnostic(),
            );
        }
    }

    fn handle_trait(&mut self, trt: TraitId, item_tree: &ItemTree) {
        let trt = &item_tree[trt];

        let trait_generic_params = &trt.generic_params;
        for method in &trt.methods.inner {
            let f = &item_tree[*method];
            let method_generic_params = &f.generic_params;
            self.combine_generic_parameters(trait_generic_params, method_generic_params);
        }
    }

    fn combine_generic_parameters(
        &mut self,
        a: &Spanned<GenericParams>,
        b: &Spanned<GenericParams>,
    ) -> GenericParams {
        GenericParams::combine(&a, &b).unwrap_or_else(|(combined_generic_params, duplicates)| {
            self.diagnostics.push(
                LowerError::DuplicateGenerics {
                    generics_that_caused_duplication: duplicates
                        .iter()
                        .map(|spur| self.string_interner.resolve(spur).to_string())
                        .collect(),
                    generics_that_caused_duplication_file_span: b.span.in_file(self.file_id()),
                    generics_that_were_chilling: (),
                    generics_that_were_chilling_file_span: a.span.in_file(self.file_id()),
                }
                .to_diagnostic(),
            );
            combined_generic_params
        })
    }

    fn file_id(&self) -> FileId {
        self.def_map.unwrap()[self.cur_module_id].file_id
    }

    fn insert_type_to_tenv(&mut self, idx: &TypeIdx, file_id: FileId) -> TypeId {
        let kind = self.type_to_tkind(idx, file_id);
        let span = self.types[idx.raw()].span;
        let ty = ts::Type::new(kind);
        self.tchk.tenv.insert(ty.file_span(file_id, span))
    }

    fn type_to_tkind(&mut self, idx: &TypeIdx, file_id: FileId) -> TypeKind {
        let ty = &self.types[idx.raw()];
        match ty.inner.clone() {
            Type::Array(ty, n) => {
                let ty = self.insert_type_to_tenv(&ty, file_id);
                TypeKind::Concrete(ConcreteKind::Array(ty, n))
            }
            Type::Path(path) => {
                TypeKind::Concrete(ConcreteKind::Path(path.to_spur(self.string_interner)))
            }
            Type::Ptr(ty) => {
                TypeKind::Concrete(ConcreteKind::Ptr(self.insert_type_to_tenv(&ty, file_id)))
            }
            Type::Tuple(types) => TypeKind::Concrete(ConcreteKind::Tuple(
                types
                    .iter()
                    .map(|idx| self.insert_type_to_tenv(idx, file_id))
                    .collect(),
            )),
            Type::Unknown => TypeKind::Unknown,
            Type::Generic(name) => TypeKind::Generic(name),
        }
    }

    fn lower_expr(
        &mut self,
        expr: Option<ast::Expr>,
        generic_params: &GenericParams,
    ) -> ExprResult {
        let (idx, tid) = self.lower_node(
            expr,
            |_, _| todo!(),
            |this, expr| match expr {
                ast::Expr::PathExpr(path) => {
                    let (idx, tid, _) = this.lower_path_expr(Some(path));
                    (idx, tid)
                }
                ast::Expr::ParenExpr(_) => todo!(),
                ast::Expr::FloatExpr(float) => this.lower_float_expr(&float),
                ast::Expr::IntExpr(int) => this.lower_int_expr(&int),
                ast::Expr::BinExpr(_) => todo!(),
                ast::Expr::CallExpr(call) => this.lower_call_expr(&call, generic_params),
                ast::Expr::StructExpr(strukt) => this.lower_struct_expr(&strukt, generic_params),
                ast::Expr::BlockExpr(block) => this.lower_block_expr(&block, generic_params),
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
        let path = self.lower_node(
            path,
            |_, path| Path::poisoned().at(path.range().to_span()),
            |_, path| {
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

    fn lower_call_expr(
        &mut self,
        call: &ast::CallExpr,
        generic_params: &GenericParams,
    ) -> ExprResult {
        let (path, ret_tid, resolved_path) = self.lower_path_expr(call.path());

        let args = self.lower_node(
            call.args(),
            |_, arg_list| vec![].at(arg_list.range().to_span()),
            |this, arg_list| {
                arg_list
                    .args()
                    .map(|arg| this.lower_expr(Some(arg), generic_params))
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

    fn lower_struct_expr(
        &mut self,
        strukt: &ast::StructExpr,
        generic_params: &GenericParams,
    ) -> ExprResult {
        let file_id = self.file_id();
        let span = strukt.range().to_span();
        let path = self.lower_path(strukt.path(), generic_params);

        let struct_decl = self
            .get_struct(&path)
            .map(|strukt| strukt.map(|strukt| strukt.clone()));

        let fields = self.lower_struct_fields(strukt.field_list(), generic_params, &struct_decl);

        let ty = ts::Type::with_params(
            TypeKind::Concrete(ConcreteKind::Path(path.to_spur(self.string_interner))),
            path.generic_args
                .iter()
                .map(|idx| self.type_to_tkind(idx, file_id)),
        );
        let tid = self.tchk.tenv.insert(ty.file_span(file_id, span));

        let strukt = Expr::Struct(StructExpr { path, fields }).at(span);
        let idx = self.exprs.alloc(strukt);

        (idx.into(), tid)
    }

    fn lower_struct_fields(
        &mut self,
        field_list: Option<ast::StructExprFieldList>,
        generic_params: &GenericParams,
        struct_decl: &Option<InFile<Struct>>,
    ) -> Vec<StructExprField> {
        let mut initialized_fields = vec![];
        let mut unknown_fields = vec![];
        self.lower_node(
            field_list,
            |_, _| vec![],
            |this, field_list| {
                let res = field_list
                    .fields()
                    .map(|field| {
                        let name = this.lower_name(field.name());
                        let (val, val_tid) = this.lower_expr(field.val(), generic_params);

                        if let Some(struct_decl) = struct_decl {
                            let field = struct_decl
                                .fields
                                .fields
                                .iter()
                                .find(|field| field.name.inner == name.inner);

                            if let Some(field) = field {
                                let field_tid =
                                    this.insert_type_to_tenv(&field.ty, struct_decl.file_id);
                                this.tchk
                                    .unify(field_tid, val_tid, name.span.in_file(this.file_id()))
                                    .unwrap_or_else(|err| {
                                        this.diagnostics.push(err);
                                    });

                                initialized_fields.push(name.inner);
                            } else {
                                unknown_fields
                                    .push(self.string_interner.resolve(&name).to_string());
                            }
                        }

                        StructExprField { name, val }
                    })
                    .collect();

                if let Some(struct_decl) = struct_decl {
                    let mut uninitialized_fields = vec![];
                    struct_decl.fields.fields.iter().for_each(|field| {
                        if initialized_fields
                            .iter()
                            .find(|initialized_field| **initialized_field == field.name.inner)
                            .is_none()
                        {
                            uninitialized_fields
                                .push(this.string_interner.resolve(&field.name).to_string());
                        }
                    });

                    if !uninitialized_fields.is_empty() {
                        this.diagnostics.push(
                            LowerError::UninitializedFieldsInStructExpr {
                                struct_name: this
                                    .string_interner
                                    .resolve(&struct_decl.name)
                                    .to_string(),
                                uninitialized_fields,
                                uninitialized_fields_file_span: field_list
                                    .range()
                                    .to_span()
                                    .in_file(this.file_id()),
                            }
                            .to_diagnostic(),
                        );
                    }
                    if !unknown_fields.is_empty() {
                        this.diagnostics.push(
                            LowerError::UnknownFieldsInStructExpr {
                                struct_name: this
                                    .string_interner
                                    .resolve(&struct_decl.name)
                                    .to_string(),
                                unknown_fields,
                                unknown_fields_file_span: field_list
                                    .range()
                                    .to_span()
                                    .in_file(this.file_id()),
                            }
                            .to_diagnostic(),
                        );
                    }
                }

                res
            },
        )
    }

    fn lower_block_expr(
        &mut self,
        block: &ast::BlockExpr,
        generic_params: &GenericParams,
    ) -> ExprResult {
        let file_id = self.file_id();
        let span = block.range().to_span();
        let mut block_tid = self.tchk.tenv.insert_unit(span.in_file(file_id));
        let mut exprs = vec![];
        let stmts = block.stmts().collect::<Vec<_>>();
        let stmts_len = stmts.len();
        for i in 0..stmts_len {
            let (expr, _) = match &stmts[i] {
                ast::Stmt::ExprStmt(expr) => self.lower_expr(expr.expr(), generic_params),
                ast::Stmt::LetStmt(let_expr) => self.lower_let_expr(let_expr, generic_params),
                ast::Stmt::TerminatorExprStmt(expr) => {
                    let (e, tid) = self.lower_expr(expr.expr(), generic_params);
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

    fn lower_let_expr(
        &mut self,
        let_expr: &ast::LetStmt,
        generic_params: &GenericParams,
    ) -> ExprResult {
        let name = self.lower_name(let_expr.name());
        let ty = match let_expr.ty() {
            Some(ty) => self.lower_type(Some(ty), generic_params),
            None => self.types.alloc(Type::Unknown.at(name.span)).into(),
        };
        let declared_tid = self.insert_type_to_tenv(&ty, self.file_id());
        let (val, val_tid) = self.lower_expr(let_expr.value(), generic_params);
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

    pub(crate) fn lower_path(
        &mut self,
        path: Option<ast::Path>,
        generic_params: &GenericParams,
    ) -> Spanned<Path> {
        self.lower_node(
            path,
            |_, path| Path::poisoned().at(path.range().to_span()),
            |this, path| {
                let segments = path.segments().map(|segment| segment.text_key()).collect();
                let generic_args = path
                    .generic_arg_list()
                    .map(|arg_list| {
                        arg_list
                            .args()
                            .map(|arg| this.lower_type(Some(arg), generic_params))
                            .collect()
                    })
                    .unwrap_or(vec![]);
                Path::new(segments, generic_args).at(path.range().to_span())
            },
        )
    }

    pub(crate) fn lower_name(&mut self, name: Option<ast::Name>) -> Name {
        self.lower_node(
            name,
            |this, name| {
                this.string_interner
                    .get_or_intern_static("poisoned_name")
                    .at(name.range().to_span())
            },
            |_, name| {
                let name = name
                    .ident()
                    .unwrap_or_else(|| ice("name parsed without identifier token"));
                name.text_key().at(name.text_range().to_span())
            },
        )
    }

    pub(crate) fn lower_type(
        &mut self,
        ty: Option<ast::Type>,
        generic_params: &GenericParams,
    ) -> TypeIdx {
        let ty = self.lower_node(
            ty,
            |_, ty| Type::Unknown.at(ty.range().to_span()),
            |this, ty| {
                let span = ty.range().to_span();
                match ty {
                    ast::Type::PathType(path) => {
                        let path = this.lower_path(path.path(), generic_params);
                        if path.segments.len() == 1 {
                            if let Some((_, generic)) = generic_params
                                .types
                                .iter()
                                .find(|(_, name)| name.inner == *path.segments.first().unwrap())
                            {
                                Type::Generic(generic.inner)
                            } else {
                                Type::Path(path.inner)
                            }
                        } else {
                            Type::Path(path.inner)
                        }
                    }
                    ast::Type::TupleType(_) => todo!(),
                    ast::Type::ArrayType(_) => todo!(),
                    ast::Type::PtrType(_) => todo!(),
                }
                .at(span)
            },
        );
        self.types.alloc(ty).into()
    }
}

pub fn lower_def_map_bodies(
    def_map: &DefMap,
    string_interner: &'static ThreadedRodeo,
    types: &mut Arena<Spanned<Type>>,
) -> (LoweredBodies, Vec<Diagnostic>) {
    tracing::info!("lowering definition map bodies");
    let mut ctx = LowerCtx::with_def_map(def_map, string_interner, types);
    for (module_id, item_tree) in def_map.item_trees.iter() {
        ctx.handle_item_tree(item_tree, module_id);
    }
    ctx.finish()
}
