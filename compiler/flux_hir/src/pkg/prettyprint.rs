use std::{collections::HashMap, convert::identity};

use ascii_tree::{write_tree, Tree};
use colored::Colorize;
use cstree::interning::TokenKey;
use flux_span::Interner;
use flux_typesystem::{ConcreteKind, TEnv, TypeId, Typed};
use la_arena::Idx;
use pretty::RcDoc;

use crate::{
    hir::{
        ApplyDecl, AssociatedTypeDecl, AssociatedTypeDefinition, BinOp, EnumDecl, EnumDeclVariant,
        EnumDeclVariantList, FnDecl, GenericParams, If, ModDecl, Param, ParamList, Path,
        StructDecl, StructFieldDecl, StructFieldDeclList, TraitDecl, TypeBound, TypeBoundList,
        UseDecl, Visibility,
    },
    item_tree::ItemTree,
    module::{ModuleData, ModuleId, ModuleTree},
    Expr, ExprIdx, PackageBodies,
};

use super::PackageDefs;

const INDENT_SIZE: isize = 2;

impl PackageDefs {
    pub(crate) fn to_doc<'a>(
        &'a self,
        interner: &'static Interner,
        bodies: &'a PackageBodies,
        tenv: &'a TEnv,
        with_bodies: bool,
    ) -> RcDoc {
        RcDoc::text("+-------------+\n| Module Tree |\n+-------------+\n")
            .append(self.module_tree.to_doc(interner))
            .append(RcDoc::text(
                "+-------------+\n|  Item Tree  |\n+-------------+\n",
            ))
            .append(
                self.item_tree
                    .to_doc(&self.module_tree, interner, bodies, tenv, with_bodies),
            )
    }

    pub(crate) fn to_pretty(
        &self,
        width: usize,
        bodies: &PackageBodies,
        tenv: &TEnv,
        interner: &'static Interner,
        with_bodies: bool,
    ) -> String {
        let mut w = Vec::new();
        self.to_doc(interner, bodies, tenv, with_bodies)
            .render(width, &mut w)
            .unwrap();
        String::from_utf8(w).unwrap()
    }
}

impl ItemTree {
    pub fn to_doc<'a>(
        &'a self,
        module_tree: &'a ModuleTree,
        interner: &'static Interner,
        bodies: &'a PackageBodies,
        tenv: &'a TEnv,
        with_bodies: bool,
    ) -> RcDoc {
        let (module_id, root) = module_tree
            .get()
            .iter()
            .find(|(module_id, module)| {
                module.parent.is_none() && *module_id != ModuleTree::PRELUDE_ID
            })
            .expect("missing root module");
        let name = interner.get_or_intern_static("main");
        root.to_doc(
            &[name],
            &module_id,
            self,
            module_tree,
            interner,
            bodies,
            tenv,
            with_bodies,
        )
    }
}

impl Visibility {
    pub fn to_doc(&self) -> RcDoc {
        match self {
            Self::Private => RcDoc::nil(),
            Self::Public => RcDoc::text("pub".red().to_string()).append(RcDoc::space()),
        }
    }
}

impl FnDecl {
    pub fn to_doc<'a>(
        &'a self,
        interner: &'static Interner,
        bodies: &'a PackageBodies,
        tenv: &'a TEnv,
        id: Option<(ModuleId, Idx<FnDecl>)>,
    ) -> RcDoc {
        let opt_body = if let Some(id) = id {
            RcDoc::space()
                .append(RcDoc::text("{"))
                .append(RcDoc::line())
                .append(bodies.fn_exprs[&id].to_doc(interner, bodies, tenv))
                .nest(INDENT_SIZE)
                .append(RcDoc::line())
                .append(RcDoc::text("}"))
        } else {
            RcDoc::text(";".black().to_string())
        };
        self.visibility
            .to_doc()
            .append(RcDoc::text("fn".red().to_string()))
            .append(RcDoc::space())
            .append(RcDoc::text(
                interner.resolve(&self.name).green().to_string(),
            ))
            .append(self.generic_params.params_to_doc(interner))
            .append(self.params.to_doc(interner, tenv))
            .append(RcDoc::space())
            .append(RcDoc::text("→".blue().to_string()))
            .append(RcDoc::space())
            .append(type_id_to_doc(&self.return_ty, interner, tenv))
            .append(self.generic_params.where_clause_to_doc(interner))
            .append(opt_body)
    }
}

