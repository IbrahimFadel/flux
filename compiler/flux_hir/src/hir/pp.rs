use flux_span::Spanned;
use la_arena::Arena;
use lasso::ThreadedRodeo;
use pretty::{DocAllocator, DocBuilder, RcDoc};

use crate::{type_interner::TypeIdx, TypeInterner};

use super::{
    Block, Call, Expr, ExprIdx, Function, GenericParams, Let, Param, Params, Path, Type,
    Visibility, WherePredicate, WherePredicates,
};

impl Function {
    pub fn pretty<'b, D, A>(
        &'b self,
        allocator: &'b D,
        string_interner: &'static ThreadedRodeo,
        type_interner: &'static TypeInterner,
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
            + self
                .params
                .pretty(allocator, string_interner, type_interner)
            + allocator.text(" -> ")
            + self
                .ret_ty
                .inner
                .pretty(allocator, string_interner, type_interner)
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
        type_interner: &'static TypeInterner,
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
                    .map(|param| param.pretty(allocator, string_interner, type_interner)),
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
        type_interner: &'static TypeInterner,
    ) -> DocBuilder<'b, D, A>
    where
        D: DocAllocator<'b, A>,
        D::Doc: Clone,
        A: Clone,
    {
        allocator.text(string_interner.resolve(&self.name))
            + allocator.text(" ")
            + self
                .ty
                .inner
                .pretty(allocator, string_interner, type_interner)
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
    pub fn to_doc(
        &self,
        string_interner: &'static ThreadedRodeo,
        type_interner: &'static TypeInterner,
    ) -> RcDoc<()> {
        let t = type_interner.resolve(*self);
        t.to_doc(string_interner)
    }
    // pub fn pretty<'b, D, A>(
    //     &'b self,
    //     allocator: &'b D,
    //     string_interner: &'static ThreadedRodeo,
    //     type_interner: &'static TypeInterner,
    // ) -> DocBuilder<'b, D, A>
    // where
    //     D: DocAllocator<'b, A>,
    //     D::Doc: Clone,
    //     A: Clone,
    // {
    //     let t = type_interner.resolve(*self);
    //     t.pretty(allocator, string_interner, type_interner)
    // }
}

impl Type {
    pub fn to_doc(&self, string_interner: &'static ThreadedRodeo) -> RcDoc<()> {
        match self {
            Self::Generic(name) => RcDoc::text(string_interner.resolve(name)),
            _ => todo!(),
        }
    }
    // pub fn pretty<'b, D, A>(
    //     &'b self,
    //     allocator: &'b D,
    //     string_interner: &'static ThreadedRodeo,
    //     type_interner: &'static TypeInterner,
    // ) -> DocBuilder<'b, D, A>
    // where
    //     D: DocAllocator<'b, A>,
    //     D::Doc: Clone,
    //     A: Clone,
    // {
    //     match self {
    //         Self::Path(path) => path.pretty(allocator, string_interner, type_interner),
    //         Self::Ptr(ty) => {
    //             allocator.text("*") + ty.inner.pretty(allocator, string_interner, type_interner)
    //         }
    //         Self::Tuple(types) => {
    //             allocator.text("(")
    //                 + allocator.intersperse(
    //                     types
    //                         .iter()
    //                         .map(|ty| ty.inner.pretty(allocator, string_interner, type_interner)),
    //                     ", ",
    //                 )
    //                 + allocator.text(")")
    //         }
    //         Self::Unknown => allocator.text("<unknown type>"),
    //         Self::Generic(name) => allocator.text(string_interner.resolve(name)),
    //     }
    // }
}

impl Path {
    pub fn pretty<'b, D, A>(
        &'b self,
        allocator: &'b D,
        string_interner: &'static ThreadedRodeo,
        type_interner: &'static TypeInterner,
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
                        .map(|arg| arg.inner.pretty(allocator, string_interner, type_interner)),
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
        type_interner: &'static TypeInterner,
        exprs: &'b Arena<Spanned<Expr>>,
    ) -> DocBuilder<'b, D, A>
    where
        D: DocAllocator<'b, A>,
        D::Doc: Clone,
        A: Clone,
    {
        exprs[self.raw()].pretty(allocator, string_interner, type_interner, exprs)
    }
}

impl Expr {
    pub fn pretty<'b, D, A>(
        &'b self,
        allocator: &'b D,
        string_interner: &'static ThreadedRodeo,
        type_interner: &'static TypeInterner,
        exprs: &'b Arena<Spanned<Expr>>,
    ) -> DocBuilder<'b, D, A>
    where
        D: DocAllocator<'b, A>,
        D::Doc: Clone,
        A: Clone,
    {
        match self {
            Self::Block(block) => block.pretty(allocator, string_interner, type_interner, exprs),
            Self::Call(call) => call.pretty(allocator, string_interner, type_interner, exprs),
            Self::Float(float) => allocator.text(float.to_string()),
            Self::Int(int) => allocator.text(int.to_string()),
            Self::Let(l) => l.pretty(allocator, string_interner, type_interner, exprs),
            Self::Path(path) => path.pretty(allocator, string_interner, type_interner),
            Self::Poisoned => allocator.text("<poisoned expression>"),
        }
    }
}

impl Block {
    pub fn pretty<'b, D, A>(
        &'b self,
        allocator: &'b D,
        string_interner: &'static ThreadedRodeo,
        type_interner: &'static TypeInterner,
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
                    .map(|expr| expr.pretty(allocator, string_interner, type_interner, exprs)),
                allocator.hardline(),
            )
            + allocator.line()
            + allocator.text("}")
    }
}

impl Call {
    pub fn pretty<'b, D, A>(
        &'b self,
        allocator: &'b D,
        string_interner: &'static ThreadedRodeo,
        type_interner: &'static TypeInterner,
        exprs: &'b Arena<Spanned<Expr>>,
    ) -> DocBuilder<'b, D, A>
    where
        D: DocAllocator<'b, A>,
        D::Doc: Clone,
        A: Clone,
    {
        self.path
            .pretty(allocator, string_interner, type_interner, exprs)
            + allocator.text("(")
            + allocator.intersperse(
                self.args
                    .iter()
                    .map(|arg| arg.pretty(allocator, string_interner, type_interner, exprs)),
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
        type_interner: &'static TypeInterner,
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
            + self
                .ty
                .inner
                .pretty(allocator, string_interner, type_interner)
            + allocator.text(" = ")
            + self
                .val
                .pretty(allocator, string_interner, type_interner, exprs)
            + allocator.text(";")
    }
}
