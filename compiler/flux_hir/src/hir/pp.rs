use lasso::ThreadedRodeo;
use pretty::{DocAllocator, DocBuilder};

use crate::{
    body::LoweredBodies,
    item_tree::{ItemTree, ModItem},
    DefMap, ModuleDefId, ModuleId,
};

use super::*;

impl DefMap {
    pub fn pretty<'b, D, A>(
        &'b self,
        allocator: &'b D,
        string_interner: &'static ThreadedRodeo,
        bodies: &'b LoweredBodies,
        global_item_tree: &'b ItemTree,
    ) -> DocBuilder<'b, D, A>
    where
        D: DocAllocator<'b, A>,
        D::Doc: Clone,
        A: Clone,
    {
        allocator.nil()
        // allocator.intersperse(
        //     self.items.iter().map(|(module_id, items)| {
        //         let items = allocator.intersperse(
        //             items.iter().map(|item| {
        //                 item.pretty(
        //                     allocator,
        //                     string_interner,
        //                     bodies,
        //                     module_id,
        //                     global_item_tree,
        //                 )
        //             }),
        //             allocator.hardline(),
        //         );
        //         allocator.text("// ")
        //             + allocator.text(string_interner.resolve(&self[module_id].file_id.0))
        //             + allocator.hardline()
        //             + allocator.hardline()
        //             + items
        //     }),
        //     "\n",
        // )
    }
}

impl ModItem {
    pub fn pretty<'b, D, A>(
        &'b self,
        allocator: &'b D,
        string_interner: &'static ThreadedRodeo,
        bodies: &'b LoweredBodies,
        module_id: ModuleId,
        item_tree: &'b ItemTree,
    ) -> DocBuilder<'b, D, A>
    where
        D: DocAllocator<'b, A>,
        D::Doc: Clone,
        A: Clone,
    {
        match self {
            ModItem::Apply(apply) => item_tree[*apply].pretty(
                allocator,
                string_interner,
                bodies,
                &item_tree.functions,
                module_id,
            ),
            ModItem::Function(f_idx) => {
                let body = bodies
                    .indices
                    .get(&(module_id, ModuleDefId::FunctionId(f_idx.index)))
                    .unwrap();
                item_tree[*f_idx].pretty(allocator, string_interner, bodies)
                    + allocator.space()
                    + body.pretty(allocator, string_interner, bodies)
            }
            ModItem::Mod(m_idx) => item_tree[*m_idx].pretty(allocator, string_interner),
            ModItem::Struct(s_idx) => item_tree[*s_idx].pretty(allocator, string_interner, bodies),
            ModItem::Trait(t_idx) => {
                item_tree[*t_idx].pretty(allocator, string_interner, bodies, &item_tree.functions)
            }
            ModItem::Use(u_idx) => item_tree[*u_idx].pretty(allocator, string_interner, bodies),
            _ => allocator.nil(),
        }
    }
}

impl ItemTree {
    pub fn pretty<'b, D, A>(
        &'b self,
        allocator: &'b D,
        string_interner: &'static ThreadedRodeo,
        bodies: &'b LoweredBodies,
        module_id: ModuleId,
    ) -> DocBuilder<'b, D, A>
    where
        D: DocAllocator<'b, A>,
        D::Doc: Clone,
        A: Clone,
    {
        allocator.intersperse(
            self.top_level.iter().map(|mod_item| match mod_item {
                ModItem::Apply(apply) => self[*apply].pretty(
                    allocator,
                    string_interner,
                    bodies,
                    &self.functions,
                    module_id,
                ),
                ModItem::Function(f_idx) => {
                    let body = bodies
                        .indices
                        .get(&(module_id, ModuleDefId::FunctionId(f_idx.index)))
                        .unwrap();
                    self[*f_idx].pretty(allocator, string_interner, bodies)
                        + allocator.space()
                        + body.pretty(allocator, string_interner, bodies)
                }
                ModItem::Mod(m_idx) => self[*m_idx].pretty(allocator, string_interner),
                ModItem::Struct(s_idx) => self[*s_idx].pretty(allocator, string_interner, bodies),
                ModItem::Trait(t_idx) => {
                    self[*t_idx].pretty(allocator, string_interner, bodies, &self.functions)
                }
                ModItem::Use(u_idx) => self[*u_idx].pretty(allocator, string_interner, bodies),
                _ => allocator.nil(),
            }),
            allocator.hardline(),
        )
    }
}

