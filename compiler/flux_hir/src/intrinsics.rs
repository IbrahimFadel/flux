use flux_typesystem::{ConcreteKind, TypeKind};
use lasso::{Spur, ThreadedRodeo};

#[inline]
pub(crate) fn panic_name(string_interner: &'static ThreadedRodeo) -> Spur {
    string_interner.get_or_intern_static("@flux.intrinsics.panic")
}

pub(crate) static PANIC_NUM_ARGS: usize = 1;
pub(crate) static PANIC_RETURN_TYPE: TypeKind = TypeKind::Never;

#[inline]
pub(crate) fn panic_param_types(string_interner: &ThreadedRodeo) -> Vec<TypeKind> {
    vec![TypeKind::Concrete(ConcreteKind::Path(
        string_interner.get_or_intern_static("str"),
    ))]
}
