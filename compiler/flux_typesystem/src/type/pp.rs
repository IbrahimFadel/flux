use lasso::ThreadedRodeo;
use pretty::{DocAllocator, DocBuilder};

use crate::{env::TEntry, ConcreteKind, TEnv, TypeId, TypeKind};

impl TypeId {
    pub fn pretty<'b, D, A>(
        &'b self,
        allocator: &'b D,
        string_interner: &'static ThreadedRodeo,
        tenv: &'b TEnv,
    ) -> DocBuilder<'b, D, A>
    where
        D: DocAllocator<'b, A>,
        D::Doc: Clone,
        A: Clone,
    {
        tenv.get_entry(*self)
            .pretty(allocator, string_interner, tenv)
    }
}

impl TEntry {
    pub fn pretty<'b, D, A>(
        &'b self,
        allocator: &'b D,
        string_interner: &'static ThreadedRodeo,
        tenv: &'b TEnv,
    ) -> DocBuilder<'b, D, A>
    where
        D: DocAllocator<'b, A>,
        D::Doc: Clone,
        A: Clone,
    {
        self.get_constr().pretty(allocator, string_interner, tenv)
            + self.get_params().map_or_else(
                || allocator.nil(),
                |params| {
                    allocator.intersperse(
                        params
                            .iter()
                            .map(|param| param.pretty(allocator, string_interner, tenv)),
                        ",",
                    )
                },
            )
    }
}

impl TypeKind {
    pub fn pretty<'b, D, A>(
        &'b self,
        allocator: &'b D,
        string_interner: &'static ThreadedRodeo,
        tenv: &'b TEnv,
    ) -> DocBuilder<'b, D, A>
    where
        D: DocAllocator<'b, A>,
        D::Doc: Clone,
        A: Clone,
    {
        match self {
            TypeKind::AssocPath(path) => {
                allocator.text(format!("This::{}", string_interner.resolve(path)))
            }
            TypeKind::Concrete(concrete) => concrete.pretty(allocator, string_interner, tenv),
            TypeKind::Int(depends_on) => match depends_on {
                None => allocator.text("<int type>"),
                Some(id) => id.pretty(allocator, string_interner, tenv),
            },
            TypeKind::Float(depends_on) => match depends_on {
                None => allocator.text("<float type>"),
                Some(id) => id.pretty(allocator, string_interner, tenv),
            },
            TypeKind::Ref(id) => id.pretty(allocator, string_interner, tenv),
            TypeKind::Generic(name, _restrictions) => {
                allocator.text(string_interner.resolve(name))
                // + if restrictions.is_empty() {
                //     allocator.nil()
                // } else {
                //     allocator.text("<")
                // }
            }
            TypeKind::Never => allocator.text("!"),
            TypeKind::Unknown => allocator.text("<unknown type>"),
        }
    }
}

impl ConcreteKind {
    pub fn pretty<'b, D, A>(
        &'b self,
        allocator: &'b D,
        string_interner: &'static ThreadedRodeo,
        tenv: &'b TEnv,
    ) -> DocBuilder<'b, D, A>
    where
        D: DocAllocator<'b, A>,
        D::Doc: Clone,
        A: Clone,
    {
        match self {
            ConcreteKind::Array(t, n) => {
                allocator.text("[")
                    + t.pretty(allocator, string_interner, tenv)
                    + allocator.text(",")
                    + allocator.space()
                    + allocator.text(format!("{}]", n))
            }
            ConcreteKind::Ptr(t) => {
                allocator.text("*") + t.pretty(allocator, string_interner, tenv)
            }
            ConcreteKind::Path(path, args) => {
                allocator.text(string_interner.resolve(path))
                    + if args.is_empty() {
                        allocator.nil()
                    } else {
                        allocator.text("<")
                            + allocator.intersperse(
                                args.iter()
                                    .map(|arg| arg.pretty(allocator, string_interner, tenv)),
                                ", ",
                            )
                            + allocator.text(">")
                    }
            }
            ConcreteKind::Tuple(types) => {
                allocator.text("(")
                    + allocator.intersperse(
                        types
                            .iter()
                            .map(|t| t.pretty(allocator, string_interner, tenv)),
                        ", ",
                    )
                    + allocator.text(")")
            }
        }
    }
}
