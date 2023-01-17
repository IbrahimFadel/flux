use flux_diagnostics::Diagnostic;
use flux_span::{FileId, Spanned, WithSpan};
use itertools::Itertools;
use la_arena::{Arena, Idx};
use lasso::{Spur, ThreadedRodeo};
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
mod expr;
mod name_resolver;
mod scope;
mod r#type;

use self::{
    constraints::Constraint,
    env::TEnv,
    name_resolver::NameResolver,
    r#type::{ConcreteKind, FnSignature, Type, TypeKind},
    scope::Scope,
};

pub(super) struct TChecker<'a> {
    env: TEnv,
    // package_defs: &'a PackageDefs,
    // mod_idx: Idx<ModuleData>,
    module: &'a ModuleData,
    local_scopes: Vec<Scope>,
    name_resolver: &'a NameResolver,
    string_interner: &'static ThreadedRodeo,
}

impl<'a> TChecker<'a> {
    // pub fn new(package_defs: &'a PackageDefs, string_interner: &'static ThreadedRodeo) -> Self {
    //     Self {
    //         env: TEnv::new(string_interner),
    //         package_defs,
    //         mod_idx: Idx::from_raw(0.into()),
    //         name_resolver: NameResolver::from_package_defs(package_defs, string_interner),
    //         string_interner,
    //     }
    // }

    pub fn new(
        module: &'a ModuleData,
        name_resolver: &'a NameResolver,
        string_interner: &'static ThreadedRodeo,
    ) -> Self {
        Self {
            env: TEnv::new(string_interner),
            module,
            local_scopes: vec![Scope::new()],
            name_resolver,
            string_interner,
        }
    }

    pub fn check_module(&mut self) {
        // println!(
        //     "{}",
        //     self.name_resolver
        //         .use_paths
        //         .iter()
        //         .map(|(m_id, path)| {
        //             format!(
        //                 "Module {:?} uses {}",
        //                 m_id,
        //                 self.string_interner.resolve(path)
        //             )
        //         })
        //         .join("\n")
        // );
        // println!(
        //     "{}",
        //     self.name_resolver
        //         .absolute_path_map
        //         .iter()
        //         .map(|(path, (module_id, item_id))| {
        //             format!(
        //                 "{} -> {:?} {:?}",
        //                 self.string_interner.resolve(path),
        //                 module_id,
        //                 item_id
        //             )
        //         })
        //         .join("\n")
        // );
        // for (module_data_idx, item_scope) in self.package_defs.modules.iter() {
        //     self.mod_idx = module_data_idx;
        //     let module_data = self.package_defs.get_module_data(module_data_idx);
        //     for (_, f) in module_data.module.functions.iter() {
        //         let sig = self.get_fn_signature(f);
        //         let file_id = self.module.file_id;
        //         // println!("{} {:#?}", self.string_interner.resolve(&file_id.0), sig);
        //     }
        // }

        // for (module_data_idx, item_scope) in self.package_defs.modules.iter() {
        //     self.mod_idx = module_data_idx;
        //     let module_data = self.package_defs.get_module_data(module_data_idx);
        //     for (_, f) in module_data.module.functions.iter() {
        //         self.check_function(f);
        //     }
        // }

        for (_, f) in self.module.module.functions.iter() {
            self.check_function(f);
        }
    }

    fn get_fn_signature(&mut self, f: &FnDecl) -> FnSignature {
        let vis = f.visibility;
        let param_types = f
            .params
            .iter()
            .map(|param| self.hir_ty_to_ts_ty(&param.ty).in_file(self.module.file_id))
            .collect();
        let return_ty = self
            .hir_ty_to_ts_ty(&f.ret_type)
            .in_file(self.module.file_id);
        FnSignature::new(vis, param_types, return_ty)
    }

    fn hir_ty_to_ts_ty(&mut self, idx: &Spanned<TypeIdx>) -> Spanned<Type> {
        let ty = self.module.module.types.resolve(idx.inner);
        let (tykind, params) = match ty {
            hir::Type::Array(t, n) => {
                let t = self.hir_ty_to_ts_ty(t);
                let t = self.env.insert(t.in_file(self.module.file_id));
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
                let t = self.env.insert(t.in_file(self.module.file_id));
                (TypeKind::Concrete(ConcreteKind::Ptr(t)), None)
            }
            hir::Type::Tuple(types) => {
                let types = types
                    .iter()
                    .map(|ty| {
                        let file_id = self.module.file_id;
                        let t = self.hir_ty_to_ts_ty(ty);
                        self.env.insert(t.in_file(file_id))
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

    fn check_function(&mut self, f: &FnDecl) {
        self.check_expr(f.body.inner);
    }
}

pub fn tychk_package(package_defs: &mut PackageDefs, string_interner: &'static ThreadedRodeo) {
    let name_resolver = NameResolver::from_package_defs(package_defs, string_interner);
    for (module_data_idx, _) in package_defs.modules.iter() {
        let module = package_defs.get_module_data(module_data_idx);
        let mut tchecker = TChecker::new(module, &name_resolver, string_interner);
        tchecker.check_module();
    }
}
