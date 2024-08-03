use flux_id::id;

use crate::{ConcreteKind, TEnv, Type};

impl<'res> TEnv<'res> {
    pub fn fmt_tid(&self, tid: id::Ty) -> String {
        self.fmt_type(&self.types.get(tid))
    }

    fn fmt_type(&self, ty: &Type) -> String {
        use crate::Type::*;
        match ty {
            ThisPath(this_path) => std::iter::once("This")
                .chain(this_path.path.iter().map(|key| self.interner.resolve(key)))
                .collect::<Vec<_>>()
                .join("::"),
            Concrete(concrete_kind) => self.fmt_concrete_kind(concrete_kind),
            Int(_) => format!("int"),
            Float(_) => format!("float"),
            Generic(generic) => format!("{}", self.interner.resolve(&generic.name)),
            Ref(tid) => self.fmt_tid(*tid),
            Never => format!("!"),
            Unknown => format!("unknown"),
        }
    }

    fn fmt_concrete_kind(&self, concrete_kind: &ConcreteKind) -> String {
        use ConcreteKind::*;
        match concrete_kind {
            Array(ty, n) => format!("[{}; {n}]", self.fmt_type(ty)),
            Ptr(ty) => format!("{}*", self.fmt_type(ty)),
            Path(path) => format!(
                "{}{}",
                path.segments
                    .iter()
                    .map(|key| self.interner.resolve(key))
                    .collect::<Vec<_>>()
                    .join("::"),
                if path.args.is_empty() {
                    String::new()
                } else {
                    format!(
                        "<{}>",
                        path.args
                            .iter()
                            .map(|ty| self.fmt_type(ty))
                            .collect::<Vec<_>>()
                            .join(", ")
                    )
                }
            ),
            Tuple(tids) => format!(
                "({})",
                tids.iter()
                    .map(|ty| self.fmt_type(ty))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        }
    }
}
