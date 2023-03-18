use lasso::{Spur, ThreadedRodeo};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BuiltinType {
    SInt(BuiltinSInt),
    Uint(BuiltinUint),
    Float(BuiltinFloat),
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BuiltinSInt {
    S8,
    S16,
    S32,
    S64,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BuiltinUint {
    U8,
    U16,
    U32,
    U64,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BuiltinFloat {
    F32,
    F64,
}

impl BuiltinType {
    pub fn all(string_interner: &'static ThreadedRodeo) -> Vec<(Spur, BuiltinType)> {
        [
            (
                string_interner.get_or_intern_static("s8"),
                BuiltinType::SInt(BuiltinSInt::S8),
            ),
            (
                string_interner.get_or_intern_static("s16"),
                BuiltinType::SInt(BuiltinSInt::S16),
            ),
            (
                string_interner.get_or_intern_static("s32"),
                BuiltinType::SInt(BuiltinSInt::S32),
            ),
            (
                string_interner.get_or_intern_static("s64"),
                BuiltinType::SInt(BuiltinSInt::S64),
            ),
            (
                string_interner.get_or_intern_static("u8"),
                BuiltinType::Uint(BuiltinUint::U8),
            ),
            (
                string_interner.get_or_intern_static("u16"),
                BuiltinType::Uint(BuiltinUint::U16),
            ),
            (
                string_interner.get_or_intern_static("u32"),
                BuiltinType::Uint(BuiltinUint::U32),
            ),
            (
                string_interner.get_or_intern_static("u64"),
                BuiltinType::Uint(BuiltinUint::U64),
            ),
            (
                string_interner.get_or_intern_static("f32"),
                BuiltinType::Float(BuiltinFloat::F32),
            ),
            (
                string_interner.get_or_intern_static("f64"),
                BuiltinType::Float(BuiltinFloat::F64),
            ),
        ]
        .to_vec()
    }

    pub fn by_name(name: &Spur, string_interner: &'static ThreadedRodeo) -> Option<Self> {
        Self::all(string_interner)
            .iter()
            .find_map(|(n, ty)| if n == name { Some(*ty) } else { None })
    }
}
