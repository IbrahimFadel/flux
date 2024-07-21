use cstree::interning::TokenKey;
use flux_span::{FileId, Interner, Span};
use flux_typesystem::{FnSignature, TEnv};

macro_rules! prefix {
    ($name:expr) => {
        concat!("@flux.intrinsics.", $name)
    };
}

macro_rules! intrinsic_signatures {
    ($name:ident($($param_ty:ident),*) -> !; $($rest:tt)*) => {
        paste::paste! {
            fn [<$name _signature>](file_id: FileId, span: Span, tenv: &mut TEnv) -> FnSignature {
                FnSignature::new(
                    [$(tenv.[<insert_$param_ty>](file_id, span)),*].into_iter(),
                    tenv.insert_never(file_id, span),
                )
            }
        }
        intrinsic_signatures!($($rest)*);
    };
    ($name:ident($($param_ty:ident),*) -> $ret_ty:ident; $($rest:tt)*) => {
        paste::paste! {
            fn [<$name _signature>](file_id: FileId, span: Span, tenv: &mut TEnv) -> FnSignature {
                FnSignature::new(
                    [$(tenv.[<insert_$param_ty>](file_id, span)),*].into_iter(),
                    tenv.[<insert_$ret_ty>](file_id, span),
                )
            }
        }
        intrinsic_signatures!($($rest)*);
    };
    () => {}
}

intrinsic_signatures!(
    panic(str) -> !;

    add_s64(s64, s64) -> s64;
    add_s32(s32, s32) -> s32;
    add_s16(s16, s16) -> s16;
    add_s8(s8, s8) -> s8;
    add_u64(u64, u64) -> u64;
    add_u32(u32, u32) -> u32;
    add_u16(u16, u16) -> u16;
    add_u8(u8, u8) -> u8;

    cmp_eq_s64(s64, s64) -> bool;
    cmp_eq_s32(s32, s32) -> bool;
    cmp_eq_s16(s16, s16) -> bool;
    cmp_eq_s8(s8, s8) -> bool;
    cmp_eq_u64(u64, u64) -> bool;
    cmp_eq_u32(u32, u32) -> bool;
    cmp_eq_u16(u16, u16) -> bool;
    cmp_eq_u8(u8, u8) -> bool;
);

pub(crate) fn get_signature(
    intrinsic_name: TokenKey,
    file_id: FileId,
    span: Span,
    tenv: &mut TEnv,
    interner: &'static Interner,
) -> Option<FnSignature> {
    if intrinsic_name == interner.get_or_intern_static("panic") {
        Some(panic_signature(file_id, span, tenv))
    } else if intrinsic_name == interner.get_or_intern_static(prefix!("add_s64")) {
        Some(add_s64_signature(file_id, span, tenv))
    } else if intrinsic_name == interner.get_or_intern_static(prefix!("add_s32")) {
        Some(add_s32_signature(file_id, span, tenv))
    } else if intrinsic_name == interner.get_or_intern_static(prefix!("add_s16")) {
        Some(add_s16_signature(file_id, span, tenv))
    } else if intrinsic_name == interner.get_or_intern_static(prefix!("add_s8")) {
        Some(add_s8_signature(file_id, span, tenv))
    } else if intrinsic_name == interner.get_or_intern_static(prefix!("add_u64")) {
        Some(add_u64_signature(file_id, span, tenv))
    } else if intrinsic_name == interner.get_or_intern_static(prefix!("add_u32")) {
        Some(add_u32_signature(file_id, span, tenv))
    } else if intrinsic_name == interner.get_or_intern_static(prefix!("add_u16")) {
        Some(add_u16_signature(file_id, span, tenv))
    } else if intrinsic_name == interner.get_or_intern_static(prefix!("add_u8")) {
        Some(add_u8_signature(file_id, span, tenv))
    } else if intrinsic_name == interner.get_or_intern_static(prefix!("cmp_eq_s64")) {
        Some(cmp_eq_s64_signature(file_id, span, tenv))
    } else if intrinsic_name == interner.get_or_intern_static(prefix!("cmp_eq_s32")) {
        Some(cmp_eq_s32_signature(file_id, span, tenv))
    } else if intrinsic_name == interner.get_or_intern_static(prefix!("cmp_eq_s16")) {
        Some(cmp_eq_s16_signature(file_id, span, tenv))
    } else if intrinsic_name == interner.get_or_intern_static(prefix!("cmp_eq_s8")) {
        Some(cmp_eq_s8_signature(file_id, span, tenv))
    } else if intrinsic_name == interner.get_or_intern_static(prefix!("cmp_eq_u64")) {
        Some(cmp_eq_u64_signature(file_id, span, tenv))
    } else if intrinsic_name == interner.get_or_intern_static(prefix!("cmp_eq_u32")) {
        Some(cmp_eq_u32_signature(file_id, span, tenv))
    } else if intrinsic_name == interner.get_or_intern_static(prefix!("cmp_eq_u16")) {
        Some(cmp_eq_u16_signature(file_id, span, tenv))
    } else if intrinsic_name == interner.get_or_intern_static(prefix!("cmp_eq_u8")) {
        Some(cmp_eq_u8_signature(file_id, span, tenv))
    } else {
        None
    }
}
