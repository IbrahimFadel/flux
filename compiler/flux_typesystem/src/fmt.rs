use itertools::Itertools;

use crate::{ConcreteKind, Generic, TEnv, TypeId, TypeKind};

impl TEnv {
    pub fn fmt_tid(&self, tid: &TypeId) -> String {
        self.fmt_tkind(&self.get(tid))
    }

    pub fn fmt_tkind(&self, tkind: &TypeKind) -> String {
        use crate::TypeKind::*;
        match tkind {
            ThisPath(this_path, _) => std::iter::once("This")
                .chain(this_path.iter().map(|key| self.interner.resolve(key)))
                .join("::"),
            Concrete(concrete_kind) => self.fmt_concrete_kind(concrete_kind),
            Int(_) => format!("int"),
            Float(_) => format!("float"),
            Ref(tid) => self.fmt_tid(tid),
            // Ref(tid) => format!(""),
            Generic(generic) => self.fmt_generic(generic),
            Never => format!("!"),
            Unknown => format!("unknown"),
        }
    }

    fn fmt_concrete_kind(&self, concrete_kind: &ConcreteKind) -> String {
        use ConcreteKind::*;
        match concrete_kind {
            Array(tid, n) => format!("[{}; {n}]", self.fmt_tid(tid)),
            Ptr(tid) => format!("*{}", self.fmt_tid(tid)),
            Path(path) => format!(
                "{}{}",
                path.segments
                    .iter()
                    .map(|key| self.interner.resolve(key))
                    .join("::"),
                if path.generic_args.is_empty() {
                    String::new()
                } else {
                    format!(
                        "<{}>",
                        path.generic_args
                            .iter()
                            .map(|tid| self.fmt_tid(tid))
                            .join(", ")
                    )
                }
            ),
            Tuple(tids) => format!("({})", tids.iter().map(|tid| self.fmt_tid(tid)).join(", ")),
        }
    }

    fn fmt_generic(&self, generic: &Generic) -> String {
        format!("{}", self.interner.resolve(&generic.name))
    }
}