impl GenericParams {
    pub fn params_to_doc(&self, interner: &'static Interner) -> RcDoc {
        if self.types.is_empty() {
            RcDoc::nil()
        } else {
            let names = self
                .types
                .iter()
                .map(|(_, name)| RcDoc::text(interner.resolve(&name).yellow().to_string()));
            RcDoc::text("<".black().to_string())
                .append(RcDoc::intersperse(names, RcDoc::text(", ")))
                .append(">".black().to_string())
        }
    }

    pub fn where_clause_to_doc(&self, interner: &'static Interner) -> RcDoc {
        if self.where_predicates.is_empty() {
            RcDoc::nil()
        } else {
            let mut bound_lists = HashMap::new();
            self.where_predicates.iter().for_each(|p| {
                let text = p.bound.to_string(interner);
                bound_lists.entry(p.name).or_insert(vec![]).push(text);
            });
            let predicates = RcDoc::intersperse(
                bound_lists.iter().map(|(name, bounds)| {
                    RcDoc::text(interner.resolve(name).yellow().to_string())
                        .append(RcDoc::space())
                        .append(RcDoc::text("is".red().to_string()))
                        .append(RcDoc::space())
                        .append(RcDoc::intersperse(
                            bounds
                                .iter()
                                .map(|bound| RcDoc::text(bound.yellow().to_string())),
                            RcDoc::space()
                                .append(RcDoc::text("+".black().to_string()))
                                .append(RcDoc::space()),
                        ))
                }),
                RcDoc::text(",".black().to_string()).append(RcDoc::space()),
            );
            RcDoc::text("where".red().to_string())
                .append(RcDoc::space())
                .append(predicates)
        }
    }
}

impl ParamList {
    pub fn to_doc<'a>(&'a self, interner: &'static Interner, tenv: &'a TEnv) -> RcDoc {
        let params = if self.is_empty() {
            RcDoc::nil()
        } else {
            RcDoc::intersperse(
                self.iter().map(|param| param.to_doc(interner, tenv)),
                RcDoc::text(", "),
            )
        };

        RcDoc::text("(".bright_blue().to_string())
            .append(params)
            .append(RcDoc::text(")".bright_blue().to_string()))
    }
}

impl Param {
    pub fn to_doc<'a>(&'a self, interner: &'static Interner, tenv: &'a TEnv) -> RcDoc {
        RcDoc::text(interner.resolve(&self.name).blue().to_string())
            .append(RcDoc::space())
            .append(type_id_to_doc(&self.ty, interner, tenv))
    }
}

impl TraitDecl {
    pub(crate) fn to_doc<'a>(
        &'a self,
        interner: &'static Interner,
        bodies: &'a PackageBodies,
        tenv: &'a TEnv,
        item_tree: &'a ItemTree,
    ) -> RcDoc {
        let assoc_type_decls = RcDoc::intersperse(
            self.assoc_type_decls
                .iter()
                .map(|assoc_type_decl| assoc_type_decl.to_doc(interner, tenv)),
            RcDoc::line(),
        );
        let methods = RcDoc::intersperse(
            self.methods.iter().map(|method_idx| {
                let method = &item_tree.functions[*method_idx];
                method.to_doc(interner, bodies, tenv, None)
            }),
            RcDoc::line(),
        );
        self.visibility
            .to_doc()
            .append(RcDoc::text("trait".red().to_string()))
            .append(RcDoc::space())
            .append(RcDoc::text(
                interner.resolve(&self.name).yellow().to_string(),
            ))
            .append(self.generic_params.params_to_doc(interner))
            .append(self.generic_params.where_clause_to_doc(interner))
            .append(RcDoc::space())
            .append(RcDoc::text("{".black().to_string()))
            .append(RcDoc::line())
            .append(assoc_type_decls)
            .append(if self.assoc_type_decls.is_empty() {
                RcDoc::nil()
            } else {
                RcDoc::line()
            })
            .append(methods)
            .nest(INDENT_SIZE)
            .append(RcDoc::line())
            .append(RcDoc::text("}".black().to_string()))
    }
}