impl Apply {
    pub fn pretty<'b, D, A>(
        &'b self,
        allocator: &'b D,
        string_interner: &'static ThreadedRodeo,
        bodies: &'b LoweredBodies,
        functions: &'b Arena<Function>,
        module_id: ModuleId,
    ) -> DocBuilder<'b, D, A>
    where
        D: DocAllocator<'b, A>,
        D::Doc: Clone,
        A: Clone,
    {
        let assoc_types = allocator.intersperse(
            self.assoc_types.iter().map(|(name, ty)| {
                allocator.text("type")
                    + allocator.space()
                    + allocator.text(string_interner.resolve(name))
                    + allocator.space()
                    + allocator.text("=")
                    + allocator.space()
                    + ty.pretty(allocator, string_interner, bodies)
                    + allocator.text(";")
            }),
            allocator.hardline(),
        );
        let methods = allocator.intersperse(
            self.methods.iter().map(|method| {
                let body = bodies
                    .indices
                    .get(&(module_id, ModuleDefId::FunctionId(method.inner.clone())))
                    .unwrap();
                functions[method.inner].pretty(allocator, string_interner, bodies)
                    + allocator.space()
                    + body.pretty(allocator, string_interner, bodies)
            }),
            allocator.hardline(),
        );
        self.visibility.pretty(allocator)
            + allocator.text("apply")
            + self.generic_params.pretty(allocator, string_interner)
            + allocator.space()
            + match &self.trt {
                Some(trt) => trt.pretty(allocator, string_interner, bodies),
                None => allocator.nil(),
            }
            + allocator.space()
            + allocator.text("to")
            + allocator.space()
            + self.ty.pretty(allocator, string_interner, bodies)
            + self
                .generic_params
                .where_predicates
                .pretty(allocator, string_interner)
            + allocator.space()
            + allocator.text("{")
            + allocator.hardline()
            + assoc_types.indent(4)
            + allocator.hardline()
            + methods.indent(4)
            + allocator.hardline()
            + allocator.text("}")
            + allocator.hardline()
    }
}

impl Function {
    pub fn pretty<'b, D, A>(
        &'b self,
        allocator: &'b D,
        string_interner: &'static ThreadedRodeo,
        bodies: &'b LoweredBodies,
    ) -> DocBuilder<'b, D, A>
    where
        D: DocAllocator<'b, A>,
        D::Doc: Clone,
        A: Clone,
    {
        self.visibility
            .inner
            .pretty(allocator)
            .append(allocator.text("fn "))
            + allocator.text(string_interner.resolve(&self.name.inner))
            + self.generic_params.pretty(allocator, string_interner)
            + self.params.pretty(allocator, string_interner, bodies)
            + allocator.text(" -> ")
            + self.ret_ty.pretty(allocator, string_interner, bodies)
            + self
                .generic_params
                .where_predicates
                .pretty(allocator, string_interner)
    }
}

impl Mod {
    pub fn pretty<'b, D, A>(
        &'b self,
        allocator: &'b D,
        string_interner: &'static ThreadedRodeo,
    ) -> DocBuilder<'b, D, A>
    where
        D: DocAllocator<'b, A>,
        D::Doc: Clone,
        A: Clone,
    {
        self.visibility.pretty(allocator)
            + allocator.text("mod ")
            + string_interner.resolve(&self.name)
            + allocator.text(";")
    }
}

impl Struct {
    pub fn pretty<'b, D, A>(
        &'b self,
        allocator: &'b D,
        string_interner: &'static ThreadedRodeo,
        bodies: &'b LoweredBodies,
    ) -> DocBuilder<'b, D, A>
    where
        D: DocAllocator<'b, A>,
        D::Doc: Clone,
        A: Clone,
    {
        let fields = allocator.intersperse(
            self.fields
                .fields
                .iter()
                .map(|field| field.pretty(allocator, string_interner, bodies)),
            ",",
        );
        self.visibility.pretty(allocator)
            + allocator.text("struct")
            + allocator.space()
            + allocator.text(string_interner.resolve(&self.name))
            + self.generic_params.pretty(allocator, string_interner)
            + allocator.space()
            + self
                .generic_params
                .where_predicates
                .pretty(allocator, string_interner)
            + allocator.space()
            + allocator.text("{")
            + allocator.line()
            + fields
            + allocator.text("}")
            + allocator.line()
    }
}

