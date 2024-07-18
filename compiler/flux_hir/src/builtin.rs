use flux_span::{Interner, Word};

#[derive(Debug, Clone, Hash)]
pub(crate) enum BuiltinType {
    UInt(BuiltinUInt),
    SInt(BuiltinSInt),
    Float(BuiltinFloat),
    Str,
    Bool,
}

impl BuiltinType {
    pub(crate) fn ints(interner: &'static Interner) -> [(Word, BuiltinType); 8] {
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
        ]
    }

    pub(crate) fn floats(interner: &'static Interner) -> [(Word, BuiltinType); 2] {
        [
            (
                interner.get_or_intern_static("f64"),
                BuiltinType::Float(BuiltinFloat::F64),
            ),
            (
                interner.get_or_intern_static("f32"),
                BuiltinType::Float(BuiltinFloat::F32),
            ),
        ]
    }

    pub(crate) fn str(interner: &'static Interner) -> (Word, BuiltinType) {
        (interner.get_or_intern_static("str"), BuiltinType::Str)
    }

    pub(crate) fn bool(interner: &'static Interner) -> (Word, BuiltinType) {
        (interner.get_or_intern_static("bool"), BuiltinType::Bool)
    }

    pub(crate) fn all(interner: &'static Interner) -> [(Word, BuiltinType); 12] {
        let ints = Self::ints(interner);
        let floats = Self::floats(interner);
        let str = Self::str(interner);
        let bool = Self::bool(interner);
        unsafe {
            let mut result = std::mem::MaybeUninit::uninit();
            let dest = result.as_mut_ptr() as *mut (Word, BuiltinType);
            std::ptr::copy_nonoverlapping(ints.as_ptr(), dest, ints.len());
            std::ptr::copy_nonoverlapping(floats.as_ptr(), dest.add(floats.len()), floats.len());
            std::ptr::copy_nonoverlapping(
                &str as *const (Word, BuiltinType),
                dest.add(floats.len() + 1),
                1,
            );
            std::ptr::copy_nonoverlapping(
                &bool as *const (Word, BuiltinType),
                dest.add(floats.len() + 2),
                1,
            );
            result.assume_init()
        }
    }
}

#[derive(Debug, Clone, Hash)]
pub(crate) enum BuiltinUInt {
    U64,
    U32,
    U16,
    U8,
}

#[derive(Debug, Clone, Hash)]
pub(crate) enum BuiltinSInt {
    S64,
    S32,
    S16,
    S8,
}

#[derive(Debug, Clone, Hash)]
pub(crate) enum BuiltinFloat {
    F64,
    F32,
}