impl AssociatedTypeDecl {
    pub fn to_doc<'a>(&'a self, interner: &'static Interner, tenv: &'a TEnv) -> RcDoc {
        RcDoc::text("type".red().to_string())
            .append(RcDoc::space())
            .append(RcDoc::text(
                interner.resolve(&self.name).yellow().to_string(),
            ))
            .append(self.type_bound_list.to_doc(interner, tenv))
            .append(RcDoc::text(";".black().to_string()))
    }
}

impl TypeBoundList {
    pub fn to_doc<'a>(&'a self, interner: &'static Interner, tenv: &'a TEnv) -> RcDoc {
        if self.as_slice().is_empty() {
            RcDoc::nil()
        } else {
            let type_bounds = RcDoc::intersperse(
                self.iter()
                    .map(|type_bound| type_bound.to_doc(interner, tenv)),
                RcDoc::text("+").append(RcDoc::space()),
            );
            RcDoc::text(":").append(RcDoc::space()).append(type_bounds)
        }
    }
}

impl TypeBound {
    pub fn to_doc<'a>(&'a self, interner: &'static Interner, tenv: &'a TEnv) -> RcDoc {
        self.path().to_doc(interner, tenv)
    }
}

impl Path {
    pub fn to_doc<'a>(&'a self, interner: &'static Interner, tenv: &'a TEnv) -> RcDoc {
        let segments = RcDoc::intersperse(
            self.segments
                .iter()
                .map(|segment| RcDoc::text(interner.resolve(segment).yellow().to_string())),
            RcDoc::text("::".black().to_string()),
        );
        let generics = if self.generic_args.is_empty() {
            RcDoc::nil()
        } else {
            let generics = RcDoc::intersperse(
                self.generic_args
                    .iter()
                    .map(|tid| type_id_to_doc(tid, interner, tenv)),
                RcDoc::text(",").append(RcDoc::space()),
            );
            RcDoc::text("<".black().to_string())
                .append(generics)
                .append(RcDoc::text(">".black().to_string()))
        };
        segments.append(generics)
    }
}

impl ApplyDecl {
    pub(crate) fn to_doc<'a>(
        &'a self,
        interner: &'static Interner,
        bodies: &'a PackageBodies,
        tenv: &'a TEnv,
        item_tree: &'a ItemTree,
        module_id: ModuleId,
    ) -> RcDoc {
        let assoc_types = RcDoc::intersperse(
            self.assoc_types
                .iter()
                .map(|assoc_type| assoc_type.to_doc(interner, tenv)),
            RcDoc::line(),
        );
        let methods = RcDoc::intersperse(
            self.methods.iter().map(|method_idx| {
                item_tree.functions[*method_idx].to_doc(
                    interner,
                    bodies,
                    tenv,
                    Some((module_id, *method_idx)),
                )
            }),
            RcDoc::line(),
        );
        RcDoc::text("apply".red().to_string())
            .append(self.generic_params.params_to_doc(interner))
            .append(RcDoc::space())
            .append(
                self.trt
                    .as_ref()
                    .map(|trt| trt.to_doc(interner, tenv).append(RcDoc::space())),
            )
            .append(RcDoc::text("to".red().to_string()))
            .append(RcDoc::space())
            .append(type_id_to_doc(&self.to_ty, interner, tenv))
            .append(RcDoc::space())
            .append(RcDoc::text("{".black().to_string()))
            .append(RcDoc::line())
            .append(assoc_types)
            .append(RcDoc::line())
            .append(methods)
            .nest(INDENT_SIZE)
            .append(RcDoc::line())
            .append(RcDoc::text("}".black().to_string()))
    }
}