impl Trait {
    pub fn pretty<'b, D, A>(
        &'b self,
        allocator: &'b D,
        string_interner: &'static ThreadedRodeo,
        bodies: &'b LoweredBodies,
        functions: &'b Arena<Function>,
    ) -> DocBuilder<'b, D, A>
    where
        D: DocAllocator<'b, A>,
        D::Doc: Clone,
        A: Clone,
    {
        let assoc_types = allocator.intersperse(
            self.assoc_types.iter().map(|(name, erstrictions)| {
                allocator.text("type")
                    + allocator.space()
                    + allocator.text(string_interner.resolve(name))
                    + allocator.text(";")
            }),
            allocator.hardline(),
        );
        let methods = allocator.intersperse(
            self.methods.iter().map(|method| {
                functions[method.inner].pretty(allocator, string_interner, bodies)
                    + allocator.text(";")
            }),
            allocator.hardline(),
        );
        self.visibility.pretty(allocator)
            + allocator.text("trait")
            + allocator.space()
            + allocator.text(string_interner.resolve(&self.name))
            + self.generic_params.pretty(allocator, string_interner)
            + allocator.space()
            + self
                .generic_params
                .where_predicates
                .pretty(allocator, string_interner)
            + allocator.space()
            + allocator.text("{")
            + allocator.line()
            + assoc_types.indent(4)
            + allocator.hardline()
            + methods.indent(4)
            + allocator.hardline()
            + allocator.text("}")
            + allocator.line()
    }
}

impl Use {
    pub fn pretty<'b, D, A>(
        &'b self,
        allocator: &'b D,
        string_interner: &'static ThreadedRodeo,
        bodies: &'b LoweredBodies,
    ) -> DocBuilder<'b, D, A>
    where
        D: DocAllocator<'b, A>,
        D::Doc: Clone,
        A: Clone,
    {
        self.visibility.pretty(allocator)
            + allocator.text("use")
            + allocator.space()
            + self.path.pretty(allocator, string_interner, bodies)
            + allocator.text(";")
    }
}

impl StructField {
    pub fn pretty<'b, D, A>(
        &'b self,
        allocator: &'b D,
        string_interner: &'static ThreadedRodeo,
        bodies: &'b LoweredBodies,
    ) -> DocBuilder<'b, D, A>
    where
        D: DocAllocator<'b, A>,
        D::Doc: Clone,
        A: Clone,
    {
        self.ty.pretty(allocator, string_interner, bodies)
            + allocator.space()
            + string_interner.resolve(&self.name)
    }
}

impl Visibility {
    pub fn pretty<'b, D, A>(&'b self, allocator: &'b D) -> DocBuilder<'b, D, A>
    where
        D: DocAllocator<'b, A>,
        D::Doc: Clone,
        A: Clone,
    {
        match self {
            Self::Public => allocator.text("pub") + allocator.space(),
            Self::Private => allocator.text(""),
        }
    }
}

impl Params {
    pub fn pretty<'b, D, A>(
        &'b self,
        allocator: &'b D,
        string_interner: &'static ThreadedRodeo,
        bodies: &'b LoweredBodies,
    ) -> DocBuilder<'b, D, A>
    where
        D: DocAllocator<'b, A>,
        D::Doc: Clone,
        A: Clone,
    {
        allocator.text("(")
            + allocator.intersperse(
                self.0
                    .iter()
                    .map(|param| param.pretty(allocator, string_interner, bodies)),
                ", ",
            )
            + allocator.text(")")
    }
}

impl Param {
    pub fn pretty<'b, D, A>(
        &'b self,
        allocator: &'b D,
        string_interner: &'static ThreadedRodeo,
        bodies: &'b LoweredBodies,
    ) -> DocBuilder<'b, D, A>
    where
        D: DocAllocator<'b, A>,
        D::Doc: Clone,
        A: Clone,
    {
        allocator.text(string_interner.resolve(&self.name))
            + allocator.text(" ")
            + self.ty.pretty(allocator, string_interner, bodies)
    }
}

