use std::{collections::HashMap, sync::OnceLock};

use flux_typesystem::{FnSignature, Type};
use flux_util::{Interner, Path, Word};

macro_rules! prefix {
    ($name:expr) => {
        concat!("@flux.intrinsics.", $name)
    };
}

macro_rules! intrinsic_signatures {
($name:ident($($param_ty:ident),*) -> !; $($rest:tt)*) => {
		paste::paste! {
				fn [<$name _signature>]( interner: &'static Interner) -> FnSignature {
						FnSignature::new(
								[$(Type::path(Path::new(vec![interner.get_or_intern(stringify!($param_ty))], vec![]))),*].into_iter(),
								Type::never(),
						)
				}
		}
		intrinsic_signatures!($($rest)*);
};
($name:ident($($param_ty:ident),*) -> $ret_ty:ident *; $($rest:tt)*) => {
		paste::paste! {
				fn [<$name _signature>](interner: &'static Interner) -> FnSignature {
						FnSignature::new(
								[$(Type::path(Path::new(vec![interner.get_or_intern(stringify!($param_ty))], vec![]))),*].into_iter(),
								Type::ptr(Type::path(Path::new(vec![interner.get_or_intern(stringify!($ret_ty))], vec![])))
						)
				}
		}
		intrinsic_signatures!($($rest)*);
};
($name:ident($($param_ty:ident),*) -> $ret_ty:ident; $($rest:tt)*) => {
		paste::paste! {
				fn [<$name _signature>](interner: &'static Interner) -> FnSignature {
						FnSignature::new(
								[$(Type::path(Path::new(vec![interner.get_or_intern(stringify!($param_ty))], vec![]))),*].into_iter(),
								Type::path(Path::new(vec![interner.get_or_intern(stringify!($ret_ty))], vec![]))
						)
				}
		}
		intrinsic_signatures!($($rest)*);
};
() => {}
}

intrinsic_signatures!(
    panic(str) -> !;
    malloc(u64) -> u8*;

    add_s64(s64, s64) -> s64;
    add_s32(s32, s32) -> s32;
    add_s16(s16, s16) -> s16;
    add_s8(s8, s8) -> s8;
    add_u64(u64, u64) -> u64;
    add_u32(u32, u32) -> u32;
    add_u16(u16, u16) -> u16;
    add_u8(u8, u8) -> u8;

    mul_s64(s64, s64) -> s64;
    mul_s32(s32, s32) -> s32;
    mul_s16(s16, s16) -> s16;
    mul_s8(s8, s8) -> s8;
    mul_u64(u64, u64) -> u64;
    mul_u32(u32, u32) -> u32;
    mul_u16(u16, u16) -> u16;
    mul_u8(u8, u8) -> u8;

    cmp_eq_s64(s64, s64) -> bool;
    cmp_eq_s32(s32, s32) -> bool;
    cmp_eq_s16(s16, s16) -> bool;
    cmp_eq_s8(s8, s8) -> bool;
    cmp_eq_u64(u64, u64) -> bool;
    cmp_eq_u32(u32, u32) -> bool;
    cmp_eq_u16(u16, u16) -> bool;
    cmp_eq_u8(u8, u8) -> bool;
);

type Handler = fn(&'static Interner) -> FnSignature;
static SIGNATURE_MAP: OnceLock<HashMap<Word, Handler>> = OnceLock::new();

pub(crate) fn get_signature(
    intrinsic_name: &Word,
    interner: &'static Interner,
) -> Option<FnSignature> {
    let signature_map = SIGNATURE_MAP.get_or_init(|| {
        HashMap::from([
            (
                interner.get_or_intern_static(prefix!("panic")),
                panic_signature as Handler,
            ),
            (
                interner.get_or_intern_static(prefix!("malloc")),
                malloc_signature as Handler,
            ),
            (
                interner.get_or_intern_static(prefix!("add_s64")),
                add_s64_signature as Handler,
            ),
            (
                interner.get_or_intern_static(prefix!("add_s32")),
                add_s32_signature as Handler,
            ),
            (
                interner.get_or_intern_static(prefix!("add_s16")),
                add_s16_signature as Handler,
            ),
            (
                interner.get_or_intern_static(prefix!("add_s8")),
                add_s8_signature as Handler,
            ),
            (
                interner.get_or_intern_static(prefix!("add_u64")),
                mul_u64_signature as Handler,
            ),
            (
                interner.get_or_intern_static(prefix!("add_u32")),
                mul_u32_signature as Handler,
            ),
            (
                interner.get_or_intern_static(prefix!("add_u16")),
                mul_u16_signature as Handler,
            ),
            (
                interner.get_or_intern_static(prefix!("add_u8")),
                mul_u8_signature as Handler,
            ),
            (
                interner.get_or_intern_static(prefix!("mul_s64")),
                mul_s64_signature as Handler,
            ),
            (
                interner.get_or_intern_static(prefix!("mul_s32")),
                mul_s32_signature as Handler,
            ),
            (
                interner.get_or_intern_static(prefix!("mul_s16")),
                mul_s16_signature as Handler,
            ),
            (
                interner.get_or_intern_static(prefix!("mul_s8")),
                mul_s8_signature as Handler,
            ),
            (
                interner.get_or_intern_static(prefix!("mul_u64")),
                mul_u64_signature as Handler,
            ),
            (
                interner.get_or_intern_static(prefix!("mul_u32")),
                mul_u32_signature as Handler,
            ),
            (
                interner.get_or_intern_static(prefix!("mul_u16")),
                mul_u16_signature as Handler,
            ),
            (
                interner.get_or_intern_static(prefix!("mul_u8")),
                mul_u8_signature as Handler,
            ),
            (
                interner.get_or_intern_static(prefix!("cmp_eq_s64")),
                cmp_eq_s64_signature as Handler,
            ),
            (
                interner.get_or_intern_static(prefix!("cmp_eq_s32")),
                cmp_eq_s32_signature as Handler,
            ),
            (
                interner.get_or_intern_static(prefix!("cmp_eq_s16")),
                cmp_eq_s16_signature as Handler,
            ),
            (
                interner.get_or_intern_static(prefix!("cmp_eq_s8")),
                cmp_eq_s8_signature as Handler,
            ),
            (
                interner.get_or_intern_static(prefix!("cmp_eq_u64")),
                cmp_eq_u64_signature as Handler,
            ),
            (
                interner.get_or_intern_static(prefix!("cmp_eq_u32")),
                cmp_eq_u32_signature as Handler,
            ),
            (
                interner.get_or_intern_static(prefix!("cmp_eq_u16")),
                cmp_eq_u16_signature as Handler,
            ),
            (
                interner.get_or_intern_static(prefix!("cmp_eq_u8")),
                cmp_eq_u8_signature as Handler,
            ),
        ])
    });

    signature_map
        .get(intrinsic_name)
        .map(|handler| handler(interner))
}
