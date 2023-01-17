use flux_diagnostics::Diagnostic;
use flux_span::{FileId, Spanned, WithSpan};
use la_arena::Arena;
use lasso::ThreadedRodeo;
use tracing::instrument;

use crate::{
    hir::{self, Block, Expr, ExprIdx, FnDecl, Let, Path},
    package_defs::{ModuleData, PackageDefs},
    type_interner::{TypeIdx, TypeInterner},
    Module,
};

mod constraints;
mod diagnostics;
mod env;
mod name_resolver;
mod r#type;

use self::{
    constraints::Constraint,
    env::TEnv,
    r#type::{ConcreteKind, FnSignature, Type, TypeKind},
};

pub(super) struct TChecker<'a> {
    env: TEnv,
    package_defs: &'a PackageDefs,
    // module_data: &'a ModuleData,
    string_interner: &'static ThreadedRodeo,
    // env: TEnv,
    // exprs: &'a Arena<Spanned<Expr>>,
    // types: &'a TypeInterner,
    // string_interner: &'static ThreadedRodeo,
    // file_id: FileId,
}

impl<'a> TChecker<'a> {
    pub fn new(package_defs: &'a PackageDefs, string_interner: &'static ThreadedRodeo) -> Self {
        Self {
            env: TEnv::new(string_interner),
            package_defs,
            string_interner,
        }
    }

    pub fn check_package(&mut self) {
        for (module_data_idx, item_scope) in self.package_defs.modules.iter() {
            let module_data = self.package_defs.get_module_data(module_data_idx);
        }
    }
    // pub fn new(module_data: &'a ModuleData, string_interner: &'static ThreadedRodeo) -> Self {
    //     Self {
    //         env: TEnv::new(string_interner),
    //         module_data,
    //         string_interner,
    //     }
    // }
    // pub fn new(
    //     file_id: FileId,
    //     exprs: &'a Arena<Spanned<Expr>>,
    //     types: &'a TypeInterner,
    //     string_interner: &'static ThreadedRodeo,
    // ) -> Self {
    //     Self {
    //         env: TEnv::new(string_interner),
    //         exprs,
    //         types,
    //         string_interner,
    //         file_id,
    //     }
    // }

    pub fn check_functions(&mut self) {
        for (_, f) in self.module_data.module.functions.iter() {
            let sig = self.get_fn_signature(f);
        }
    }

    pub fn get_fn_signature(&mut self, f: &FnDecl) -> FnSignature {
        let vis = f.visibility;
        let param_types = f
            .params
            .iter()
            .map(|param| {
                self.hir_ty_to_ts_ty(&param.ty)
                    .in_file(self.module_data.file_id)
            })
            .collect();
        let return_ty = self
            .hir_ty_to_ts_ty(&f.ret_type)
            .in_file(self.module_data.file_id);
        FnSignature::new(vis, param_types, return_ty)
    }

    #[instrument(level = "info", name = "Typechk::check_function", skip_all)]
    pub fn check_fn_decl(&mut self, f: &mut FnDecl) -> Vec<Diagnostic> {
        self.env.reset();
        let body_type = self.check_expr(f.body.inner);
        let body_typeid = self.env.insert(body_type.in_file(
            self.module_data.file_id,
            self.module_data.module.exprs[f.body.inner].span,
        ));
        let return_ty = self.hir_ty_to_ts_ty(&f.ret_type);
        let return_ty_span = return_ty.span;
        let return_typeid = self.env.insert(return_ty.in_file(self.module_data.file_id));

        self.env.push_constraint(Constraint::TypeEq(
            body_typeid,
            return_typeid,
            return_ty_span.in_file(self.module_data.file_id),
        ));

        self.env.solve_constraints();
        std::mem::take(&mut self.env.diagnostics)
    }

    fn check_expr(&mut self, idx: ExprIdx) -> Type {
        match &self.module_data.module.exprs[idx].inner {
            Expr::Block(block) => self.check_block_expr(block),
            Expr::Float(_) => Type::new(TypeKind::Float(None)),
            Expr::Int(_) => Type::new(TypeKind::Int(None)),
            Expr::Let(let_expr) => self.check_let_expr(let_expr),
            Expr::Path(path) => self.check_path_expr(path),
            _ => todo!(),
        }
    }

    fn check_block_expr(&mut self, block: &Block) -> Type {
        block
            .iter()
            .map(|expr| self.check_expr(*expr))
            .last()
            .unwrap_or(Type::new(TypeKind::Concrete(ConcreteKind::Tuple(vec![]))))
    }

    fn check_let_expr(&mut self, let_expr: &Let) -> Type {
        let lhs_ty = self.hir_ty_to_ts_ty(&let_expr.ty);
        let lhs_span = lhs_ty.span;
        let lhs_tyid = self.env.insert(lhs_ty.in_file(self.module_data.file_id));
        let rhs_ty = self.check_expr(let_expr.val.inner);
        let rhs_span = self.module_data.module.exprs[let_expr.val.inner].span;
        let rhs_tyid = self
            .env
            .insert(rhs_ty.in_file(self.module_data.file_id, rhs_span));
        self.env.push_constraint(Constraint::TypeEq(
            lhs_tyid,
            rhs_tyid,
            lhs_span.in_file(self.module_data.file_id),
        ));
        self.env.insert_var(let_expr.name.inner, lhs_tyid);
        Type::new(TypeKind::Concrete(ConcreteKind::Tuple(vec![])))
    }