impl GenericParams {
    pub fn pretty<'b, D, A>(
        &'b self,
        allocator: &'b D,
        string_interner: &'static ThreadedRodeo,
    ) -> DocBuilder<'b, D, A>
    where
        D: DocAllocator<'b, A>,
        D::Doc: Clone,
        A: Clone,
    {
        if self.types.is_empty() {
            return allocator.nil();
        }
        allocator.text("<")
            + allocator.intersperse(
                self.types
                    .iter()
                    .map(|(_, ty)| allocator.text(string_interner.resolve(ty))),
                ", ",
            )
            + allocator.text(">")
    }
}

impl WherePredicates {
    pub fn pretty<'b, D, A>(
        &'b self,
        allocator: &'b D,
        string_interner: &'static ThreadedRodeo,
    ) -> DocBuilder<'b, D, A>
    where
        D: DocAllocator<'b, A>,
        D::Doc: Clone,
        A: Clone,
    {
        if self.0.is_empty() {
            return allocator.nil();
        }

        allocator.text(" where ")
            + allocator.intersperse(
                self.0
                    .iter()
                    .map(|predicate| predicate.pretty(allocator, string_interner)),
                ", ",
            )
    }
}

impl WherePredicate {
    pub fn pretty<'b, D, A>(
        &'b self,
        allocator: &'b D,
        string_interner: &'static ThreadedRodeo,
    ) -> DocBuilder<'b, D, A>
    where
        D: DocAllocator<'b, A>,
        D::Doc: Clone,
        A: Clone,
    {
        allocator.text(string_interner.resolve(&self.name))
            + allocator.text(" is ")
            + allocator.text(self.bound.to_string(string_interner))
    }
}

impl TypeIdx {
    // pub fn to_doc(
    //     &self,
    //     string_interner: &'static ThreadedRodeo,
    //     types: &'b Arena<Spanned<Type>>,,
    // ) -> RcDoc<()> {
    //     let t = types.resolve(*self);
    //     t.to_doc(string_interner)
    // }
    pub fn pretty<'b, D, A>(
        &'b self,
        allocator: &'b D,
        string_interner: &'static ThreadedRodeo,
        bodies: &'b LoweredBodies,
    ) -> DocBuilder<'b, D, A>
    where
        D: DocAllocator<'b, A>,
        D::Doc: Clone,
        A: Clone,
    {
        let t = &bodies.types[self.raw()];
        t.pretty(allocator, string_interner, bodies)
    }
}

impl Type {
    //     pub fn to_doc(&self, string_interner: &'static ThreadedRodeo) -> RcDoc<()> {
    //         match self {
    //             Self::Generic(name) => RcDoc::text(string_interner.resolve(name)),
    //             _ => todo!(),
    //         }
    //     }
    pub fn pretty<'b, D, A>(
        &'b self,
        allocator: &'b D,
        string_interner: &'static ThreadedRodeo,
        bodies: &'b LoweredBodies,
    ) -> DocBuilder<'b, D, A>
    where
        D: DocAllocator<'b, A>,
        D::Doc: Clone,
        A: Clone,
    {
        match self {
            Self::Array(t, n) => {
                allocator.text("[")
                    + t.pretty(allocator, string_interner, bodies)
                    + allocator.text("; ")
                    + allocator.text(n.to_string())
                    + allocator.text("]")
            }
            Self::Path(path) => path.pretty(allocator, string_interner, bodies),
            Self::ThisPath(_, _) => allocator.text("todo"),
            Self::Ptr(ty) => allocator.text("*") + ty.pretty(allocator, string_interner, bodies),
            Self::Tuple(tys) => {
                allocator.text("(")
                    + allocator.intersperse(
                        tys.iter()
                            .map(|ty| ty.pretty(allocator, string_interner, bodies)),
                        ", ",
                    )
                    + allocator.text(")")
            }
            Self::Unknown => allocator.text("<unknown type>"),
            Self::Generic(name, restrictions) => {
                allocator.text(string_interner.resolve(name))
                    + if restrictions.is_empty() {
                        allocator.nil()
                    } else {
                        allocator.text(": ")
                            + allocator.intersperse(
                                restrictions.iter().map(|restriction| {
                                    restriction.pretty(allocator, string_interner, bodies)
                                }),
                                ", ",
                            )
                    }
            }
        }
    }
}

