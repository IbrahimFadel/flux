use flux_diagnostics::{Diagnostic, ToDiagnostic};
use flux_span::FileSpanned;
use flux_typesystem::{self as ts, TChecker, TEnv, TypeId};
use hashbrown::HashSet;
use la_arena::{Idx, RawIdx};
use ts::{ConcreteKind, ExpectedPathType, TraitRestriction, TypeKind};

use crate::{
    diagnostics::LowerError,
    hir::{Call, ItemDefinitionId, StructField, StructFieldList, StructId},
};

use super::*;

type ExprResult = (ExprIdx, TypeId);

pub struct ModuleBodyContext<'a> {
    tchk: TChecker,
    module_id: ModuleId,
    modules: &'a mut Arena<Module>,
    string_interner: &'static ThreadedRodeo,
    type_interner: &'static TypeInterner,
    function_namespace: &'a HashMap<Spur, (FunctionId, ModuleId)>,
    struct_namespace: &'a HashMap<Spur, (StructId, ModuleId)>,
    trait_namespace: &'a HashMap<Spur, (TraitId, ModuleId)>,
    pub(super) diagnostics: Vec<Diagnostic>,
}

impl<'a> ModuleBodyContext<'a> {
    pub fn new(
        module_id: ModuleId,
        modules: &'a mut Arena<Module>,
        string_interner: &'static ThreadedRodeo,
        type_interner: &'static TypeInterner,
        function_namespace: &'a HashMap<Spur, (FunctionId, ModuleId)>,
        struct_namespace: &'a HashMap<Spur, (StructId, ModuleId)>,
        trait_namespace: &'a HashMap<Spur, (TraitId, ModuleId)>,
    ) -> Self {
        let mut tchk = TChecker::new(
            TEnv::new(string_interner),
            // function_namespace,
            // struct_namespace,
        );
        for (name, _) in trait_namespace.iter() {
            tchk.add_trait_to_context(*name);
        }
        Self {
            tchk,
            module_id,
            modules,
            string_interner,
            type_interner,
            function_namespace,
            struct_namespace,
            trait_namespace,
            diagnostics: vec![],
        }
    }

    fn this_module(&self) -> &Module {
        &self.modules[self.module_id]
    }

    fn this_module_mut(&mut self) -> &mut Module {
        &mut self.modules[self.module_id]
    }

    fn path_to_absolute(&self, path: &Spur) -> Spur {
        let mut segments: Vec<_> = self.string_interner.resolve(path).split("::").collect();
        let first = segments.first().unwrap();
        let module_path = &self.this_module().absolute_path;
        let module_name = self.string_interner.resolve(module_path.last().unwrap());
        if first == &"pkg" {
            *path
        } else if first == &module_name {
            let rest = segments.get(1..).unwrap();
            let mut result = module_path
                .iter()
                .map(|spur| self.string_interner.resolve(spur))
                .collect::<Vec<_>>();
            result.append(&mut rest.to_vec());
            self.string_interner.get_or_intern(result.join("::"))
        } else {
            let mut result = module_path
                .iter()
                .map(|spur| self.string_interner.resolve(spur))
                .collect::<Vec<_>>();
            result.append(&mut segments);
            self.string_interner.get_or_intern(result.join("::"))
        }
    }

