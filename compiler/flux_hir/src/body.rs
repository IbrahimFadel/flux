use flux_diagnostics::{ice, Diagnostic, ToDiagnostic};
use flux_span::{FileId, InFile, Span, Spanned, ToSpan, WithSpan};
use flux_syntax::ast::{self, AstNode};
use flux_typesystem::{self as ts, ConcreteKind, TChecker, TypeId, TypeKind};
use hashbrown::HashMap;
use la_arena::{Arena, Idx, RawIdx};
use lasso::{Spur, ThreadedRodeo};

use crate::{
    diagnostics::LowerError,
    hir::{
        Apply, Block, Call, EnumExpr, Expr, ExprIdx, Function, GenericParams, Item, Let,
        MemberAccess, MemberAccessKind, Name, Path, Struct, StructExpr, StructExprField,
        StructField, Trait, Type, TypeIdx, Typed, WherePredicate, WithType,
    },
    item_tree::ItemTree,
    name_res::{path_res::PathResolutionResultKind, DefMap},
    EnumId, FunctionId, ModuleDefId, ModuleId, StructId, TraitId,
};

mod apply;
mod resolve;

//todo:
// type Typed<ExprIdx> = Typed<ExprIdx>;

#[derive(Debug)]
pub struct LoweredBodies {
    pub exprs: Arena<Spanned<Expr>>,
    pub types: Arena<Spanned<Type>>,
    pub indices: HashMap<(ModuleId, ModuleDefId), ExprIdx>,
}

