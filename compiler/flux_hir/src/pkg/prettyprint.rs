use std::{collections::HashMap, convert::identity};

use ascii_tree::{write_tree, Tree};
use colored::Colorize;
use cstree::interning::TokenKey;
use flux_span::Interner;
use flux_typesystem::{ConcreteKind, TEnv, TypeId};
use pretty::RcDoc;

use crate::{
    hir::{
        ApplyDecl, AssociatedTypeDecl, AssociatedTypeDefinition, EnumDecl, EnumDeclVariant,
        EnumDeclVariantList, FnDecl, GenericParams, ModDecl, Param, ParamList, Path, StructDecl,
        StructFieldDecl, StructFieldDeclList, TraitDecl, TypeBound, TypeBoundList, UseDecl,
        Visibility,
    },
    item_tree::ItemTree,
    module::{ModuleData, ModuleId, ModuleTree},
};

use super::Package;

const INDENT_SIZE: isize = 2;

impl Package {
    pub fn to_doc<'a>(&'a self, interner: &'static Interner, tenv: &'a TEnv) -> RcDoc {
        RcDoc::text("+-------------+\n| Module Tree |\n+-------------+\n")
            .append(self.module_tree.to_doc(interner))
            .append(RcDoc::text(
                "+-------------+\n|  Item Tree  |\n+-------------+\n",
            ))
            .append(self.item_tree.to_doc(&self.module_tree, interner, tenv))
    }

    pub fn to_pretty(&self, width: usize, interner: &'static Interner, tenv: &TEnv) -> String {
        let mut w = Vec::new();
        self.to_doc(interner, tenv).render(width, &mut w).unwrap();
        String::from_utf8(w).unwrap()
    }
}

impl ItemTree {
    pub fn to_doc<'a>(
        &'a self,
        module_tree: &'a ModuleTree,
        interner: &'static Interner,
        tenv: &'a TEnv,
    ) -> RcDoc {
        let (module_id, root) = module_tree
            .get()
            .iter()
            .find(|(module_id, module)| {
                module.parent.is_none() && *module_id != ModuleTree::PRELUDE_ID
            })
            .expect("missing root module");
        let name = interner.get_or_intern_static("main");
        root.to_doc(&[name], &module_id, self, module_tree, interner, tenv)
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
    pub fn to_doc<'a>(&'a self, interner: &'static Interner, tenv: &'a TEnv) -> RcDoc {
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
            .append(RcDoc::text(";".black().to_string()))
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
                method.to_doc(interner, tenv)
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
            .append(RcDoc::line())
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
        tenv: &'a TEnv,
        item_tree: &'a ItemTree,
    ) -> RcDoc {
        let assoc_types = RcDoc::intersperse(
            self.assoc_types
                .iter()
                .map(|assoc_type| assoc_type.to_doc(interner, tenv)),
            RcDoc::line(),
        );
        let methods = RcDoc::intersperse(
            self.methods
                .iter()
                .map(|method_idx| item_tree.functions[*method_idx].to_doc(interner, tenv)),
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

fn type_id_to_doc<'a>(tid: &TypeId, interner: &'static Interner, tenv: &'a TEnv) -> RcDoc<'a> {
    use flux_typesystem::TypeKind::*;
    match tenv.get(tid) {
        ThisPath(this_path) => RcDoc::text("This".red().to_string())
            .append(RcDoc::text("::".black().to_string()))
            .append(RcDoc::intersperse(
                this_path
                    .iter()
                    .map(|segment| interner.resolve(segment).yellow().to_string()),
                RcDoc::text("::".black().to_string()),
            )),
        Concrete(concrete) => concrete_tkind_to_doc(concrete, interner, tenv),
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
        Path(segments, generic_args) => {
            let path = RcDoc::intersperse(
                segments
                    .iter()
                    .map(|key| RcDoc::text(interner.resolve(key).yellow().to_string())),
                RcDoc::text("::".blue().to_string()),
            );
            let generics = if generic_args.is_empty() {
                RcDoc::nil()
            } else {
                RcDoc::text("<")
                    .append(RcDoc::intersperse(
                        generic_args
                            .iter()
                            .map(|tid| type_id_to_doc(tid, interner, tenv)),
                        RcDoc::text(", "),
                    ))
                    .append(RcDoc::text(">"))
            };
            path.append(generics)
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

    pub fn to_doc<'a>(
        &'a self,
        path: &[TokenKey],
        module_id: &ModuleId,
        item_tree: &'a ItemTree,
        module_tree: &'a ModuleTree,
        interner: &'static Interner,
        tenv: &'a TEnv,
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
                            crate::item::ItemTreeIdx::Apply(apply_decl) => {
                                item_tree.applies[apply_decl].to_doc(interner, tenv, item_tree)
                            }
                            crate::item::ItemTreeIdx::Enum(enum_decl) => {
                                item_tree.enums[enum_decl].to_doc(interner, tenv)
                            }
                            crate::item::ItemTreeIdx::Function(fn_decl) => {
                                item_tree.functions[fn_decl].to_doc(interner, tenv)
                            }
                            crate::item::ItemTreeIdx::Module(mod_decl) => {
                                item_tree.mods[mod_decl].to_doc(interner)
                            }
                            crate::item::ItemTreeIdx::Struct(struct_decl) => {
                                item_tree.structs[struct_decl].to_doc(interner, tenv)
                            }
                            crate::item::ItemTreeIdx::Trait(trait_decl) => {
                                item_tree.traits[trait_decl].to_doc(interner, tenv, item_tree)
                            }
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
                module_tree[*mod_id].to_doc(path, mod_id, item_tree, module_tree, interner, tenv)
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