    fn hir_ty_to_ts_ty(
        &mut self,
        idx: &Spanned<TypeIdx>,
        where_clause: Option<&WhereClause>,
        file_id: FileId,
    ) -> TypeId {
        let ty = match self.type_interner.resolve(idx.inner).value() {
            Type::Array(ty, n) => {
                let ty = self.hir_ty_to_ts_ty(ty, where_clause, file_id);
                TypeKind::Concrete(ConcreteKind::Array(ty, n.inner))
            }
            Type::Generic(generic_name) => {
                if let Some(where_clause) = where_clause {
                    let restrictions = where_clause
                        .iter()
                        .filter_map(|where_predicate| {
                            if where_predicate.generic.inner == *generic_name {
                                let arr: Vec<TraitRestriction> = where_predicate
                                    .trait_restrictions
                                    .iter()
                                    .map(|restriction| {
                                        TraitRestriction::new(
                                            self.path_to_absolute(
                                                &restriction.path.to_spur(self.string_interner),
                                            )
                                            .in_file(file_id, restriction.path.span),
                                            restriction
                                                .args
                                                .iter()
                                                .map(|idx| {
                                                    self.hir_ty_to_ts_ty(
                                                        idx,
                                                        Some(where_clause),
                                                        file_id,
                                                    )
                                                })
                                                .collect(),
                                        )
                                    })
                                    .collect();
                                Some(arr)
                            } else {
                                None
                            }
                        })
                        .flatten()
                        .collect();

                    let ty = ts::Type::new(TypeKind::Generic, &mut self.tchk.tenv.type_interner);
                    return self
                        .tchk
                        .tenv
                        .insert_with_constraints(ty.in_file(file_id, idx.span), restrictions);
                } else {
                    TypeKind::Generic
                }
            }
            Type::Path(path, _) => {
                TypeKind::Concrete(ConcreteKind::Path(path.to_spur(self.string_interner)))
            }
            Type::Ptr(ty) => TypeKind::Concrete(ConcreteKind::Ptr(self.hir_ty_to_ts_ty(
                ty,
                where_clause,
                file_id,
            ))),
            Type::Tuple(types) => TypeKind::Concrete(ConcreteKind::Tuple(
                types
                    .iter()
                    .map(|idx| self.hir_ty_to_ts_ty(idx, where_clause, file_id))
                    .collect(),
            )),
            Type::Unknown => TypeKind::Unknown,
        };
        let ty = ts::Type::new(ty, &mut self.tchk.tenv.type_interner);
        self.tchk.tenv.insert(ty.in_file(file_id, idx.span))
    }

    pub fn lower_path(
        &self,
        path: Option<ast::Path>,
    ) -> (Spanned<Path>, Option<(ItemDefinitionId, ModuleId)>) {
        let path = lower_node(
            path,
            |path| Path::poisoned().at(path.range().to_span()),
            |path| {
                Path::from_segments(
                    path.segments()
                        .map(|tok| tok.text_key().at(tok.text_range().to_span())),
                )
                .at(path.range().to_span())
            },
        );
        let path_spur = path.to_spur(self.string_interner);

        let item_def_id = match self.resolve_path(&path_spur) {
            Some(id) => Some(id),
            None => {
                let mut module_path = self.this_module().absolute_path.clone();
                module_path.append(&mut path.get_unspanned_spurs().collect());
                let full_path_as_spur = self.string_interner.get_or_intern(
                    module_path
                        .iter()
                        .map(|spur| self.string_interner.resolve(spur))
                        .join("::"),
                );
                match self.resolve_path(&full_path_as_spur) {
                    Some(id) => Some(id),
                    None => {
                        let mut v = None;
                        for (_, use_decl) in self.this_module().uses.clone().iter() {
                            let path = &use_decl.path;
                            let use_path_spur = path.to_spur(self.string_interner);
                            if let Some(res) = self.resolve_path(&use_path_spur) {
                                v = Some(res);
                                break;
                            } else {
                                let mut module_path = self.this_module().absolute_path.clone();
                                module_path.append(&mut path.get_unspanned_spurs().collect());
                                let full_path_as_spur = self.string_interner.get_or_intern(
                                    module_path
                                        .iter()
                                        .map(|spur| self.string_interner.resolve(spur))
                                        .join("::"),
                                );
                                if let Some(res) = self.resolve_path(&full_path_as_spur) {
                                    v = Some(res);
                                    break;
                                }
                            }
                        }
                        v
                    }
                }
            }
        };

        (path, item_def_id)
    }

    // TODO: enums
    fn resolve_path(&self, path: &Spur) -> Option<(ItemDefinitionId, ModuleId)> {
        self.struct_namespace
            .get(path)
            .map(|(s_idx, mod_idx)| (ItemDefinitionId::StructId(*s_idx), *mod_idx))
    }