pub(crate) struct LowerCtx<'a> {
    def_map: Option<&'a DefMap>,
    cur_module_id: ModuleId,
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
            cur_module_id: ModuleId::from_raw(RawIdx::from(0)),
            tchk: TChecker::new(string_interner),
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
            cur_module_id: ModuleId::from_raw(RawIdx::from(0)),
            tchk: TChecker::new(string_interner),
            string_interner,
            exprs: Arena::new(),
            types,
            diagnostics: Vec::new(),
            indices: HashMap::new(),
        }
    }

    pub fn handle_item_tree(&mut self, item_tree: &ItemTree, module_id: ModuleId) {
        self.cur_module_id = module_id;
        let mut function_bodies = vec![];
        for mod_item in &item_tree.top_level {
            match mod_item {
                crate::item_tree::ModItem::Apply(a) => self.handle_apply(a.index, item_tree),
                crate::item_tree::ModItem::Enum(e) => self.handle_enum(e.index, item_tree),
                crate::item_tree::ModItem::Function(f) => {
                    let body = self.handle_function(f.index, item_tree);
                    function_bodies.push(body);
                }
                crate::item_tree::ModItem::Mod(_) => {}
                crate::item_tree::ModItem::Struct(s) => self.handle_struct(s.index, item_tree),
                crate::item_tree::ModItem::Trait(trt) => self.handle_trait(trt.index, item_tree),
                crate::item_tree::ModItem::Use(_) => {}
            }
        }
    }

    fn handle_enum(&mut self, e_idx: EnumId, item_tree: &ItemTree) {
        let e = &item_tree[e_idx];

        self.check_where_predicates(
            &e.generic_params,
            e.generic_params.span.in_file(self.file_id()),
            self.cur_module_id,
        );

        let mut used_generics = vec![];

        for variant in &e.variants {
            if let Some(ty) = &variant.ty {
                self.insert_type_to_tenv(ty, self.file_id());
                if let Type::Generic(name, _) = &self.types[ty.raw()].inner {
                    used_generics.push(*name);
                }
            }
        }

        let unused_generic_params = e.generic_params.unused(&used_generics);
        if !unused_generic_params.is_empty() {
            self.diagnostics.push(
                LowerError::UnusedGenericParams {
                    unused_generic_params: unused_generic_params
                        .iter()
                        .map(|spur| self.string_interner.resolve(spur).to_string())
                        .collect(),
                    unused_generic_params_file_span: e.generic_params.span.in_file(self.file_id()),
                }
                .to_diagnostic(),
            );
        }
    }

    fn handle_function(&mut self, f_idx: FunctionId, item_tree: &ItemTree) -> Typed<ExprIdx> {
        let f = &item_tree[f_idx];

        self.check_where_predicates(
            &f.generic_params,
            f.generic_params.span.in_file(self.file_id()),
            self.cur_module_id,
        );

        let mut used_generics = vec![];

        for param in f.params.inner.iter() {
            let param_tid = self.insert_type_to_tenv(&param.ty, self.file_id());
            self.tchk
                .tenv
                .insert_local_to_scope(param.name.inner, param_tid);
            if let Type::Generic(name, _) = &self.types[param.ty.raw()].inner {
                used_generics.push(*name);
            }
        }

        let body = self.lower_expr(
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
                body.tid,
                ret_tid,
                self.exprs[body.raw()].span.in_file(self.file_id()),
            )
            .unwrap_or_else(|err| {
                self.diagnostics.push(err);
            });

        // TODO: once generics get developed more, we'll have to check if they get used inside the function body
        let unused_generic_params = f.generic_params.unused(&used_generics);
        if !unused_generic_params.is_empty() {
            self.diagnostics.push(
                LowerError::UnusedGenericParams {
                    unused_generic_params: unused_generic_params
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
            body.expr.clone(),
        );

        body
    }

    fn handle_struct(&mut self, s: StructId, item_tree: &ItemTree) {
        let s = &item_tree[s];

        let mut used_generics = vec![];

        for field in &s.fields.fields {
            if let Type::Generic(name, _) = &self.types[field.ty.raw()].inner {
                used_generics.push(*name);
            }
        }

        let unused_generic_params = s.generic_params.unused(&used_generics);

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

        self.tchk.add_trait_to_context(trt.name.inner);

        self.check_where_predicates(
            &trt.generic_params,
            trt.generic_params.span.in_file(self.file_id()),
            self.cur_module_id,
        );

        self.check_trait_assoc_types(&trt.assoc_types);

        let trait_generic_params = &trt.generic_params;
        for method in &trt.methods.inner {
            let f = &item_tree[*method];
            let method_generic_params = &f.generic_params;
            self.combine_generic_parameters(trait_generic_params, method_generic_params);
        }
    }

    fn check_trait_assoc_types(&mut self, assoc_types: &[(Name, Vec<Spanned<Path>>)]) {
        assoc_types.iter().for_each(|(_, restrictions)| {
            restrictions.iter().for_each(|restriction| {
                self.get_trait(restriction);
            });
        });
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
                self.get_type(&path.clone().at(ty.span));
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
            Type::Generic(name, restrictions) => TypeKind::Generic(
                name,
                restrictions
                    .iter()
                    .map(|path| ts::TraitRestriction {
                        name: path
                            .to_spur(self.string_interner)
                            .file_span(file_id, path.span),
                        args: path
                            .generic_args
                            .iter()
                            .map(|arg| self.insert_type_to_tenv(arg, file_id))
                            .collect(),
                    })
                    .collect(),
            ),
        }
    }

    fn lower_expr(
        &mut self,
        expr: Option<ast::Expr>,
        generic_params: &GenericParams,
    ) -> Typed<ExprIdx> {
        self.lower_node(
            expr,
            |_, _| todo!(),
            |this, expr| match expr {
                ast::Expr::PathExpr(path) => {
                    let (idx, _) = this.lower_path_expr(Some(path), None);
                    idx
                }
                ast::Expr::ParenExpr(_) => todo!(),
                ast::Expr::FloatExpr(float) => this.lower_float_expr(&float),
                ast::Expr::IntExpr(int) => this.lower_int_expr(&int),
                ast::Expr::BinExpr(bin) => this.lower_bin_expr(&bin, generic_params),
                ast::Expr::CallExpr(call) => this.lower_call_expr(&call, generic_params),
                ast::Expr::StructExpr(strukt) => this.lower_struct_expr(&strukt, generic_params),
                ast::Expr::BlockExpr(block) => this.lower_block_expr(&block, generic_params),
                ast::Expr::TupleExpr(_) => todo!(),
                ast::Expr::AddressExpr(_) => todo!(),
                ast::Expr::IdxExpr(_) => todo!(),
                ast::Expr::MemberAccessExpr(member_access_expr) => this.lower_member_access_expr(
                    &member_access_expr,
                    generic_params,
                    MemberAccessKind::Field, // If it's a method it will be called by the lower_call_expr and given the correct access kind (that's the theory at least)
                ),
            },
        )
    }

    fn lower_path_expr(
        &mut self,
        path: Option<ast::PathExpr>,
        args: Option<&Spanned<Vec<Typed<ExprIdx>>>>,
    ) -> (Typed<ExprIdx>, Option<InFile<Item>>) {
        let path = self.lower_node(
            path,
            |_, path| Path::poisoned().at(path.range().to_span()),
            |_, path| {
                let segments = path.segments().map(|segment| segment.text_key()).collect();
                Path::new(segments, vec![]).at(path.range().to_span())
            },
        );

        let span = path.span;
        let item = self.try_get_item(&path, PathResolutionResultKind::Any);
        let tid = match &item {
            Ok(item) => {
                if let Some((item, _)) = &item {
                    match &item.inner {
                        Item::Function(f) => self.insert_type_to_tenv(&f.ret_ty, item.file_id),
                        Item::Struct(_) => todo!(),
                        Item::Trait(_) => todo!(),
                        _ => {
                            self.diagnostics.push(
                                {
                                    LowerError::UnresolvedType {
                                        ty: path.to_string(self.string_interner),
                                        ty_file_span: path.span.in_file(self.file_id()),
                                    }
                                }
                                .to_diagnostic(),
                            );
                            self.tchk.tenv.insert_unknown(span.in_file(self.file_id()))
                        }
                    }
                } else {
                    todo!()
                }
            }
            Err(err) => {
                let (expr_idx_tid, is_enum) = self.check_if_path_is_enum(&path, args);
                if is_enum {
                    if let Some((expr_idx, item)) = expr_idx_tid {
                        return (expr_idx, Some(item));
                    } else {
                        self.tchk.tenv.insert_unknown(span.in_file(self.file_id()))
                    }
                } else {
                    if path.segments.len() == 1 {
                        let name = Path::spanned_segment(&path, 0, self.string_interner).unwrap();
                        self.tchk
                            .tenv
                            .get_local_typeid(name.in_file(self.file_id()))
                            .unwrap_or_else(|err| {
                                self.diagnostics.push(err);
                                self.tchk.tenv.insert_unknown(span.in_file(self.file_id()))
                            })
                    } else {
                        // We already gave error for unknown variant
                        if !is_enum {
                            self.diagnostics.push(err.clone());
                        }
                        self.tchk.tenv.insert_unknown(span.in_file(self.file_id()))
                    }
                }
            }
        };
        let item = item.unwrap_or(None).map(|(i, _)| i);
        let path = Expr::Path(path.inner).at(span);
        let idx: ExprIdx = self.exprs.alloc(path).into();

        (idx.with_type(tid), item)
    }

    fn check_if_path_is_enum(
        &mut self,
        path: &Spanned<Path>,
        args: Option<&Spanned<Vec<Typed<ExprIdx>>>>,
    ) -> (Option<(Typed<ExprIdx>, InFile<Item>)>, bool) {
        let enum_path = Path::new(
            path.segments[0..path.segments.len() - 1].to_vec(),
            path.generic_args.clone(),
        )
        .at(path.span);
        let variant_name =
            Path::spanned_segment(&path, path.segments.len() - 1, self.string_interner).unwrap();
        let item = self.try_get_item(&enum_path, PathResolutionResultKind::Any);
        if let Ok(item) = item {
            if let Some((item, _)) = item {
                let e = match &item.inner {
                    Item::Enum(e) => Some(e),
                    _ => None,
                };

                if let Some(e) = e {
                    let variant = e
                        .variants
                        .iter()
                        .find(|variant| variant.name.inner == *variant_name);

                    match variant {
                        Some(variant) => {
                            match (&variant.ty, args) {
                                (Some(variant_ty), Some(args)) => {
                                    let variant_tid =
                                        self.insert_type_to_tenv(variant_ty, self.file_id());
                                    let args_len = args.len();
                                    let variant_name_file_span =
                                        self.tchk.tenv.get_type_filespan(variant_tid);
                                    if args_len == 0 {
                                        self.diagnostics.push(
                                            LowerError::EnumVariantMissingArg {
                                                arg_type: self.tchk.tenv.fmt_ty_id(variant_tid),
                                                variant_name: self
                                                    .string_interner
                                                    .resolve(&variant_name)
                                                    .to_string(),
                                                variant_name_file_span: variant_name_file_span
                                                    .clone(),
                                                initialization: (),
                                                initialization_file_span: args
                                                    .span
                                                    .in_file(self.file_id()),
                                            }
                                            .to_diagnostic(),
                                        );
                                    } else if args_len > 1 {
                                        self.diagnostics.push(LowerError::IncorrectNumArgsInEnumVariantInitialization { variant_name: self
                                            .string_interner
                                            .resolve(&variant_name)
                                            .to_string(), variant_name_file_span: variant_name_file_span.clone(), expected_num: 1, expected_num_file_span: variant_name_file_span, got_num: args_len, got_num_file_span: args
                                            .span
                                            .in_file(self.file_id()) }.to_diagnostic());
                                    }
                                }
                                (Some(variant_ty), None) => {
                                    let variant_tid =
                                        self.insert_type_to_tenv(variant_ty, self.file_id());
                                    let variant_name_file_span =
                                        self.tchk.tenv.get_type_filespan(variant_tid);
                                    self.diagnostics.push(
                                        LowerError::EnumVariantMissingArg {
                                            arg_type: self.tchk.tenv.fmt_ty_id(variant_tid),
                                            variant_name: self
                                                .string_interner
                                                .resolve(&variant_name)
                                                .to_string(),
                                            variant_name_file_span: variant_name_file_span.clone(),
                                            initialization: (),
                                            initialization_file_span: variant_name
                                                .span
                                                .in_file(self.file_id()),
                                        }
                                        .to_diagnostic(),
                                    );
                                }
                                _ => {}
                            }
                            let ty = ts::Type::new(TypeKind::Concrete(ConcreteKind::Path(
                                enum_path.to_spur(self.string_interner),
                            )))
                            .file_span(self.file_id(), enum_path.span);
                            let tid = self.tchk.tenv.insert(ty);
                            let enum_expr = EnumExpr {
                                path: enum_path,
                                variant: variant_name,
                                arg: None,
                            };
                            let expr: ExprIdx =
                                self.exprs.alloc(Expr::Enum(enum_expr).at(path.span)).into();
                            (Some((expr.with_type(tid), item)), true)
                        }
                        None => {
                            self.diagnostics.push(
                                LowerError::UnknownEnumVariant {
                                    eenum: enum_path.to_string(self.string_interner),
                                    variant: self
                                        .string_interner
                                        .resolve(&variant_name)
                                        .to_string(),
                                    variant_file_span: variant_name.span.in_file(self.file_id()),
                                }
                                .to_diagnostic(),
                            );
                            (None, true)
                        }
                    }
                } else {
                    (None, false)
                }
            } else {
                (None, false)
            }
        } else {
            (None, false)
        }
    }

    fn lower_float_expr(&mut self, float: &ast::FloatExpr) -> Typed<ExprIdx> {
        let span = float.range().to_span();
        let float_ty = self.tchk.tenv.insert_float(span.in_file(self.file_id()));
        let value_str = match float.v() {
            Some(v) => self
                .string_interner
                .resolve(&v.text_key())
                .at(v.text_range().to_span()),
            None => {
                let idx: ExprIdx = self.exprs.alloc(Expr::Poisoned.at(span)).into();
                return idx.with_type(float_ty);
            }
        };
        let value: Spanned<f64> = value_str.map(|v| match v.parse() {
            Ok(v) => v,
            Err(_) => todo!(),
        });
        let idx: ExprIdx = self.exprs.alloc(Expr::Float(value.inner).at(span)).into();
        idx.with_type(float_ty)
    }

    fn lower_int_expr(&mut self, int: &ast::IntExpr) -> Typed<ExprIdx> {
        let span = int.range().to_span();
        let int_ty = self.tchk.tenv.insert_int(span.in_file(self.file_id()));
        let value_str = match int.v() {
            Some(v) => self
                .string_interner
                .resolve(&v.text_key())
                .at(v.text_range().to_span()),
            None => {
                let idx: ExprIdx = self.exprs.alloc(Expr::Poisoned.at(span)).into();
                return idx.with_type(int_ty);
            }
        };
        let value: Spanned<u64> = value_str.map(|v| match v.parse() {
            Ok(v) => v,
            Err(_) => todo!(),
        });
        let idx: ExprIdx = self.exprs.alloc(Expr::Int(value.inner).at(span)).into();
        idx.with_type(int_ty)
    }

    fn lower_bin_expr(
        &mut self,
        bin: &ast::BinExpr,
        generic_params: &GenericParams,
    ) -> Typed<ExprIdx> {
        let eq = self.string_interner.get_or_intern_static("=");
        let op = bin
            .op()
            .unwrap_or_else(|| ice("how did we get here without a binop"));

        if op.text_key() == eq {
            self.lower_bin_eq_expr(bin, generic_params)
        } else {
            todo!()
        }
    }

    fn lower_bin_eq_expr(
        &mut self,
        bin: &ast::BinExpr,
        generic_params: &GenericParams,
    ) -> Typed<ExprIdx> {
        let lhs = self.lower_expr(bin.lhs(), generic_params);
        let rhs = self.lower_expr(bin.rhs(), generic_params);
        let rhs_span = self.exprs[rhs.raw()].span;
        self.tchk
            .unify(lhs.tid, rhs.tid, rhs_span.in_file(self.file_id()))
            .unwrap_or_else(|err| {
                self.diagnostics.push(err);
            });
        let span = bin.range().to_span();
        let tid = self.tchk.tenv.insert_unit(span.in_file(self.file_id()));
        let expr = Expr::Tuple(vec![]).at(span);
        let idx: ExprIdx = self.exprs.alloc(expr).into();
        idx.with_type(tid)
    }

    fn lower_call_expr(
        &mut self,
        call: &ast::CallExpr,
        generic_params: &GenericParams,
    ) -> Typed<ExprIdx> {
        if let Some(callee) = call.callee() {
            match callee {
                ast::Expr::PathExpr(path) => self.lower_func_call_expr(call, &path, generic_params),
                ast::Expr::MemberAccessExpr(member_access) => {
                    self.lower_expr_call_expr(call, &member_access, generic_params)
                }
                _ => unreachable!(),
            }
        } else {
            todo!()
        }
    }

    fn lower_func_call_expr(
        &mut self,
        call: &ast::CallExpr,
        path: &ast::PathExpr,
        generic_params: &GenericParams,
    ) -> Typed<ExprIdx> {
        let args = self.lower_call_args(call.args(), generic_params);

        let (path, item) = self.lower_path_expr(Some(path.clone()), Some(&args));

        if let Expr::Enum(enum_expr) = &self.exprs[path.raw()].inner {
            let item = item
                .as_ref()
                .unwrap_or_else(|| ice("got enum expr but no item"));

            let eenum = match &item.inner {
                Item::Enum(eenum) => eenum,
                _ => unreachable!(),
            };
            let variant = eenum
                .variants
                .iter()
                .find(|variant| variant.name.inner == enum_expr.variant.inner);

            let tid = variant
                .map(|variant| {
                    variant
                        .ty
                        .clone()
                        .map(|ty| self.insert_type_to_tenv(&ty, self.file_id()))
                })
                .flatten();

            if let Some(tid) = tid {
                args.first().map(|arg| {
                    self.tchk
                        .unify(
                            tid,
                            arg.tid,
                            self.exprs[arg.raw()].span.in_file(self.file_id()),
                        )
                        .unwrap_or_else(|err| {
                            self.diagnostics.push(err);
                        });
                });
            }
        }
        if let Expr::Enum(enum_expr) = &mut self.exprs[path.raw()].inner {
            if let Some(arg) = args.first() {
                enum_expr.arg = Some(arg.clone());
            }
        }

        if let Some(item) = item {
            let function: Result<Function, ()> = item.inner.try_into();
            if let Ok(function) = &function {
                self.check_call_args_with_function_decl(&args, &function.in_file_ref(item.file_id));
            }
        }

        let tid = path.tid;
        let call = Expr::Call(Call {
            callee: path,
            args: args.inner,
        })
        .at(call.range().to_span());
        let idx: ExprIdx = self.exprs.alloc(call).into();

        idx.with_type(tid)
    }

    fn lower_expr_call_expr(
        &mut self,
        call: &ast::CallExpr,
        member_access_expr: &ast::MemberAccessExpr,
        generic_params: &GenericParams,
    ) -> Typed<ExprIdx> {
        let member_access = self.lower_member_access_expr(
            member_access_expr,
            generic_params,
            MemberAccessKind::Method,
        );
        let args = self.lower_call_args(call.args(), generic_params);
        let tid = member_access.tid;
        let call = Expr::Call(Call {
            callee: member_access,
            args: args.inner,
        })
        .at(call.range().to_span());
        let idx: ExprIdx = self.exprs.alloc(call).into();
        idx.with_type(tid)
    }

    fn lower_call_args(
        &mut self,
        args: Option<ast::ArgList>,
        generic_params: &GenericParams,
    ) -> Spanned<Vec<Typed<ExprIdx>>> {
        self.lower_node(
            args,
            |_, arg_list| vec![].at(arg_list.range().to_span()),
            |this, arg_list| {
                arg_list
                    .args()
                    .map(|arg| this.lower_expr(Some(arg), generic_params))
                    .collect::<Vec<_>>()
                    .at(arg_list.range().to_span())
            },
        )
    }

    fn search_item_tree_for_method(
        &mut self,
        item_tree: &ItemTree,
        tid: TypeId,
        name: Spur,
    ) -> Option<FunctionId> {
        let mut suitable_applies = item_tree.applies.iter().filter(|(_, apply)| {
            let apply_ty = self.insert_type_to_tenv(&apply.ty, self.file_id());
            self.tchk
                .unify(tid, apply_ty, Span::poisoned().in_file(self.file_id()))
                .is_ok()
        });

        let method = suitable_applies.find_map(|(_, apply)| {
            apply
                .methods
                .iter()
                .find(|method| item_tree[**method].name.inner == name)
                .cloned()
        });
        method
    }

    fn search_item_tree_for_struct_with_field(
        &mut self,
        item_tree: &ItemTree,
        struct_name: Spur,
        field_name: Spur,
    ) -> Option<StructField> {
        let mut suitable_structs = item_tree
            .structs
            .iter()
            .filter(|(_, strukt)| strukt.name.inner == struct_name);

        let field = suitable_structs.find_map(|(_, strukt)| {
            strukt
                .fields
                .fields
                .iter()
                .find(|field| field.name.inner == field_name)
                .cloned()
        });
        field
    }

    fn check_call_args_with_function_decl(
        &mut self,
        args: &Spanned<Vec<Typed<ExprIdx>>>,
        function: &InFile<&Function>,
    ) {
        args.iter()
            .zip(function.params.iter())
            .for_each(|(idx, param)| {
                let param_tid = self.insert_type_to_tenv(&param.ty, function.file_id);
                self.tchk
                    .unify(
                        param_tid,
                        idx.tid,
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
    ) -> Typed<ExprIdx> {
        let file_id = self.file_id();
        let span = strukt.range().to_span();
        let path = self.lower_path(strukt.path(), generic_params);

        let struct_decl = self.get_struct(&path).map(|(s, _)| s);
        let fields = self.lower_struct_fields(strukt.field_list(), generic_params, &struct_decl);

        let ty = ts::Type::with_params(
            TypeKind::Concrete(ConcreteKind::Path(path.to_spur(self.string_interner))),
            path.generic_args
                .iter()
                .map(|idx| self.type_to_tkind(idx, file_id)),
        );
        let tid = self.tchk.tenv.insert(ty.file_span(file_id, span));

        let strukt = Expr::Struct(StructExpr { path, fields }).at(span);
        let idx: ExprIdx = self.exprs.alloc(strukt).into();
        idx.with_type(tid)
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
                        let val = this.lower_expr(field.val(), generic_params);

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
                                    .unify(field_tid, val.tid, name.span.in_file(this.file_id()))
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
    ) -> Typed<ExprIdx> {
        let file_id = self.file_id();
        let span = block.range().to_span();
        let mut block_tid = self.tchk.tenv.insert_unit(span.in_file(file_id));
        let mut exprs = vec![];
        let stmts = block.stmts().collect::<Vec<_>>();
        let stmts_len = stmts.len();
        for i in 0..stmts_len {
            let expr = match &stmts[i] {
                ast::Stmt::ExprStmt(expr) => self.lower_expr(expr.expr(), generic_params),
                ast::Stmt::LetStmt(let_expr) => self.lower_let_expr(let_expr, generic_params),
                ast::Stmt::TerminatorExprStmt(expr) => {
                    let e = self.lower_expr(expr.expr(), generic_params);
                    block_tid = e.tid;
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
        let idx: ExprIdx = self.exprs.alloc(block).into();
        idx.with_type(block_tid)
    }

    fn lower_member_access_expr(
        &mut self,
        member_access_expr: &ast::MemberAccessExpr,
        generic_params: &GenericParams,
        member_access_kind: MemberAccessKind,
    ) -> Typed<ExprIdx> {
        let span = member_access_expr.range().to_span();
        let rhs = self.lower_name(member_access_expr.rhs());
        let lhs = self.lower_expr(member_access_expr.lhs(), generic_params);

        let lhs_tid = lhs.tid;
        let member_access = MemberAccess {
            lhs,
            rhs: rhs.clone(),
        };

        let def_map = self.def_map.unwrap();

        let mut for_type = None;
        if let TypeKind::Concrete(ConcreteKind::Path(path)) =
            self.tchk.tenv.get_typekind_with_id(lhs_tid).inner.inner
        {
            for_type = Some(self.string_interner.resolve(&path).to_string());
            let result = def_map.item_trees.iter().find_map(|(mod_id, item_tree)| {
                let result =
                    self.search_item_tree_for_struct_with_field(item_tree, path, rhs.inner);

                let file_id = def_map.modules[mod_id].file_id;

                if let Some(field) = &result {
                    let tid = self.insert_type_to_tenv(&field.ty, file_id);
                    let member_access = Expr::MemberAccess(member_access.clone()).at(span);
                    let idx: ExprIdx = self.exprs.alloc(member_access).into();
                    return Some(idx.with_type(tid));
                }

                None
            });
            if let Some(result) = result {
                return result;
            }
        }

        let item_tree_and_method = def_map.item_trees.iter().find_map(|(mod_id, item_tree)| {
            let file_id = def_map.modules[mod_id].file_id;
            self.search_item_tree_for_method(item_tree, lhs_tid, rhs.inner)
                .map(|method| (item_tree, InFile::new(method, file_id)))
        });

        let tid = if let Some((item_tree, method_idx)) = item_tree_and_method {
            let method = &item_tree[method_idx.inner];
            self.insert_type_to_tenv(&method.ret_ty, method_idx.file_id)
        } else {
            let field_or_method = self.string_interner.resolve(&rhs).to_string();
            let field_or_method_file_span = rhs.span.in_file(self.file_id());
            let err = match member_access_kind {
                MemberAccessKind::Method => LowerError::CouldNotFindMethodReferenced {
                    method: field_or_method,
                    method_file_span: field_or_method_file_span,
                    for_type,
                },
                MemberAccessKind::Field => LowerError::CouldNotFindFieldReferenced {
                    field: field_or_method,
                    field_file_span: field_or_method_file_span,
                    for_type,
                },
            };
            self.diagnostics.push(err.to_diagnostic());
            self.tchk.tenv.insert_unknown(span.in_file(self.file_id()))
        };

        let member_access = Expr::MemberAccess(member_access).at(span);
        let idx: ExprIdx = self.exprs.alloc(member_access).into();
        idx.with_type(tid)
    }

    fn lower_let_expr(
        &mut self,
        let_expr: &ast::LetStmt,
        generic_params: &GenericParams,
    ) -> Typed<ExprIdx> {
        let span = let_expr.range().to_span();
        let name = self.lower_name(let_expr.name());
        let ty = match let_expr.ty() {
            Some(ty) => self.lower_type(Some(ty), generic_params),
            None => self.types.alloc(Type::Unknown.at(name.span)).into(),
        };
        let declared_tid = self.insert_type_to_tenv(&ty, self.file_id());
        let val = self.lower_expr(let_expr.value(), generic_params);
        self.tchk
            .unify(
                declared_tid,
                val.tid,
                self.exprs[val.raw()].span.in_file(self.file_id()),
            )
            .unwrap_or_else(|err| {
                self.diagnostics.push(err);
            });

        self.tchk
            .tenv
            .insert_local_to_scope(name.inner, declared_tid);

        let l = Expr::Let(Let { name, ty, val }).at(span);
        let idx: ExprIdx = self.exprs.alloc(l).into();
        idx.with_type(self.tchk.tenv.insert_unit(span.in_file(self.file_id())))
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
                                let restrictions =
                                    this.get_restrictions_on_generic(generic.inner, generic_params);
                                Type::Generic(generic.inner, restrictions)
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

    fn get_restrictions_on_generic(
        &self,
        generic_name: Spur,
        generic_params: &GenericParams,
    ) -> Vec<Spanned<Path>> {
        generic_params
            .where_predicates
            .0
            .iter()
            .filter_map(|predicate| {
                if predicate.name.inner == generic_name {
                    Some(predicate.bound.clone())
                } else {
                    None
                }
            })
            .collect()
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