impl Path {
    pub fn pretty<'b, D, A>(
        &'b self,
        allocator: &'b D,
        string_interner: &'static ThreadedRodeo,
        bodies: &'b LoweredBodies,
    ) -> DocBuilder<'b, D, A>
    where
        D: DocAllocator<'b, A>,
        D::Doc: Clone,
        A: Clone,
    {
        allocator.intersperse(
            self.segments
                .iter()
                .map(|segment| string_interner.resolve(segment)),
            "::",
        ) + if self.generic_args.is_empty() {
            allocator.text("")
        } else {
            allocator.text("<")
                + allocator.intersperse(
                    self.generic_args
                        .iter()
                        .map(|arg| arg.pretty(allocator, string_interner, bodies)),
                    ",",
                )
                + allocator.text(">")
        }
    }
}

impl ExprIdx {
    pub fn pretty<'b, D, A>(
        &'b self,
        allocator: &'b D,
        string_interner: &'static ThreadedRodeo,
        bodies: &'b LoweredBodies,
    ) -> DocBuilder<'b, D, A>
    where
        D: DocAllocator<'b, A>,
        D::Doc: Clone,
        A: Clone,
    {
        bodies.exprs[self.raw()].pretty(allocator, string_interner, bodies)
    }
}

impl Expr {
    pub fn pretty<'b, D, A>(
        &'b self,
        allocator: &'b D,
        string_interner: &'static ThreadedRodeo,
        bodies: &'b LoweredBodies,
    ) -> DocBuilder<'b, D, A>
    where
        D: DocAllocator<'b, A>,
        D::Doc: Clone,
        A: Clone,
    {
        match self {
            Self::Block(block) => block.pretty(allocator, string_interner, bodies),
            Self::BinOp(binop) => binop.pretty(allocator, string_interner, bodies),
            Self::Enum(eenum) => eenum.pretty(allocator, string_interner, bodies),
            Self::Call(call) => call.pretty(allocator, string_interner, bodies),
            Self::Float(float) => allocator.text(float.to_string()),
            Self::Int(int) => allocator.text(int.to_string()),
            Self::Let(l) => l.pretty(allocator, string_interner, bodies),
            Self::MemberAccess(access) => access.pretty(allocator, string_interner, bodies),
            Self::Path(path) => path.pretty(allocator, string_interner, bodies),
            Self::Poisoned => allocator.text("<poisoned expression>"),
            Self::Struct(strukt) => strukt.pretty(allocator, string_interner, bodies),
            Self::Tuple(vals) => {
                allocator.text("(")
                    + allocator.intersperse(
                        vals.iter()
                            .map(|val| val.pretty(allocator, string_interner, bodies)),
                        ", ",
                    )
                    + allocator.text(")")
            }
        }
    }
}

impl Block {
    pub fn pretty<'b, D, A>(
        &'b self,
        allocator: &'b D,
        string_interner: &'static ThreadedRodeo,
        bodies: &'b LoweredBodies,
    ) -> DocBuilder<'b, D, A>
    where
        D: DocAllocator<'b, A>,
        D::Doc: Clone,
        A: Clone,
    {
        let exprs = self
            .exprs
            .iter()
            .map(|expr| expr.pretty(allocator, string_interner, bodies));
        allocator.text("{")
            + allocator.line()
            + allocator.intersperse(exprs, allocator.hardline()).indent(4)
            + allocator.line()
            + allocator.text("}")
    }
}

impl BinOp {
    pub fn pretty<'b, D, A>(
        &'b self,
        allocator: &'b D,
        string_interner: &'static ThreadedRodeo,
        bodies: &'b LoweredBodies,
    ) -> DocBuilder<'b, D, A>
    where
        D: DocAllocator<'b, A>,
        D::Doc: Clone,
        A: Clone,
    {
        self.lhs.pretty(allocator, string_interner, bodies)
            + allocator.space()
            + self.op.pretty(allocator)
            + allocator.space()
            + self.rhs.pretty(allocator, string_interner, bodies)
    }
}

impl Op {
    pub fn pretty<'b, D, A>(&'b self, allocator: &'b D) -> DocBuilder<'b, D, A>
    where
        D: DocAllocator<'b, A>,
        D::Doc: Clone,
        A: Clone,
    {
        match self {
            Op::Eq => allocator.text("="),
            Op::Plus => allocator.text("+"),
        }
    }
}