    pub fn lower_expr(&mut self, expr: Option<ast::Expr>) -> ExprResult {
        let expr =
            expr.expect("internal compiler error: missing node that should always be emitted");
        if expr.is_poisoned() {
            let span = expr.range().to_span();
            (
                self.this_module_mut().exprs.alloc(Expr::Poisoned.at(span)),
                self.tchk
                    .tenv
                    .insert_unknown(span.in_file(self.this_module().file_id)),
            )
        } else {
            match expr {
                ast::Expr::BlockExpr(block) => self.lower_block_expr(block),
                ast::Expr::CallExpr(call) => self.lower_call_expr(call),
                ast::Expr::FloatExpr(float) => self.lower_float_expr(float),
                ast::Expr::IntExpr(int) => self.lower_int_expr(int),
                ast::Expr::PathExpr(path) => self.lower_path_expr(path, ExpectedPathType::Any),
                ast::Expr::StructExpr(strukt) => self.lower_struct_expr(strukt),
                _ => todo!(
                    "internal compiler error: unhandled expression type: {:#?}",
                    expr
                ),
            }
        }
        // lower_node(
        //     expr,
        //     |expr| {
        //         self.module
        //             .exprs
        //             .alloc(Expr::Poisoned.at(expr.range().to_span()))
        //     },
        //     |expr| match expr {
        //         ast::Expr::BlockExpr(block) => self.lower_block_expr(block),
        //         ast::Expr::FloatExpr(float) => self.lower_float_expr(float),
        //         ast::Expr::IntExpr(int) => self.lower_int_expr(int),
        //         ast::Expr::PathExpr(path) => self.lower_path_expr(path),
        //         ast::Expr::StructExpr(strukt) => self.lower_struct_expr(strukt),
        //         _ => todo!(
        //             "internal compiler error: unhandled expression type: {:#?}",
        //             expr
        //         ),
        //     },
        // )
    }

    fn lower_block_expr(&mut self, block: ast::BlockExpr) -> ExprResult {
        let file_id = self.this_module().file_id;
        let span = block.range().to_span();
        let mut exprs = vec![];
        let stmts = block.stmts().collect::<Vec<_>>();
        let mut block_type = self.tchk.tenv.insert_unit(span.in_file(file_id));
        let stmts_len = stmts.len();
        for i in 0..stmts_len {
            let expr = match &stmts[i] {
                ast::Stmt::ExprStmt(expr) => self.lower_expr(expr.expr()),
                ast::Stmt::LetStmt(let_expr) => self.lower_let_expr(let_expr),
                ast::Stmt::TerminatorExprStmt(expr) => {
                    let (e, id) = self.lower_expr(expr.expr());
                    block_type = id;
                    if i < stmts_len - 1 {
                        self.diagnostics.push(
                            LowerError::StmtFollowingTerminatorExpr {
                                terminator: expr.range().to_span().in_file(file_id),
                                following_expr: stmts[i + 1].range().to_span().in_file(file_id),
                            }
                            .to_diagnostic(),
                        );
                    }
                    exprs.push((e, id));
                    break;
                }
            };
            exprs.push(expr);
        }
        let block = self
            .this_module_mut()
            .exprs
            .alloc(Expr::Block(Block::new(exprs)).at(span));
        (block, block_type)
    }

    // TODO: figure out the lower_node api this is retarded
    fn lower_call_expr(&mut self, call: ast::CallExpr) -> ExprResult {
        let path_expr = call
            .path()
            .expect("internal compiler error: missing node that should always be emitted");
        let (path_expr, path_ty) = if path_expr.is_poisoned() {
            let span = path_expr.range().to_span();
            (
                self.this_module_mut()
                    .exprs
                    .alloc(Expr::Path(Path::poisoned()).at(span)),
                self.tchk
                    .tenv
                    .insert_unknown(span.in_file(self.this_module().file_id)),
            )
        } else {
            self.lower_path_expr(path_expr, ExpectedPathType::Function)
        };
        let args = call
            .args()
            .expect("internal compiler error: missing node that should always be emitted");
        let args = if args.is_poisoned() {
            vec![]
        } else {
            args.args()
                .map(|arg| self.lower_expr(Some(arg)).0)
                .collect()
        };
        let call = Expr::Call(Call::new(path_expr, args)).at(call.range().to_span());
        let idx = self.this_module_mut().exprs.alloc(call);
        (idx, path_ty)
    }