impl AssociatedTypeDefinition {
    pub fn to_doc<'a>(&'a self, interner: &'static Interner, tenv: &'a TEnv) -> RcDoc {
        RcDoc::text("type".red().to_string())
            .append(RcDoc::space())
            .append(RcDoc::text(
                interner.resolve(&self.name).yellow().to_string(),
            ))
            .append(RcDoc::space())
            .append(RcDoc::text("="))
            .append(RcDoc::space())
            .append(type_id_to_doc(&self.ty, interner, tenv))
            .append(RcDoc::text(";".black().to_string()))
    }
}

impl ModDecl {
    pub fn to_doc<'a>(&'a self, interner: &'static Interner) -> RcDoc {
        self.visibility
            .to_doc()
            .append(RcDoc::text("mod".red().to_string()))
            .append(RcDoc::space())
            .append(RcDoc::text(
                interner.resolve(&self.name).yellow().to_string(),
            ))
            .append(RcDoc::text(";".black().to_string()))
    }
}

impl StructDecl {
    pub fn to_doc<'a>(&'a self, interner: &'static Interner, tenv: &'a TEnv) -> RcDoc {
        self.visibility
            .to_doc()
            .append(RcDoc::text("struct".red().to_string()))
            .append(RcDoc::space())
            .append(RcDoc::text(
                interner.resolve(&self.name).yellow().to_string(),
            ))
            .append(self.generic_params.params_to_doc(interner))
            .append(RcDoc::space())
            .append(self.generic_params.where_clause_to_doc(interner))
            .append(RcDoc::space())
            .append(RcDoc::text("{"))
            .append(RcDoc::line())
            .append(self.fields.to_doc(interner, tenv))
            .nest(INDENT_SIZE)
            .append(RcDoc::line())
            .append(RcDoc::text("}"))
    }
}

impl StructFieldDeclList {
    pub fn to_doc<'a>(&'a self, interner: &'static Interner, tenv: &'a TEnv) -> RcDoc {
        RcDoc::intersperse(
            self.iter()
                .map(|field_decl| field_decl.to_doc(interner, tenv)),
            RcDoc::text(",".black().to_string()).append(RcDoc::line()),
        )
    }
}

impl StructFieldDecl {
    pub fn to_doc<'a>(&'a self, interner: &'static Interner, tenv: &'a TEnv) -> RcDoc {
        RcDoc::text(interner.resolve(&self.name).blue().to_string())
            .append(RcDoc::space())
            .append(type_id_to_doc(&self.ty, interner, tenv))
    }
}

impl EnumDecl {
    pub fn to_doc<'a>(&'a self, interner: &'static Interner, tenv: &'a TEnv) -> RcDoc {
        self.visibility
            .to_doc()
            .append(RcDoc::text("enum".red().to_string()))
            .append(RcDoc::space())
            .append(RcDoc::text(
                interner.resolve(&self.name).yellow().to_string(),
            ))
            .append(self.generic_params.params_to_doc(interner))
            .append(RcDoc::space())
            .append(self.generic_params.where_clause_to_doc(interner))
            .append(RcDoc::space())
            .append(RcDoc::text("{"))
            .append(RcDoc::line())
            .append(self.variants.to_doc(interner, tenv))
            .nest(INDENT_SIZE)
            .append(RcDoc::line())
            .append(RcDoc::text("}"))
    }
}

impl EnumDeclVariantList {
    pub fn to_doc<'a>(&'a self, interner: &'static Interner, tenv: &'a TEnv) -> RcDoc {
        RcDoc::intersperse(
            self.iter().map(|variant| variant.to_doc(interner, tenv)),
            RcDoc::text(",".black().to_string()).append(RcDoc::line()),
        )
    }
}

