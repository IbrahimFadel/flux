macro_rules! prefix {
    ($name:expr) => {
        concat!("@flux.intrinsics.", $name)
    };
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