impl EnumExpr {
    pub fn pretty<'b, D, A>(
        &'b self,
        allocator: &'b D,
        string_interner: &'static ThreadedRodeo,
        bodies: &'b LoweredBodies,
    ) -> DocBuilder<'b, D, A>
    where
        D: DocAllocator<'b, A>,
        D::Doc: Clone,
        A: Clone,
    {
        self.path.pretty(allocator, string_interner, bodies)
            + allocator.text("::")
            + allocator.text(string_interner.resolve(&self.variant))
            + if let Some(arg) = &self.arg {
                allocator.text("(")
                    + arg.pretty(allocator, string_interner, bodies)
                    + allocator.text(")")
            } else {
                allocator.nil()
            }
    }
}

impl Call {
    pub fn pretty<'b, D, A>(
        &'b self,
        allocator: &'b D,
        string_interner: &'static ThreadedRodeo,
        bodies: &'b LoweredBodies,
    ) -> DocBuilder<'b, D, A>
    where
        D: DocAllocator<'b, A>,
        D::Doc: Clone,
        A: Clone,
    {
        self.callee.pretty(allocator, string_interner, bodies)
            + allocator.text("(")
            + allocator.intersperse(
                self.args
                    .iter()
                    .map(|arg| arg.pretty(allocator, string_interner, bodies)),
                ", ",
            )
            + allocator.text(")")
    }
}

impl Let {
    pub fn pretty<'b, D, A>(
        &'b self,
        allocator: &'b D,
        string_interner: &'static ThreadedRodeo,
        bodies: &'b LoweredBodies,
    ) -> DocBuilder<'b, D, A>
    where
        D: DocAllocator<'b, A>,
        D::Doc: Clone,
        A: Clone,
    {
        let ty = bodies
            .tid_to_tkind
            .get(&self.val.tid)
            .map(|ty| ty.pretty(allocator, string_interner, bodies))
            .unwrap_or_else(|| allocator.text("<unknown type>"));
        (allocator.text("let ")
            + allocator.text(string_interner.resolve(&self.name))
            + allocator.text(" ")
            + ty
            + allocator.text(" = ")
            + self.val.pretty(allocator, string_interner, bodies)
            + allocator.text(";"))
        .group()
    }
}

impl MemberAccess {
    pub fn pretty<'b, D, A>(
        &'b self,
        allocator: &'b D,
        string_interner: &'static ThreadedRodeo,
        bodies: &'b LoweredBodies,
    ) -> DocBuilder<'b, D, A>
    where
        D: DocAllocator<'b, A>,
        D::Doc: Clone,
        A: Clone,
    {
        self.lhs.pretty(allocator, string_interner, bodies)
            + allocator.text(".")
            + allocator.text(string_interner.resolve(&self.rhs))
    }
}

impl StructExpr {
    pub fn pretty<'b, D, A>(
        &'b self,
        allocator: &'b D,
        string_interner: &'static ThreadedRodeo,
        bodies: &'b LoweredBodies,
    ) -> DocBuilder<'b, D, A>
    where
        D: DocAllocator<'b, A>,
        D::Doc: Clone,
        A: Clone,
    {
        let fields = self
            .fields
            .iter()
            .map(|field| field.pretty(allocator, string_interner, bodies));
        allocator.text("struct ")
            + self.path.pretty(allocator, string_interner, bodies)
            + allocator.text(" {")
            + allocator.line()
            + allocator
                .intersperse(fields, allocator.text(", ") + allocator.line())
                .group()
                .indent(4)
            + allocator.line()
            + allocator.text("}")
    }
}

impl StructExprField {
    pub fn pretty<'b, D, A>(
        &'b self,
        allocator: &'b D,
        string_interner: &'static ThreadedRodeo,
        bodies: &'b LoweredBodies,
    ) -> DocBuilder<'b, D, A>
    where
        D: DocAllocator<'b, A>,
        D::Doc: Clone,
        A: Clone,
    {
        allocator.text(string_interner.resolve(&self.name))
            + allocator.text(": ")
            + self.val.pretty(allocator, string_interner, bodies)
    }
}