impl EnumDeclVariant {
    pub fn to_doc<'a>(&'a self, interner: &'static Interner, tenv: &'a TEnv) -> RcDoc {
        RcDoc::text(interner.resolve(&self.name).blue().to_string()).append(
            self.ty.as_ref().map_or(RcDoc::nil(), |tid| {
                RcDoc::space()
                    .append(RcDoc::text("→".black().to_string()))
                    .append(RcDoc::space())
                    .append(type_id_to_doc(tid, interner, tenv))
            }),
        )
    }
}

impl UseDecl {
    pub fn to_doc<'a>(&'a self, interner: &'static Interner, tenv: &'a TEnv) -> RcDoc {
        RcDoc::text("use".red().to_string())
            .append(RcDoc::space())
            .append(self.path.to_doc(interner, tenv))
            .append(self.alias.as_ref().map_or(RcDoc::nil(), |alias| {
                RcDoc::space()
                    .append(RcDoc::text("as".red().to_string()))
                    .append(RcDoc::space())
                    .append(RcDoc::text(interner.resolve(&alias).blue().to_string()))
            }))
            .append(RcDoc::text(";".black().to_string()))
    }
}

impl ExprIdx {
    pub fn to_doc<'a>(
        &'a self,
        interner: &'static Interner,
        bodies: &'a PackageBodies,
        tenv: &'a TEnv,
    ) -> RcDoc {
        bodies.exprs[self.idx()].to_doc(interner, bodies, tenv)
    }
}

impl Expr {
    pub fn to_doc<'a>(
        &'a self,
        interner: &'static Interner,
        bodies: &'a PackageBodies,
        tenv: &'a TEnv,
    ) -> RcDoc {
        match self {
            Expr::Address(address_expr) => {
                RcDoc::text("&").append(address_expr.to_doc(interner, bodies, tenv))
            }
            Expr::BinOp(bin_op_expr) => bin_op_expr.to_doc(interner, bodies, tenv),
            Expr::Int(int_expr) => RcDoc::text(int_expr.to_string().purple().to_string()),
            Expr::Tuple(vals) => RcDoc::text("(")
                .append(RcDoc::intersperse(
                    vals.iter().map(|idx| idx.to_doc(interner, bodies, tenv)),
                    RcDoc::text(", "),
                ))
                .append(RcDoc::text(")")),
            Expr::Path(path_expr) => path_expr.to_doc(interner, tenv),
            Expr::Struct(_) => todo!(),
            Expr::If(if_expr) => if_expr.to_doc(interner, bodies, tenv),
            Expr::Poisoned => todo!(),
        }
    }
}

impl BinOp {
    pub fn to_doc<'a>(
        &'a self,
        interner: &'static Interner,
        bodies: &'a PackageBodies,
        tenv: &'a TEnv,
    ) -> RcDoc {
        use crate::hir::Op::*;
        let op = match self.op.inner {
            Eq => "=",
            Add => "+",
            Sub => "-",
            Mul => "*",
            Div => "/",
            CmpAnd => "&&",
            CmpEq => "==",
            CmpGt => ">",
            CmpGte => ">=",
            CmpLt => "<",
            CmpLte => "<=",
            CmpNeq => "!=",
            CmpOr => "||",
        }
        .black()
        .to_string();
        self.lhs
            .to_doc(interner, bodies, tenv)
            .append(RcDoc::space())
            .append(RcDoc::text(op))
            .append(RcDoc::space().append(self.rhs.to_doc(interner, bodies, tenv)))
    }
}