    fn check_path_expr(&mut self, path: &Path) -> Type {
        // self.env.name_resolver.resolve_path();
        todo!()
    }

    fn hir_ty_to_ts_ty(&mut self, idx: &Spanned<TypeIdx>) -> Spanned<Type> {
        let ty = self.module_data.module.types.resolve(idx.inner);
        let (tykind, params) = match ty {
            hir::Type::Array(t, n) => {
                let t = self.hir_ty_to_ts_ty(t);
                let t = self.env.insert(t.in_file(self.module_data.file_id));
                (TypeKind::Concrete(ConcreteKind::Array(t, n.inner)), None)
            }
            hir::Type::Generic(name) => (TypeKind::Generic(name.clone()), None),
            hir::Type::Path(path, params) => {
                let kind =
                    TypeKind::Concrete(ConcreteKind::Path(path.to_spur(self.string_interner)));
                let params = params
                    .iter()
                    .map(|param| self.hir_ty_to_ts_ty(param).inner)
                    .collect();
                (kind, Some(params))
            }
            hir::Type::Ptr(ty) => {
                let t = self.hir_ty_to_ts_ty(ty);
                let t = self.env.insert(t.in_file(self.module_data.file_id));
                (TypeKind::Concrete(ConcreteKind::Ptr(t)), None)
            }
            hir::Type::Tuple(types) => {
                let types = types
                    .iter()
                    .map(|ty| {
                        let t = self.hir_ty_to_ts_ty(ty);
                        self.env.insert(t.in_file(self.module_data.file_id))
                    })
                    .collect();
                (TypeKind::Concrete(ConcreteKind::Tuple(types)), None)
            }
            hir::Type::Unknown => (TypeKind::Unknown, None),
        };
        match params {
            Some(params) => Type::with_params(tykind, params).at(idx.span),
            None => Type::new(tykind).at(idx.span),
        }
    }

    // fn get_expr_ty(&mut self, idx: ExprIdx) -> Spanned<Type> {
    //     let e = &self.exprs[idx];
    //     let tykind = match &e.inner {
    //         hir::Expr::Block(_) => todo!(),
    //         hir::Expr::Float(_) => TypeKind::Float(None),
    //         hir::Expr::Int(_) => TypeKind::Int(None),
    //         hir::Expr::Let(_) => TypeKind::Concrete(ConcreteKind::Tuple(vec![])),
    //         hir::Expr::Path(path) => {
    //             let id = self
    //                 .env
    //                 .get_path_typeid(&path.to_spur(self.string_interner));
    //             return self.env.get_type_with_id(id).clone().inner;
    //         }
    //         hir::Expr::Poisoned => TypeKind::Unknown,
    //         hir::Expr::Tuple(exprs) => TypeKind::Concrete(ConcreteKind::Tuple(
    //             exprs
    //                 .iter()
    //                 .map(|e| {
    //                     let t = self.get_expr_ty(*e);
    //                     self.env.insert(t.in_file(self.file_id))
    //                 })
    //                 .collect(),
    //         )),
    //         hir::Expr::Struct(strukt) => TypeKind::Concrete(ConcreteKind::Path(
    //             strukt.path.to_spur(self.string_interner),
    //         )), // hir::Expr::Struct(strukt) => {
    //             //     TypeKind::Concrete(ConcreteKind::Struct(StructConcreteKind::new(
    //             //         vec![],
    //             //         strukt
    //             //             .fields
    //             //             .iter()
    //             //             .map(|(name, expr_idx)| {
    //             //                 let ty = self.get_expr_ty(*expr_idx).in_file(self.file_id);
    //             //                 (name.inner, self.env.insert(ty))
    //             //             })
    //             //             .collect(),
    //             //     )))
    //             // }
    //     };
    //     Type::new(tykind).at(e.span)
    // }
}

pub fn tychk_hir_module(
    file_id: FileId,
    module: &mut Module,
    exprs: &Arena<Spanned<Expr>>,
    types: &TypeInterner,
    string_interner: &'static ThreadedRodeo,
) -> Vec<Diagnostic> {
    // let num_functions = module.functions.len();
    // let mut tchecker = TChecker::new(file_id, exprs, types, string_interner);
    let diagnostics = vec![];
    // for i in 0..num_functions {
    //     let f = &mut module.functions[i];
    //     let sig = tchecker.get_fn_signature(f);
    // }
    // for i in 0..num_functions {
    //     let f = &mut module.functions[i];
    //     let mut f_diagnostics = tchecker.check_fn_decl(f);
    //     diagnostics.append(&mut f_diagnostics);
    // }
    diagnostics
}

pub fn tychk_package(package_defs: &mut PackageDefs, string_interner: &'static ThreadedRodeo) {
    let mut tchecker = TChecker::new(package_defs, string_interner);
    tchecker.check_package();
    // for (module_data_idx, item_scope) in package_defs.modules.iter() {
    // let module_data = package_defs.get_module_data(module_data_idx);
    // let mut tchecker = TChecker::new(module_data, string_interner);
    // tchecker.check_functions();
    // }
}
