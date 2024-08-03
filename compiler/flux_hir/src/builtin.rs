use std::{collections::HashMap, sync::OnceLock};

use flux_diagnostics::ice;
use flux_util::{Interner, Path, Word};

use crate::def::expr::Op;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(super) enum BuiltinType {
    UInt(BuiltinUInt),
    SInt(BuiltinSInt),
    Float(BuiltinFloat),
    Str,
    Bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(super) enum BuiltinUInt {
    U64,
    U32,
    U16,
    U8,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(super) enum BuiltinSInt {
    S64,
    S32,
    S16,
    S8,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(super) enum BuiltinFloat {
    F64,
    F32,
}

static ALL: OnceLock<[(Word, BuiltinType); 12]> = OnceLock::new();

impl BuiltinType {
    pub(super) fn all(interner: &'static Interner) -> &'static [(Word, BuiltinType); 12] {
        ALL.get_or_init(|| {
            [
                (
                    interner.get_or_intern_static("u64"),
                    BuiltinType::UInt(BuiltinUInt::U64),
                ),
                (
                    interner.get_or_intern_static("u32"),
                    BuiltinType::UInt(BuiltinUInt::U32),
                ),
                (
                    interner.get_or_intern_static("u16"),
                    BuiltinType::UInt(BuiltinUInt::U16),
                ),
                (
                    interner.get_or_intern_static("u8"),
                    BuiltinType::UInt(BuiltinUInt::U8),
                ),
                (
                    interner.get_or_intern_static("s64"),
                    BuiltinType::SInt(BuiltinSInt::S64),
                ),
                (
                    interner.get_or_intern_static("s32"),
                    BuiltinType::SInt(BuiltinSInt::S32),
                ),
                (
                    interner.get_or_intern_static("s16"),
                    BuiltinType::SInt(BuiltinSInt::S16),
                ),
                (
                    interner.get_or_intern_static("s8"),
                    BuiltinType::SInt(BuiltinSInt::S8),
                ),
                (
                    interner.get_or_intern_static("f64"),
                    BuiltinType::Float(BuiltinFloat::F64),
                ),
                (
                    interner.get_or_intern_static("f32"),
                    BuiltinType::Float(BuiltinFloat::F32),
                ),
                (interner.get_or_intern_static("str"), BuiltinType::Str),
                (interner.get_or_intern_static("bool"), BuiltinType::Bool),
            ]
        })
    }
}

static BINOP_TRAIT_PATHS: OnceLock<HashMap<Op, (Path<Word>, Word)>> = OnceLock::new();

pub(crate) fn get_binop_trait(op: &Op, interner: &'static Interner) -> &'static (Path<Word>, Word) {
    let map = BINOP_TRAIT_PATHS.get_or_init(|| {
        HashMap::from(Op::binops().map(|op| {
            (
                op,
                (
                    Path::new(
                        vec![interner.get_or_intern_static(op.as_trait_name())],
                        vec![],
                    ),
                    interner.get_or_intern_static(op.as_trait_method_name()),
                ),
            )
        }))
    });
    map.get(op)
        .unwrap_or_else(|| ice(format!("could not resolve trait for binop `{}`", op)))
}