impl If {
    pub fn to_doc<'a>(
        &'a self,
        interner: &'static Interner,
        bodies: &'a PackageBodies,
        tenv: &'a TEnv,
    ) -> RcDoc {
        RcDoc::text("if".red().to_string())
            .append(RcDoc::space())
            .append(self.condition().to_doc(interner, bodies, tenv))
            .append(RcDoc::space())
            .append(RcDoc::text("{"))
            .append(RcDoc::line())
            .append(self.then().to_doc(interner, bodies, tenv))
            .nest(INDENT_SIZE)
            .append(RcDoc::line())
            .append(RcDoc::text("}"))
            .append(self.else_ifs().map(|else_ifs| {
                RcDoc::intersperse(
                    else_ifs
                        .iter()
                        .array_chunks()
                        .map(|exprs: [&Typed<ExprIdx>; 2]| {
                            RcDoc::space()
                                .append(RcDoc::text("else".red().to_string()))
                                .append(RcDoc::space())
                                .append(RcDoc::text("if".red().to_string()))
                                .append(RcDoc::space())
                                .append(exprs[0].to_doc(interner, bodies, tenv))
                                .append(RcDoc::space())
                                .append(RcDoc::text("{"))
                                .append(RcDoc::line())
                                .append(exprs[1].to_doc(interner, bodies, tenv))
                                .nest(INDENT_SIZE)
                                .append(RcDoc::line())
                                .append(RcDoc::text("}"))
                        }),
                    RcDoc::nil(),
                )
            }))
            .append(self.else_block().map_or(RcDoc::nil(), |else_block| {
                RcDoc::space()
                    .append(RcDoc::text("else".red().to_string()))
                    .append(RcDoc::space())
                    .append(RcDoc::text("{"))
                    .append(RcDoc::line())
                    .append(else_block.to_doc(interner, bodies, tenv))
                    .nest(INDENT_SIZE)
                    .append(RcDoc::line())
                    .append(RcDoc::text("}"))
            }))
    }
}

fn type_id_to_doc<'a>(tid: &TypeId, interner: &'static Interner, tenv: &'a TEnv) -> RcDoc<'a> {
    use flux_typesystem::TypeKind::*;
    match &tenv.get(tid).inner.inner {
        ThisPath(this_path) => RcDoc::text("This".red().to_string())
            .append(RcDoc::text("::".black().to_string()))
            .append(RcDoc::intersperse(
                this_path
                    .segments
                    .iter()
                    .map(|segment| interner.resolve(segment).yellow().to_string()),
                RcDoc::text("::".black().to_string()),
            )),
        Concrete(concrete) => concrete_tkind_to_doc(&concrete, interner, tenv),
        Int(_) => todo!(),
        Float(_) => todo!(),
        Ref(_) => todo!(),
        Generic(generic) => generic.to_doc(interner),
        Never => todo!(),
        Unknown => todo!(),
    }
}

fn concrete_tkind_to_doc<'a>(
    concrete: &ConcreteKind,
    interner: &'static Interner,
    tenv: &'a TEnv,
) -> RcDoc<'a> {
    use ConcreteKind::*;
    match concrete {
        Array(tid, n) => RcDoc::text("[")
            .append(type_id_to_doc(tid, interner, tenv))
            .append(";".black().to_string())
            .append(RcDoc::space())
            .append(RcDoc::text(n.to_string()))
            .append("]"),
        Ptr(tid) => RcDoc::text("*").append(type_id_to_doc(tid, interner, tenv)),
        Path(path) => {
            let path_segments = RcDoc::intersperse(
                path.segments
                    .iter()
                    .map(|key| RcDoc::text(interner.resolve(key).yellow().to_string())),
                RcDoc::text("::".blue().to_string()),
            );
            let generics = if path.generic_args.is_empty() {
                RcDoc::nil()
            } else {
                RcDoc::text("<")
                    .append(RcDoc::intersperse(
                        path.generic_args
                            .iter()
                            .map(|tid| type_id_to_doc(tid, interner, tenv)),
                        RcDoc::text(", "),
                    ))
                    .append(RcDoc::text(">"))
            };
            path_segments.append(generics)
        }
        Tuple(types) => RcDoc::text("(".bright_blue().to_string())
            .append(RcDoc::intersperse(
                types.iter().map(|tid| type_id_to_doc(tid, interner, tenv)),
                RcDoc::text(", "),
            ))
            .append(RcDoc::text(")".bright_blue().to_string())),
    }
}