    fn lower_float_expr(&mut self, float: ast::FloatExpr) -> ExprResult {
        let span = float.range().to_span();
        let float_ty = self
            .tchk
            .tenv
            .insert_float(span.in_file(self.this_module().file_id));
        let value_str = match float.v() {
            Some(v) => self
                .string_interner
                .resolve(&v.text_key())
                .at(v.text_range().to_span()),
            None => {
                return (
                    self.this_module_mut().exprs.alloc(Expr::Poisoned.at(span)),
                    float_ty,
                )
            }
        };
        let value: Spanned<f64> = value_str.map(|v| match v.parse() {
            Ok(v) => v,
            Err(_) => todo!(),
        });
        (
            self.this_module_mut()
                .exprs
                .alloc(Expr::Float(value.inner).at(span)),
            float_ty,
        )
    }

    fn lower_int_expr(&mut self, int: ast::IntExpr) -> ExprResult {
        let span = int.range().to_span();
        let int_ty = self
            .tchk
            .tenv
            .insert_int(span.in_file(self.this_module().file_id));
        let value_str = match int.v() {
            Some(v) => self
                .string_interner
                .resolve(&v.text_key())
                .at(v.text_range().to_span()),
            None => {
                return (
                    self.this_module_mut().exprs.alloc(Expr::Poisoned.at(span)),
                    int_ty,
                )
            }
        };
        let value: Spanned<u64> = value_str.map(|v| match v.parse() {
            Ok(v) => v,
            Err(_) => todo!(),
        });
        (
            self.this_module_mut()
                .exprs
                .alloc(Expr::Int(value.inner).at(span)),
            int_ty,
        )
    }

    fn lower_path_expr(&mut self, path: ast::PathExpr, expecting: ExpectedPathType) -> ExprResult {
        let file_id = self.this_module().file_id;
        let span = path.range().to_span();
        let segments = path
            .segments()
            .map(|segment| segment.text_key().at(segment.text_range().to_span()));
        let path = Path::from_segments(segments);

        let type_id =
            match self.resolve_path_expr(&path.to_spur(self.string_interner).at(span), expecting) {
                Some(id) => Some(id),
                None => {
                    let mut module_path = self.this_module().absolute_path.clone();
                    module_path.append(&mut path.get_unspanned_spurs().collect());
                    let full_path_as_spur = self.string_interner.get_or_intern(
                        module_path
                            .iter()
                            .map(|spur| self.string_interner.resolve(spur))
                            .join("::"),
                    );
                    match self.resolve_path_expr(&full_path_as_spur.at(span), expecting) {
                        Some(id) => Some(id),
                        None => {
                            let mut v = None;
                            for (_, use_decl) in self.this_module().uses.clone().iter() {
                                let path = &use_decl.path;
                                let use_path_spur = path.to_spur(self.string_interner);
                                if let Some(res) =
                                    self.resolve_path_expr(&use_path_spur.at(span), expecting)
                                {
                                    v = Some(res);
                                    break;
                                } else {
                                    let mut module_path = self.this_module().absolute_path.clone();
                                    module_path.append(&mut path.get_unspanned_spurs().collect());
                                    let full_path_as_spur = self.string_interner.get_or_intern(
                                        module_path
                                            .iter()
                                            .map(|spur| self.string_interner.resolve(spur))
                                            .join("::"),
                                    );
                                    if let Some(res) = self
                                        .resolve_path_expr(&full_path_as_spur.at(span), expecting)
                                    {
                                        v = Some(res);
                                        break;
                                    }
                                }
                            }
                            v
                        }
                    }
                }
            };

        let type_id = type_id.unwrap_or_else(|| {
            let path = self
                .string_interner
                .resolve(&path.to_spur(self.string_interner))
                .to_string()
                .in_file(file_id, span);
            let diagnostic = match expecting {
                ExpectedPathType::Any => LowerError::CouldNotResolvePath { path },
                ExpectedPathType::Local => todo!(),
                ExpectedPathType::Variable => todo!(),
                ExpectedPathType::Function => LowerError::CouldNotResolveFunction { path },
            };
            self.diagnostics.push(diagnostic.to_diagnostic());
            self.tchk.tenv.insert_unknown(span.in_file(file_id))
        });

        let expr = Expr::Path(path);
        (self.this_module_mut().exprs.alloc(expr.at(span)), type_id)
    }

