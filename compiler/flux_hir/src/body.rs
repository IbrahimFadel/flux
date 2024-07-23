use std::collections::HashMap;

use flux_diagnostics::{ice, Diagnostic, ToDiagnostic};
use flux_span::{FileId, Interner, Spanned, ToSpan, WithSpan, Word};
use flux_syntax::{
    ast::{self, AstNode},
    SyntaxToken,
};
use flux_typesystem::{
    self as ts, ConcreteKind, Insert, TChecker, TEnv, TypeId, TypeKind, Typed, WithType,
};
use la_arena::{Arena, Idx};
use ts::{ApplicationId, FnSignature, ThisCtx, TraitId};

use crate::{
    diagnostics::LowerError,
    hir::{ApplyDecl, Expr, ExprIdx, FnDecl, GenericParams, Op, Path, StructDecl, TraitDecl, Type},
    item::{ItemId, ItemTreeIdx},
    item_tree::ItemTree,
    module::{ModuleData, ModuleId, ModuleTree},
    name_res::item::ItemResolver,
    pkg::PackageBodies,
    BuiltPackage, PackageDefs, PackageId, POISONED_NAME,
};

mod expr;
mod stmt;
mod r#type;

pub(crate) struct LowerCtx<'a, 'pkgs> {
    diagnostics: Vec<Diagnostic>,
    item_resolver: Option<ItemResolver<'a>>,
    pub interner: &'static Interner,
    pub tckh: TChecker<'a>,
    package_bodies: &'a mut PackageBodies,
    pub(super) file_id: FileId,
    pub(crate) module_id: ModuleId,
    package_id: Option<PackageId>,
    packages: Option<&'pkgs Arena<PackageDefs>>,
    dependencies: &'a [BuiltPackage],
    trait_map: HashMap<(PackageId, Idx<TraitDecl>), TraitId>,
    application_map: HashMap<(PackageId, Idx<ApplyDecl>), ApplicationId>,
}

impl<'a, 'pkgs> LowerCtx<'a, 'pkgs> {
    pub(crate) fn new(
        item_resolver: Option<ItemResolver<'a>>,
        packages: Option<&'pkgs Arena<PackageDefs>>,
        package_id: Option<PackageId>,
        module_id: ModuleId,
        dependencies: &'a [BuiltPackage],
        package_bodies: &'a mut PackageBodies,
        tenv: &'a mut TEnv,
        interner: &'static Interner,
        file_id: FileId,
    ) -> Self {
        Self {
            diagnostics: vec![],
            item_resolver,
            interner,
            tckh: TChecker::new(tenv),
            package_bodies,
            file_id,
            module_id,
            package_id,
            dependencies,
            packages,
            trait_map: HashMap::new(),
            application_map: HashMap::new(),
        }
    }