impl ModuleTree {
    pub fn to_doc(&self, interner: &'static Interner) -> RcDoc {
        RcDoc::text(self.to_string(interner))
    }

    pub fn to_string(&self, interner: &'static Interner) -> String {
        let (_, root) = self
            .get()
            .iter()
            .find(|(module_id, module)| {
                module.parent.is_none() && *module_id != ModuleTree::PRELUDE_ID
            })
            .expect("missing root module");

        let tree = root.to_tree(self, &interner.get_or_intern_static("main"), interner);

        let mut s = String::new();
        write_tree(&mut s, &tree).unwrap();
        s
    }
}

impl ModuleData {
    fn to_tree(
        &self,
        module_tree: &ModuleTree,
        name: &TokenKey,
        interner: &'static Interner,
    ) -> Tree {
        let name = interner.resolve(name).to_string();
        if self.children.is_empty() {
            Tree::Leaf(vec![name])
        } else {
            Tree::Node(
                name,
                self.children
                    .iter()
                    .map(|(name, mod_id)| module_tree[*mod_id].to_tree(module_tree, name, interner))
                    .collect(),
            )
        }
    }

    pub(crate) fn to_doc<'a>(
        &'a self,
        path: &[TokenKey],
        module_id: &ModuleId,
        item_tree: &'a ItemTree,
        module_tree: &'a ModuleTree,
        interner: &'static Interner,
        bodies: &'a PackageBodies,
        tenv: &'a TEnv,
        with_bodies: bool,
    ) -> RcDoc {
        let name = RcDoc::intersperse(
            path.iter()
                .map(|key| interner.resolve(key).underline().black().to_string()),
            RcDoc::text("::".underline().black().to_string()),
        );
        let items = RcDoc::intersperse(
            item_tree
                .top_level
                .iter()
                .map(|item_id| {
                    if item_id.mod_id == *module_id {
                        Some(match item_id.idx {
                            crate::item::ItemTreeIdx::Apply(apply_decl) => item_tree.applies
                                [apply_decl]
                                .to_doc(interner, bodies, tenv, item_tree, *module_id),
                            crate::item::ItemTreeIdx::BuiltinType(_) => RcDoc::text("builtin"),
                            crate::item::ItemTreeIdx::Enum(enum_decl) => {
                                item_tree.enums[enum_decl].to_doc(interner, tenv)
                            }
                            crate::item::ItemTreeIdx::Function(fn_decl) => {
                                item_tree.functions[fn_decl].to_doc(
                                    interner,
                                    bodies,
                                    tenv,
                                    if with_bodies {
                                        Some((*module_id, fn_decl))
                                    } else {
                                        None
                                    },
                                )
                            }
                            crate::item::ItemTreeIdx::Module(mod_decl) => {
                                item_tree.mods[mod_decl].to_doc(interner)
                            }
                            crate::item::ItemTreeIdx::Struct(struct_decl) => {
                                item_tree.structs[struct_decl].to_doc(interner, tenv)
                            }
                            crate::item::ItemTreeIdx::Trait(trait_decl) => item_tree.traits
                                [trait_decl]
                                .to_doc(interner, bodies, tenv, item_tree),
                            crate::item::ItemTreeIdx::Use(use_decl) => {
                                item_tree.uses[use_decl].to_doc(interner, tenv)
                            }
                        })
                    } else {
                        None
                    }
                })
                .filter_map(identity),
            RcDoc::hardline(),
        );
        let children = RcDoc::intersperse(
            self.children.iter().map(|(name, mod_id)| {
                let path = &[path, &[*name]].concat();
                module_tree[*mod_id].to_doc(
                    path,
                    mod_id,
                    item_tree,
                    module_tree,
                    interner,
                    bodies,
                    tenv,
                    with_bodies,
                )
            }),
            RcDoc::hardline(),
        );
        RcDoc::text("Module: ".black().to_string())
            .append(name)
            .append(RcDoc::hardline())
            .append(items)
            .append(RcDoc::hardline())
            .append(children)
    }
}