    fn resolve_path_expr(
        &mut self,
        path: &Spanned<Spur>,
        expecting: ExpectedPathType,
    ) -> Option<TypeId> {
        match expecting {
            ExpectedPathType::Any => self
                .resolve_path_expr(path, ExpectedPathType::Local)
                .or_else(|| self.resolve_path_expr(path, ExpectedPathType::Variable))
                .or_else(|| self.resolve_path_expr(path, ExpectedPathType::Function)),
            ExpectedPathType::Function => {
                self.function_namespace.get(path).map(|(f_idx, mod_idx)| {
                    let m = &self.modules[*mod_idx];
                    let file_id = m.file_id;
                    let f = &m.functions[*f_idx];
                    let ret_ty = &f.ret_type;
                    self.hir_ty_to_ts_ty(&ret_ty.clone(), None, file_id)
                })
            }
            ExpectedPathType::Local => self
                .tchk
                .tenv
                .get_local_typeid(path.clone().in_file(self.this_module().file_id))
                .ok(),
            ExpectedPathType::Variable => None,
        }
    }

    fn lower_struct_expr(&mut self, strukt: ast::StructExpr) -> ExprResult {
        let file_id = self.this_module().file_id;
        let span = strukt.range().to_span();
        let (path, path_item_def_id) = self.lower_path(strukt.path());
        let field_list = lower_node(
            strukt.field_list(),
            |strukt| {
                println!("POISONED");
                StructFieldList::EMPTY.at(strukt.range().to_span())
            },
            |strukt| {
                StructFieldList::new(
                    strukt
                        .fields()
                        .map(|field| self.lower_struct_expr_field(field))
                        .collect(),
                )
                .at(strukt.range().to_span())
            },
        );
        let ty = ts::Type::new(
            TypeKind::Concrete(ConcreteKind::Path(path.to_spur(self.string_interner))),
            &mut self.tchk.tenv.type_interner,
        )
        .in_file(file_id, span);
        let ty = self.tchk.tenv.insert(ty);
        let struct_path_spur = path.to_spur(self.string_interner);
        if let Some((s_idx, mod_idx)) = path_item_def_id {
            // When the field list is poisoned, we initialize it as empty which makes typechecking not just useless, but wrong. It gives weird diagnostics (like incorrect number of fields)
            if strukt
                .field_list()
                .is_some_and(|field_list| !field_list.is_poisoned())
            {
                let s_idx = match s_idx {
                    ItemDefinitionId::StructId(id) => id,
                    _ => todo!(),
                };
                let module = &self.modules[mod_idx];
                self.unify_struct_expr_with_decl(
                    &struct_path_spur,
                    &field_list,
                    &module.structs[s_idx]
                        .field_list
                        .clone()
                        .in_file(module.file_id),
                );
            }
        } else {
            self.diagnostics.push(
                LowerError::CouldNotResolveStruct {
                    path: self
                        .string_interner
                        .resolve(&struct_path_spur)
                        .to_string()
                        .in_file(file_id, span),
                }
                .to_diagnostic(),
            );
        }

        let strukt = Struct::new(path.inner, field_list.inner);
        (
            self.this_module_mut()
                .exprs
                .alloc(Expr::Struct(strukt).at(span)),
            ty,
        )
    }

