use std::{
    fmt::Display,
    hash::{BuildHasher, Hash},
};

use hashbrown::{hash_map::RawEntryMut, HashMap};
use lasso::ThreadedRodeo;

use crate::hir::{Path, Type};
use flux_span::{Span, WithSpan};

#[derive(Debug)]
pub(crate) struct TypeInterner {
    ty_to_key: HashMap<Type, TypeIdx>,
    key_to_ty: HashMap<TypeIdx, Type>,
}

impl TypeInterner {
    pub fn new(string_interner: &'static ThreadedRodeo) -> Self {
        let mut interner = Self {
            ty_to_key: HashMap::new(),
            key_to_ty: HashMap::new(),
        };
        interner.intern(Type::Tuple(vec![]));
        interner.intern(Type::Path(
            Path::from_str_static("i8".at(Span::new(0..0)), string_interner),
            vec![],
        ));
        interner.intern(Type::Path(
            Path::from_str_static("i16".at(Span::new(0..0)), string_interner),
            vec![],
        ));
        interner.intern(Type::Path(
            Path::from_str_static("i32".at(Span::new(0..0)), string_interner),
            vec![],
        ));
        interner.intern(Type::Path(
            Path::from_str_static("i64".at(Span::new(0..0)), string_interner),
            vec![],
        ));
        interner.intern(Type::Path(
            Path::from_str_static("u8".at(Span::new(0..0)), string_interner),
            vec![],
        ));
        interner.intern(Type::Path(
            Path::from_str_static("u16".at(Span::new(0..0)), string_interner),
            vec![],
        ));
        interner.intern(Type::Path(
            Path::from_str_static("u32".at(Span::new(0..0)), string_interner),
            vec![],
        ));
        interner.intern(Type::Path(
            Path::from_str_static("u64".at(Span::new(0..0)), string_interner),
            vec![],
        ));
        interner
    }

    pub const fn unit(&self) -> TypeIdx {
        TypeIdx::new(0)
    }

    pub fn intern(&mut self, ty: Type) -> TypeIdx {
        let len = self.ty_to_key.len();
        let hash = hash_ty(self.ty_to_key.hasher(), &ty);
        match self
            .ty_to_key
            .raw_entry_mut()
            .from_key_hashed_nocheck(hash, &ty)
        {
            RawEntryMut::Occupied(entry) => *entry.get(),
            RawEntryMut::Vacant(entry) => {
                let key = TypeIdx::new(len.try_into().unwrap());
                entry.insert_hashed_nocheck(hash, ty.clone(), key);
                self.key_to_ty.insert(key, ty);
                key
            }
        }
    }

    pub fn resolve(&self, key: TypeIdx) -> &Type {
        assert!((key.0 as usize) < self.ty_to_key.len());
        assert!((key.0 as usize) < self.key_to_ty.len());
        self.key_to_ty
            .get(&key)
            .expect("internal compiler error: invalid type index")
    }
}

fn hash_ty<S: BuildHasher>(hash_builder: &S, ty: &Type) -> u64 {
    use core::hash::Hasher;
    let mut state = hash_builder.build_hasher();
    match ty {
        Type::Path(path, params) => {
            path.get_unspanned_spurs().hash(&mut state);
            let vec: Vec<_> = params.iter().map(|idx| idx.inner).collect();
            vec.hash(&mut state);
        }
        Type::Tuple(ids) => {
            let vec: Vec<_> = ids.iter().map(|idx| idx.inner).collect();
            vec.hash(&mut state);
        }
        _ => ty.hash(&mut state),
    }
    let hash = state.finish();
    hash
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TypeIdx(u32);

impl TypeIdx {
    pub(crate) const fn new(key: u32) -> Self {
        Self(key)
    }
}

impl Display for TypeIdx {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl WithSpan for TypeIdx {
    fn at(self, span: Span) -> flux_span::Spanned<Self> {
        self.0.at(span).map(TypeIdx::new)
    }
}

#[cfg(test)]
mod test {
    use lasso::ThreadedRodeo;
    use once_cell::sync::Lazy;

    use crate::{
        hir::{Path, Type},
        type_interner::TypeIdx,
    };
    use flux_span::{Span, WithSpan};

    use super::TypeInterner;

    static INTERNER: Lazy<ThreadedRodeo> = Lazy::new(ThreadedRodeo::new);

    #[test]
    fn basic_interning_functionality() {
        let mut type_interner = TypeInterner::new(&INTERNER);
        assert_eq!(type_interner.ty_to_key.len(), type_interner.key_to_ty.len());
        assert_eq!(type_interner.ty_to_key.len(), 9);

        let idx0 = type_interner.intern(Type::Tuple(vec![]));
        let idx1 = type_interner.intern(Type::Tuple(vec![]));
        assert_eq!(idx0, idx1);
        let i32_idx = type_interner.intern(Type::Path(
            Path::from_str_static("i32".at(Span::new(0..0)), &INTERNER),
            vec![],
        ));
        let idx1 = type_interner.intern(Type::Tuple(vec![i32_idx.at(Span::new(0..0))]));
        assert_ne!(idx0, idx1);

        let len = type_interner.ty_to_key.len();
        let new_idx = type_interner.intern(Type::Path(
            Path::from_str_static("foo".at(Span::new(0..0)), &INTERNER),
            vec![],
        ));
        assert_eq!(new_idx, TypeIdx::new(len as u32));
    }

    #[test]
    fn interns_independently_of_spans() {
        let mut type_interner = TypeInterner::new(&INTERNER);
        let path0 = type_interner.intern(Type::Path(
            Path::from_str_static("foo".at(Span::new(0..0)), &INTERNER),
            vec![],
        ));
        let path1 = type_interner.intern(Type::Path(
            Path::from_str_static("foo".at(Span::new(1..2)), &INTERNER),
            vec![],
        ));
        assert_eq!(path0, path1);
        let tuple0 = type_interner.intern(Type::Tuple(vec![path0.at(Span::new(0..0))]));
        let tuple1 = type_interner.intern(Type::Tuple(vec![path0.at(Span::new(1..2))]));
        assert_eq!(tuple0, tuple1);
    }
}
