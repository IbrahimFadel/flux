use std::{fmt::Display, hash::Hash};

use dashmap::{mapref::one::Ref, DashMap};
use lasso::ThreadedRodeo;

use crate::hir::{Path, Type};
use flux_span::{Span, WithSpan};

#[derive(Debug, Default)]
pub struct TypeInterner {
    ty_to_key: DashMap<Type, TypeIdx>,
    key_to_ty: DashMap<TypeIdx, Type>,
}

impl TypeInterner {
    pub fn new(string_interner: &'static ThreadedRodeo) -> Self {
        let interner = Self {
            ty_to_key: DashMap::new(),
            key_to_ty: DashMap::new(),
        };
        interner.intern(Type::Unknown);
        interner.intern(Type::Tuple(vec![]));
        // interner.intern(Type::Path(
        //     Path::from_str_static("i8".at(Span::new(0..0)), string_interner),
        //     vec![],
        // ));
        // interner.intern(Type::Path(
        //     Path::from_str_static("i16".at(Span::new(0..0)), string_interner),
        //     vec![],
        // ));
        // interner.intern(Type::Path(
        //     Path::from_str_static("i32".at(Span::new(0..0)), string_interner),
        //     vec![],
        // ));
        // interner.intern(Type::Path(
        //     Path::from_str_static("i64".at(Span::new(0..0)), string_interner),
        //     vec![],
        // ));
        // interner.intern(Type::Path(
        //     Path::from_str_static("u8".at(Span::new(0..0)), string_interner),
        //     vec![],
        // ));
        // interner.intern(Type::Path(
        //     Path::from_str_static("u16".at(Span::new(0..0)), string_interner),
        //     vec![],
        // ));
        // interner.intern(Type::Path(
        //     Path::from_str_static("u32".at(Span::new(0..0)), string_interner),
        //     vec![],
        // ));
        // interner.intern(Type::Path(
        //     Path::from_str_static("u64".at(Span::new(0..0)), string_interner),
        //     vec![],
        // ));
        interner
    }

    pub fn intern(&self, ty: Type) -> TypeIdx {
        let len = self.ty_to_key.len();
        if let Some(key) = self.ty_to_key.get(&ty) {
            *key
        } else {
            let make_new_key = || TypeIdx::new(len.try_into().unwrap());
            let key = *self
                .ty_to_key
                .entry(ty.clone())
                .or_insert_with(make_new_key)
                .value();
            self.key_to_ty.insert(key, ty);
            key
        }
    }

    pub fn resolve(&self, key: TypeIdx) -> Ref<TypeIdx, Type> {
        assert!((key.0 as usize) < self.ty_to_key.len());
        assert!((key.0 as usize) < self.key_to_ty.len());
        self.key_to_ty
            .get(&key)
            .expect("internal compiler error: invalid type index")
    }

    // pub fn intern(&self, ty: Type) -> TypeIdx {
    //     let len = self.ty_to_key.len();
    //     let hash = hash_ty(self.ty_to_key.hasher(), &ty);
    //     let res = self.ty_to_key.get_mut(&ty);
    //     match res {
    //         Some(res) => *res.value(),
    //         None => {
    //             let key = TypeIdx::new(len.try_into().unwrap());
    //             self.ty_to_key.insert(ty.clone(), key);
    //             // res.insert_hashed_nocheck(hash, ty.clone(), key);
    //             self.key_to_ty.insert(key, ty);
    //             key
    //         }
    //     }
    // }

    // pub fn resolve<'a>(&'a self, key: TypeIdx) -> &'a Type {
    //     assert!((key.0 as usize) < self.ty_to_key.len());
    //     assert!((key.0 as usize) < self.key_to_ty.len());
    //     self.key_to_ty.get(&key).expect("").value()
    //     // todo!()
    //     // .expect("internal compiler error: invalid type index")
    // }
}

// fn hash_ty<S: BuildHasher>(hash_builder: &S, ty: &Type) -> u64 {
//     use core::hash::Hasher;
//     let mut state = hash_builder.build_hasher();
//     ty.hash(&mut state);
//     let hash = state.finish();
//     hash
// }

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

// #[cfg(test)]
// mod test {
//     use lasso::ThreadedRodeo;
//     use once_cell::sync::Lazy;

//     use crate::{
//         hir::{Path, Type},
//         type_interner::TypeIdx,
//     };
//     use flux_span::{Span, WithSpan};

//     use super::TypeInterner;

//     static INTERNER: Lazy<ThreadedRodeo> = Lazy::new(ThreadedRodeo::new);

//     #[test]
//     fn basic_interning_functionality() {
//         let mut type_interner = TypeInterner::new(&INTERNER);
//         assert_eq!(type_interner.ty_to_key.len(), type_interner.key_to_ty.len());
//         assert_eq!(type_interner.ty_to_key.len(), 10);

//         let idx0 = type_interner.intern(Type::Tuple(vec![]));
//         let idx1 = type_interner.intern(Type::Tuple(vec![]));
//         assert_eq!(idx0, idx1);
//         let i32_idx = type_interner.intern(Type::Path(
//             Path::from_str_static("i32".at(Span::new(0..0)), &INTERNER),
//             vec![],
//         ));
//         let idx1 = type_interner.intern(Type::Tuple(vec![i32_idx.at(Span::new(0..0))]));
//         assert_ne!(idx0, idx1);

//         let len = type_interner.ty_to_key.len();
//         let new_idx = type_interner.intern(Type::Path(
//             Path::from_str_static("foo".at(Span::new(0..0)), &INTERNER),
//             vec![],
//         ));
//         assert_eq!(new_idx, TypeIdx::new(len as u32));
//     }

//     #[test]
//     fn interns_independently_of_spans() {
//         let mut type_interner = TypeInterner::new(&INTERNER);
//         let path0 = type_interner.intern(Type::Path(
//             Path::from_str_static("foo".at(Span::new(0..0)), &INTERNER),
//             vec![],
//         ));
//         let path1 = type_interner.intern(Type::Path(
//             Path::from_str_static("foo".at(Span::new(1..2)), &INTERNER),
//             vec![],
//         ));
//         assert_eq!(path0, path1);
//         let tuple0 = type_interner.intern(Type::Tuple(vec![path0.at(Span::new(0..0))]));
//         let tuple1 = type_interner.intern(Type::Tuple(vec![path0.at(Span::new(1..2))]));
//         assert_eq!(tuple0, tuple1);
//         let arr0 = type_interner.intern(Type::Array(
//             tuple0.at(Span::new(0..0)),
//             1.at(Span::new(0..0)),
//         ));
//         let arr1 = type_interner.intern(Type::Array(
//             tuple1.at(Span::new(1..2)),
//             1.at(Span::new(3..4)),
//         ));
//         assert_eq!(arr0, arr1);
//         let ptr0 = type_interner.intern(Type::Ptr(arr0.at(Span::new(0..0))));
//         let ptr1 = type_interner.intern(Type::Ptr(arr1.at(Span::new(1..2))));
//         assert_eq!(ptr0, ptr1);
//     }
// }