    fn unify_struct_expr_with_decl(
        &mut self,
        struct_name: &Spur,
        struct_expr_field_list: &Spanned<StructFieldList>,
        struct_decl_field_list: &FileSpanned<StructDeclFieldList>,
    ) {
        let mut initialized_fields = HashSet::new();
        let mut initialized_fields_that_dont_exist = vec![];
        let mut fields_expected_to_be_initialized = HashMap::new();
        for field in struct_decl_field_list.iter() {
            fields_expected_to_be_initialized.insert(
                field.name.inner,
                self.hir_ty_to_ts_ty(&field.ty, None, struct_decl_field_list.file_id),
            );
        }
        for field in struct_expr_field_list.iter() {
            initialized_fields.insert(field.name.inner);
            match fields_expected_to_be_initialized.get(&field.name.inner) {
                None => {
                    initialized_fields_that_dont_exist.push(field.name.inner);
                    continue;
                }
                Some(decl_tyid) => {
                    self.tchk
                        .unify(
                            *decl_tyid,
                            field.ty,
                            self.tchk.tenv.get_type_filespan(field.ty),
                        )
                        .unwrap_or_else(|err| {
                            self.diagnostics.push(err);
                        });
                }
            }
        }

        let uninitialized_fields = fields_expected_to_be_initialized
            .keys()
            .filter(|field| !initialized_fields.contains(*field))
            .map(|spur| self.string_interner.resolve(spur))
            .join(", ");

        let struct_name = self.string_interner.resolve(struct_name).to_string();
        if !uninitialized_fields.is_empty() {
            self.diagnostics.push(
                LowerError::UninitializedFieldsInStructExpr {
                    struct_name: struct_name.clone(),
                    missing_fields: uninitialized_fields
                        .in_file(self.this_module().file_id, struct_expr_field_list.span),
                    declaration_span: struct_decl_field_list.to_filespan(),
                }
                .to_diagnostic(),
            )
        }
        if !initialized_fields_that_dont_exist.is_empty() {
            self.diagnostics.push(
                LowerError::UnnecessaryFieldsInitializedInStructExpr {
                    struct_name,
                    unnecessary_fields: initialized_fields_that_dont_exist
                        .iter()
                        .map(|spur| self.string_interner.resolve(spur))
                        .join(", ")
                        .in_file(self.this_module().file_id, struct_expr_field_list.span),
                    declaration_span: struct_decl_field_list.to_filespan(),
                }
                .to_diagnostic(),
            );
        }
    }

    fn lower_struct_expr_field(&mut self, field: ast::StructExprField) -> StructField {
        let name = lower_name(field.name(), self.string_interner);
        let (val, val_id) = self.lower_expr(field.val());
        StructField::new(name, val, val_id)
    }

    fn lower_let_expr(&mut self, let_expr: &ast::LetStmt) -> ExprResult {
        let span = let_expr.range().to_span();
        let name = lower_name(let_expr.name(), self.string_interner);
        let lhs_ty = let_expr.ty().map_or(
            self.type_interner.intern(Type::Unknown).at(name.span),
            |ty| {
                lower_type(
                    Some(ty),
                    &GenericParamList::empty(),
                    span,
                    self.string_interner,
                    self.type_interner,
                )
            },
        );
        let lhs_ty_id = self.hir_ty_to_ts_ty(&lhs_ty, None, self.this_module().file_id);
        let (expr, expr_id) = self.lower_expr(let_expr.value());
        self.tchk
            .unify(
                lhs_ty_id,
                expr_id,
                lhs_ty.span.in_file(self.this_module().file_id),
            )
            .unwrap_or_else(|err| {
                self.diagnostics.push(err);
            });
        self.tchk.tenv.insert_local_to_scope(name.inner, lhs_ty_id);
        let let_expr = self
            .this_module_mut()
            .exprs
            .alloc(Expr::Let(Let::new(name, lhs_ty, expr)).at(span));
        (let_expr, expr_id)
    }

    pub fn lower_bodies(&mut self) {
        let file_id = self.this_module().file_id;
        for i in 0..self.this_module().functions.len() {
            let f = self.this_module().functions[Idx::from_raw(RawIdx::from(i as u32))].clone();

            for param in f.params.clone().iter() {
                let ty = self.hir_ty_to_ts_ty(&param.ty, Some(&f.where_clause), file_id);
                self.tchk.tenv.insert_local_to_scope(param.name.inner, ty);
            }

            let body = f.ast.body();
            let ret_ty = f.ret_type;
            let (_, body_tyid) = self.lower_expr(body);
            let ret_tyid = self.hir_ty_to_ts_ty(&ret_ty, Some(&f.where_clause), file_id);
            self.tchk
                .unify(ret_tyid, body_tyid, ret_ty.span.in_file(file_id))
                .unwrap_or_else(|err| {
                    self.diagnostics.push(err);
                });
        }
    }
}