    fn item_tree(&self, package_id: PackageId) -> &'pkgs ItemTree {
        debug_assert!(self.packages.is_some());
        &self.packages.unwrap()[package_id].item_tree
    }

    fn item_resolver(&self) -> &ItemResolver<'a> {
        self.item_resolver.as_ref().unwrap()
    }

    fn prelude(&self) -> &ModuleData {
        &self.packages.unwrap()[self.package_id.unwrap()].module_tree[ModuleTree::PRELUDE_ID]
    }

    fn file_id(&self, package_id: PackageId, module_id: ModuleId) -> FileId {
        self.packages.unwrap()[package_id].module_tree[module_id].file_id
    }

    fn resolve_path(&mut self, path: &Spanned<Path>) -> Option<(PackageId, ItemId)> {
        self.item_resolver()
            .resolve_path(&path, self.module_id)
            .map_err(|resolution_error| {
                self.diagnostics.push(resolution_error.to_diagnostic(
                    self.file_id,
                    path.span,
                    self.interner,
                ));
            })
            .ok()
    }

    pub(crate) fn set_module_id(&mut self, module_id: ModuleId) {
        self.module_id = module_id;
    }

    pub(crate) fn set_file_id(&mut self, file_id: FileId) {
        self.file_id = file_id;
    }

    pub(crate) fn set_package_id(&mut self, package_id: PackageId) {
        self.package_id = Some(package_id);
    }

    // pub(crate) fn populate_trait_map(&mut self) {
    //     let item_tree = self.item_tree(self.package_id.unwrap());
    //     item_tree.top_level.iter().for_each(|item| match item.idx {
    //         ItemTreeIdx::Trait(trait_idx) => self.add_trait(trait_idx),
    //         _ => {}
    //     })
    // }

    fn get_trait_id(&mut self, path: &Spanned<Path>) -> Option<TraitId> {
        self.item_resolver()
            .resolve_path(&path, self.module_id)
            .map(|(package_id, item_id)| {
                let trait_idx = match item_id.idx {
                    ItemTreeIdx::Trait(trait_idx) => trait_idx,
                    _ => ice("progammer got lazy to impl this diagnostic"),
                };
                Some(
                    self.trait_map
                        .get(&(package_id, trait_idx))
                        .unwrap_or_else(|| ice("trait wasn't added to trait map")),
                )
            })
            .unwrap_or_else(|resolution_err| {
                self.diagnostics.push(resolution_err.to_diagnostic(
                    self.file_id,
                    path.span,
                    self.interner,
                ));
                None
            })
            .copied()
    }

    pub(crate) fn attach_trait_type_contexts(&mut self) {
        let og_mod_id = self.module_id;
        let og_package_id = self.package_id.unwrap();
        let item_tree = self.item_tree(og_package_id);
        let packages = self.packages.unwrap();
        item_tree.top_level.iter().for_each(|item| {
            self.set_module_id(item.mod_id);
            let file_id = packages[og_package_id].module_tree[item.mod_id].file_id;
            self.set_file_id(file_id);
            match item.idx {
                ItemTreeIdx::Trait(trait_idx) => self.handle_trait_decl(trait_idx, og_package_id),
                _ => {}
            }
        });
        item_tree.top_level.iter().for_each(|item| {
            self.set_module_id(item.mod_id);
            let file_id = packages[og_package_id].module_tree[item.mod_id].file_id;
            self.set_file_id(file_id);
            match item.idx {
                ItemTreeIdx::Apply(apply_idx) => {
                    if item_tree.applies[apply_idx].trt.is_some() {
                        self.handle_apply_decl(apply_idx)
                    }
                }
                _ => {}
            }
        });

        // Also need to insert applications and change `ThisPath` data. But we cant mutate, so need to store this in BuiltPackage somehow
        for dep in self.dependencies {
            // dep.appl
            // self.tckh.insert_trait_application(trid, ts::Application::new(tid, assoc_types, signatures))
            dep.traits.iter().for_each(|(package_id, trait_idx, trt)| {
                let signatures = trt
                    .methods
                    .iter()
                    .map(|method_idx| {
                        let method = &dep.item_tree.functions[*method_idx];
                        let signature_types = method
                            .params
                            .iter()
                            .map(|param| param.ty.inner)
                            .chain(std::iter::once(method.return_ty.inner))
                            .map(|tid| dep.tenv.reconstruct(&tid))
                            .map(|r| match r {
                                Ok(r) => self.tckh.tenv.insert(r),
                                Err((file_span, err)) => {
                                    self.diagnostics.push(err);
                                    self.tckh.tenv.insert(
                                        TypeKind::Unknown
                                            .file_span(file_span.file_id, file_span.inner),
                                    )
                                }
                            });
                        FnSignature::from_type_ids(signature_types)
                    })
                    .collect();
                let ts_trait = ts::Trait::new(signatures);
                let trid = self.tckh.tenv.insert_trait(ts_trait);
                self.trait_map.insert((*package_id, *trait_idx), trid);
            });
        }

        // for package_id in self.item_resolver().dependency_ids {
        //     let pkg = &packages[*package_id];
        //     pkg.item_tree.top_level.iter().for_each(|item| {
        //         self.set_module_id(item.mod_id);
        //         self.set_package_id(*package_id);
        //         let file_id = packages[*package_id].module_tree[item.mod_id].file_id;
        //         self.set_file_id(file_id);
        //         match item.idx {
        //             ItemTreeIdx::Trait(trait_idx) => self.handle_trait_decl(trait_idx, *package_id),
        //             _ => {}
        //         }
        //     });
        // }
        self.set_package_id(og_package_id);
        self.set_module_id(og_mod_id);
    }

    fn handle_trait_decl(&mut self, trait_idx: Idx<TraitDecl>, package_id: PackageId) {
        let item_tree = self.item_tree(package_id);
        let trait_decl = &item_tree.traits[trait_idx];

        let trid = self
            .tckh
            .tenv
            .insert_trait(ts::Trait::new(trait_decl.method_signatures(item_tree)));
        let this_ctx = ThisCtx::new(Some(trid), None);

        trait_decl.methods.iter().for_each(|method_idx| {
            self.attach_this_ctx_to_fn_dec(*method_idx, this_ctx.clone());
        });

        self.trait_map.insert((package_id, trait_idx), trid);
    }

    // pub(crate) fn populate_applications(&mut self) {
    //     let og_mod_id = self.module_id;
    //     let item_tree = self.item_tree(self.package_id.unwrap());
    //     item_tree.top_level.iter().for_each(|item| {
    //         self.set_module_id(item.mod_id);
    //         match item.idx {
    //             ItemTreeIdx::Apply(apply_idx) => self.add_application(apply_idx),
    //             _ => {}
    //         }
    //     });
    //     self.set_module_id(og_mod_id);
    // }

    pub(crate) fn finish(self) -> Vec<Diagnostic> {
        self.diagnostics
    }

    pub(crate) fn lower_module_bodies(&mut self, module_id: ModuleId) {
        let file_id =
            self.packages.unwrap()[self.package_id.unwrap()].module_tree[module_id].file_id;
        self.set_module_id(module_id);
        self.set_file_id(file_id);
        let this_mod_id = module_id;
        let item_tree = self.item_tree(
            self.package_id
                .unwrap_or_else(|| ice("tried lowering body without active item tree")),
        );
        for item in item_tree
            .top_level
            .iter()
            .filter(|item| item.mod_id == this_mod_id)
        {
            match &item.idx {
                ItemTreeIdx::Apply(apply_idx) => {
                    if item_tree.applies[*apply_idx].trt.is_none() {
                        self.handle_apply_decl(*apply_idx)
                    }
                }
                ItemTreeIdx::Function(fn_idx) => {
                    self.lower_function_body(*fn_idx);
                }
                ItemTreeIdx::Struct(struct_idx) => self.handle_struct_decl(*struct_idx),
                _ => {}
            }
        }
    }

    fn handle_apply_decl(&mut self, apply_idx: Idx<ApplyDecl>) {
        let item_tree = self.item_tree(self.package_id.unwrap());
        let apply_decl = &item_tree.applies[apply_idx];
        let to_ty = apply_decl.to_ty.inner;
        let trid = apply_decl
            .trt
            .as_ref()
            .map(|trt| self.get_trait_id(trt))
            .flatten();

        let assoc_types = apply_decl
            .assoc_types
            .iter()
            .map(|assoc_type| assoc_type.as_ts_assoc_type())
            .collect();
        let signatures = apply_decl.method_signatures(item_tree);
        let aid = match trid {
            Some(trid) => self.tckh.insert_trait_application(
                trid,
                ts::Application::new(to_ty, assoc_types, signatures),
            ),
            None => self.tckh.insert_application(to_ty, signatures),
        };
        let this_ctx = ThisCtx::new(trid, Some(aid));

        self.tckh
            .tenv
            .insert_local(self.interner.get_or_intern_static("this"), to_ty);
        apply_decl.methods.iter().for_each(|method_idx| {
            self.attach_this_ctx_to_fn_dec(*method_idx, this_ctx.clone());
            apply_decl.assoc_types.iter().for_each(|assoc_ty| {
                self.tckh
                    .tenv
                    .attach_this_ctx(&assoc_ty.ty, this_ctx.clone());
            });
            self.lower_function_body(*method_idx);
        });
    }

    fn attach_this_ctx_to_fn_dec(&mut self, idx: Idx<FnDecl>, this_ctx: ThisCtx) {
        let item_tree = self.item_tree(self.package_id.unwrap());
        let f = &item_tree.functions[idx];
        f.params.iter().for_each(|param| {
            self.tckh.tenv.attach_this_ctx(&param.ty, this_ctx.clone());
        });
        self.tckh
            .tenv
            .attach_this_ctx(&f.return_ty, this_ctx.clone());
    }

    fn lower_function_body(&mut self, fn_idx: Idx<FnDecl>) {
        let fn_decl = &self.item_tree(self.package_id.unwrap()).functions[fn_idx];
        if let Some(ast) = &fn_decl.ast {
            fn_decl.params.iter().for_each(|param| {
                self.tckh
                    .tenv
                    .insert_local(param.name.inner, param.ty.inner);
            });

            let body = self.lower_expr(ast.body(), &fn_decl.generic_params);
            self.package_bodies
                .fn_exprs
                .insert((self.module_id, fn_idx), body.inner);
            self.tckh
                .unify(
                    fn_decl.return_ty.inner,
                    body.tid,
                    self.tckh.tenv.get_filespan(&body.tid),
                )
                .unwrap_or_else(|err| self.diagnostics.push(err));
        }
    }

    fn handle_struct_decl(&mut self, struct_idx: Idx<StructDecl>) {
        let struct_decl = &self
            .item_tree(
                self.package_id
                    .unwrap_or_else(|| ice("tried lowering body without active item tree")),
            )
            .structs[struct_idx];

        struct_decl.fields.iter().for_each(|field| {
            if let TypeKind::Concrete(ConcreteKind::Path(path)) =
                &self.tckh.tenv.get(&field.ty).inner.inner
            {
                let span = self.tckh.tenv.get_span(&field.ty);
                let path: Path = path.clone().into();
                let path = path.at(span);
                self.resolve_path(&path)
                    .map(|(package_id, item_id)| match item_id.idx {
                        ItemTreeIdx::Enum(enum_idx) => {
                            let eenum = &self.item_tree(package_id).enums[enum_idx];
                            tracing::error!("this was left unhandled.. stupidly");
                        }
                        ItemTreeIdx::Struct(struct_idx) => {
                            let strukt = &self.item_tree(package_id).structs[struct_idx];
                            let num_generics_expected = strukt.generic_params.types.len();
                            let num_generis_got = path.generic_args.len();
                            if num_generics_expected != num_generis_got {
                                self.diagnostics.push(
                                    LowerError::MissingGenericArguments {
                                        got_names: path
                                            .generic_args
                                            .iter()
                                            .map(|tid| self.tckh.tenv.fmt_tid(tid))
                                            .collect(),
                                        got_names_file_span: path.span.in_file(self.file_id),
                                        expected_names: strukt
                                            .generic_params
                                            .types
                                            .values()
                                            .map(|generic| {
                                                self.interner.resolve(&generic).to_string()
                                            })
                                            .collect(),
                                        expected_names_file_span: strukt
                                            .generic_params
                                            .span
                                            .in_file(self.file_id(package_id, item_id.mod_id)),
                                    }
                                    .to_diagnostic(),
                                );
                            }
                        }
                        _ => {}
                    });
            }
        });
    }

    // fn add_trait(&mut self, trait_idx: Idx<TraitDecl>) {
    //     let item_tree = self.item_tree(self.package_id.unwrap());
    //     let trt = &item_tree.traits[trait_idx];
    //     let signatures = trt
    //         .methods
    //         .iter()
    //         .map(|method_idx| {
    //             let method = &item_tree.functions[*method_idx];
    //             FnSignature::new(
    //                 method.params.iter().map(|param| param.ty.inner),
    //                 method.return_ty.inner,
    //             )
    //         })
    //         .collect();
    //     self.trait_map
    //         .insert((self.package_id.unwrap(), trait_idx), trid);
    // }

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
        let n = node.unwrap_or_else(|| ice("missing node that should always be emitted"));
        if n.is_poisoned() {
            poison_function(self, n)
        } else {
            normal_function(self, n)
        }
    }

    pub fn lower_optional_node<N, T, P, F>(
        &mut self,
        node: Option<N>,
        poison_function: P,
        normal_function: F,
    ) -> T
    where
        N: AstNode,
        P: FnOnce(&mut Self) -> T,
        F: FnOnce(&mut Self, N) -> T,
    {
        match node {
            Some(n) => {
                if n.is_poisoned() {
                    poison_function(self)
                } else {
                    normal_function(self, n)
                }
            }
            None => poison_function(self),
        }
    }

    pub(crate) fn lower_name(&mut self, name: Option<ast::Name>) -> Spanned<Word> {
        self.lower_node(
            name,
            |this, name| {
                this.interner
                    .get_or_intern_static(POISONED_NAME)
                    .at(name.range().to_span())
            },
            |_, name| {
                let name = name
                    .ident()
                    .unwrap_or_else(|| ice("name parsed without identifier token"));
                let key = name.text_key().unwrap_or_else(|| ice("parsed empty name"));
                key.at(name.text_range().to_span())
            },
        )
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
                let segments = path
                    .segments()
                    .map(|segment| {
                        segment
                            .text_key()
                            .unwrap_or_else(|| ice("text key contained no text"))
                    })
                    .collect();
                let generic_args = path
                    .generic_arg_list()
                    .map(|arg_list| {
                        arg_list
                            .args()
                            .map(|arg| this.lower_type(Some(arg), generic_params).inner)
                            .collect()
                    })
                    .unwrap_or(vec![]);
                Path::new(segments, generic_args).at(path.range().to_span())
            },
        )
    }

    fn lower_op(&mut self, op: Option<&SyntaxToken>) -> Spanned<Op> {
        use flux_syntax::SyntaxKind::*;
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
}
