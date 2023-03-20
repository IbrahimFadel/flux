use flux_span::Spanned;
use la_arena::Arena;
use lasso::ThreadedRodeo;
use pretty::{DocAllocator, DocBuilder};

use super::*;

impl Function {
    pub fn pretty<'b, D, A>(
        &'b self,
        allocator: &'b D,
        string_interner: &'static ThreadedRodeo,
        types: &'b Arena<Spanned<Type>>,
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
            + self.params.pretty(allocator, string_interner, types)
            + allocator.text(" -> ")
            + self.ret_ty.pretty(allocator, string_interner, types)
            + self
                .generic_params
                .where_predicates
                .pretty(allocator, string_interner)
            + allocator.text(" ")
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
            Self::Public => allocator.text("pub "),
            Self::Private => allocator.text(""),
        }
    }
}

impl Params {
    pub fn pretty<'b, D, A>(
        &'b self,
        allocator: &'b D,
        string_interner: &'static ThreadedRodeo,
        types: &'b Arena<Spanned<Type>>,
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
                    .map(|param| param.pretty(allocator, string_interner, types)),
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
        types: &'b Arena<Spanned<Type>>,
    ) -> DocBuilder<'b, D, A>
    where
        D: DocAllocator<'b, A>,
        D::Doc: Clone,
        A: Clone,
    {
        allocator.text(string_interner.resolve(&self.name))
            + allocator.text(" ")
            + self.ty.pretty(allocator, string_interner, types)
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
        types: &'b Arena<Spanned<Type>>,
    ) -> DocBuilder<'b, D, A>
    where
        D: DocAllocator<'b, A>,
        D::Doc: Clone,
        A: Clone,
    {
        let t = &types[self.raw()];
        t.pretty(allocator, string_interner, types)
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
        types: &'b Arena<Spanned<Type>>,
    ) -> DocBuilder<'b, D, A>
    where
        D: DocAllocator<'b, A>,
        D::Doc: Clone,
        A: Clone,
    {
        match self {
            Self::Array(t, n) => {
                allocator.text("[")
                    + t.pretty(allocator, string_interner, types)
                    + allocator.text("; ")
                    + allocator.text(n.to_string())
                    + allocator.text("]")
            }
            Self::Path(path) => path.pretty(allocator, string_interner, types),
            Self::Ptr(ty) => allocator.text("*") + ty.pretty(allocator, string_interner, types),
            Self::Tuple(tys) => {
                allocator.text("(")
                    + allocator.intersperse(
                        tys.iter()
                            .map(|ty| ty.pretty(allocator, string_interner, types)),
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
                                    restriction.pretty(allocator, string_interner, types)
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
        types: &'b Arena<Spanned<Type>>,
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
                        .map(|arg| arg.pretty(allocator, string_interner, types)),
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
        types: &'b Arena<Spanned<Type>>,
        exprs: &'b Arena<Spanned<Expr>>,
    ) -> DocBuilder<'b, D, A>
    where
        D: DocAllocator<'b, A>,
        D::Doc: Clone,
        A: Clone,
    {
        exprs[self.raw()].pretty(allocator, string_interner, types, exprs)
    }
}

impl Expr {
    pub fn pretty<'b, D, A>(
        &'b self,
        allocator: &'b D,
        string_interner: &'static ThreadedRodeo,
        types: &'b Arena<Spanned<Type>>,
        exprs: &'b Arena<Spanned<Expr>>,
    ) -> DocBuilder<'b, D, A>
    where
        D: DocAllocator<'b, A>,
        D::Doc: Clone,
        A: Clone,
    {
        match self {
            Self::Block(block) => block.pretty(allocator, string_interner, types, exprs),
            Self::Enum(eenum) => eenum.pretty(allocator, string_interner, types, exprs),
            Self::Call(call) => call.pretty(allocator, string_interner, types, exprs),
            Self::Float(float) => allocator.text(float.to_string()),
            Self::Int(int) => allocator.text(int.to_string()),
            Self::Let(l) => l.pretty(allocator, string_interner, types, exprs),
            Self::MemberAccess(access) => access.pretty(allocator, string_interner, types, exprs),
            Self::Path(path) => path.pretty(allocator, string_interner, types),
            Self::Poisoned => allocator.text("<poisoned expression>"),
            Self::Struct(strukt) => strukt.pretty(allocator, string_interner, types, exprs),
            Self::Tuple(vals) => {
                allocator.text("(")
                    + allocator.intersperse(
                        vals.iter()
                            .map(|val| val.pretty(allocator, string_interner, types, exprs)),
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
        types: &'b Arena<Spanned<Type>>,
        exprs: &'b Arena<Spanned<Expr>>,
    ) -> DocBuilder<'b, D, A>
    where
        D: DocAllocator<'b, A>,
        D::Doc: Clone,
        A: Clone,
    {
        allocator.text("{")
            + allocator.line()
            + allocator.intersperse(
                self.exprs
                    .iter()
                    .map(|expr| expr.pretty(allocator, string_interner, types, exprs)),
                allocator.hardline(),
            )
            + allocator.line()
            + allocator.text("}")
    }
}

impl EnumExpr {
    pub fn pretty<'b, D, A>(
        &'b self,
        allocator: &'b D,
        string_interner: &'static ThreadedRodeo,
        types: &'b Arena<Spanned<Type>>,
        exprs: &'b Arena<Spanned<Expr>>,
    ) -> DocBuilder<'b, D, A>
    where
        D: DocAllocator<'b, A>,
        D::Doc: Clone,
        A: Clone,
    {
        self.path.pretty(allocator, string_interner, types)
            + allocator.text("::")
            + allocator.text(string_interner.resolve(&self.variant))
            + if let Some(arg) = &self.arg {
                allocator.text("(")
                    + arg.pretty(allocator, string_interner, types, exprs)
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
        types: &'b Arena<Spanned<Type>>,
        exprs: &'b Arena<Spanned<Expr>>,
    ) -> DocBuilder<'b, D, A>
    where
        D: DocAllocator<'b, A>,
        D::Doc: Clone,
        A: Clone,
    {
        self.callee.pretty(allocator, string_interner, types, exprs)
            + allocator.text("(")
            + allocator.intersperse(
                self.args
                    .iter()
                    .map(|arg| arg.pretty(allocator, string_interner, types, exprs)),
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
        types: &'b Arena<Spanned<Type>>,
        exprs: &'b Arena<Spanned<Expr>>,
    ) -> DocBuilder<'b, D, A>
    where
        D: DocAllocator<'b, A>,
        D::Doc: Clone,
        A: Clone,
    {
        allocator.text("let ")
            + allocator.text(string_interner.resolve(&self.name))
            + allocator.text(" ")
            + self.ty.pretty(allocator, string_interner, types)
            + allocator.text(" = ")
            + self.val.pretty(allocator, string_interner, types, exprs)
            + allocator.text(";")
    }
}

impl MemberAccess {
    pub fn pretty<'b, D, A>(
        &'b self,
        allocator: &'b D,
        string_interner: &'static ThreadedRodeo,
        types: &'b Arena<Spanned<Type>>,
        exprs: &'b Arena<Spanned<Expr>>,
    ) -> DocBuilder<'b, D, A>
    where
        D: DocAllocator<'b, A>,
        D::Doc: Clone,
        A: Clone,
    {
        self.lhs.pretty(allocator, string_interner, types, exprs)
            + allocator.text(".")
            + allocator.text(string_interner.resolve(&self.rhs))
    }
}

impl StructExpr {
    pub fn pretty<'b, D, A>(
        &'b self,
        allocator: &'b D,
        string_interner: &'static ThreadedRodeo,
        types: &'b Arena<Spanned<Type>>,
        exprs: &'b Arena<Spanned<Expr>>,
    ) -> DocBuilder<'b, D, A>
    where
        D: DocAllocator<'b, A>,
        D::Doc: Clone,
        A: Clone,
    {
        allocator.text("struct ")
            + self.path.pretty(allocator, string_interner, types)
            + allocator.text(" {")
            + allocator.line()
            + allocator.intersperse(
                self.fields
                    .iter()
                    .map(|field| field.pretty(allocator, string_interner, types, exprs)),
                allocator.text(", ") + allocator.line(),
            )
            + allocator.line()
            + allocator.text("}")
    }
}

impl StructExprField {
    pub fn pretty<'b, D, A>(
        &'b self,
        allocator: &'b D,
        string_interner: &'static ThreadedRodeo,
        types: &'b Arena<Spanned<Type>>,
        exprs: &'b Arena<Spanned<Expr>>,
    ) -> DocBuilder<'b, D, A>
    where
        D: DocAllocator<'b, A>,
        D::Doc: Clone,
        A: Clone,
    {
        allocator.text(string_interner.resolve(&self.name))
            + allocator.text(": ")
            + self.val.pretty(allocator, string_interner, types, exprs)
    }
}
