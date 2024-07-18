use flux_span::{Interner, Word};

macro_rules! prefix {
    ($name:expr) => {
        concat!("@flux.intrinsics.", $name)
    };
}

const INTRINSIC_NAMES: [&'static str; 3] = ["panic", "add_u8", "cmp_eq_u8"];

pub(crate) fn all(interner: &'static Interner) -> Vec<Word> {
    INTRINSIC_NAMES
        .iter()
        .map(|name| interner.get_or_intern(format!("@flux.intrinsics.{}", name)))
        .collect()
}

pub(crate) mod panic {
    use flux_span::{Interner, Word};
    use flux_typesystem::{self as ts, ConcreteKind, TypeKind};

    const NAME: &str = prefix!("panic");

    pub(crate) fn name_text_key(interner: &'static Interner) -> Word {
        interner.get_or_intern_static(NAME)
    }

    pub(crate) fn arg_types(interner: &'static Interner) -> [TypeKind; 1] {
        // TODO: module for builtin types like str
        [TypeKind::Concrete(ConcreteKind::Path(ts::Path::new(
            vec![interner.get_or_intern_static("str")],
            vec![],
        )))]
    }

    pub(crate) static NUM_ARGS: usize = 1;
    pub(crate) static RETURN_TYPE: TypeKind = TypeKind::Never;
}

pub(crate) mod cmp_eq_u8 {
    use flux_span::{Interner, Word};

    const NAME: &str = prefix!("cmp_eq_u8");

    pub(crate) fn name_text_key(interner: &'static Interner) -> Word {
        interner.get_or_intern_static(NAME)
    }
}

pub(crate) mod add_u8 {
    use flux_span::{Interner, Word};

    const NAME: &str = prefix!("add_u8");

    pub(crate) fn name_text_key(interner: &'static Interner) -> Word {
        interner.get_or_intern_static(NAME)
    }
}
