use std::fmt::Display;

use hashbrown::{hash_map::Entry, HashMap};
use itertools::Itertools;
use lasso::ThreadedRodeo;
use owo_colors::OwoColorize;
use tinyvec::tiny_vec;

use crate::{ConcreteKind, TypeId, TypeKind};

macro_rules! path_kind {
    ($interner:expr, $path:expr) => {
        TypeKind::Concrete(ConcreteKind::Path(
            tiny_vec!($interner.get_or_intern($path)),
        ))
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Key(u32);

impl Key {
    pub(crate) fn new(key: u32) -> Self {
        Self(key)
    }

    pub(crate) fn into_type_id(&self) -> TypeId {
        TypeId::new(self.0 as usize)
    }
}

#[derive(Debug)]
pub(crate) struct Interner {
    ty_to_key: HashMap<TypeKind, Key>,
    key_to_ty: HashMap<Key, TypeKind>,
}

impl Interner {
    pub fn new() -> Self {
        Self {
            ty_to_key: HashMap::new(),
            key_to_ty: HashMap::new(),
        }
    }

    pub fn with_preinterned(string_interner: &'static ThreadedRodeo) -> Self {
        Self {
            ty_to_key: HashMap::from([
                (path_kind!(string_interner, "u8"), Key::new(0)),
                (path_kind!(string_interner, "u16"), Key::new(1)),
                (path_kind!(string_interner, "u32"), Key::new(2)),
                (path_kind!(string_interner, "u64"), Key::new(3)),
                (path_kind!(string_interner, "i8"), Key::new(4)),
                (path_kind!(string_interner, "i16"), Key::new(5)),
                (path_kind!(string_interner, "i32"), Key::new(6)),
                (path_kind!(string_interner, "i64"), Key::new(7)),
                (path_kind!(string_interner, "f32"), Key::new(8)),
                (path_kind!(string_interner, "f64"), Key::new(9)),
            ]),
            key_to_ty: HashMap::from([
                (Key::new(0), path_kind!(string_interner, "u8")),
                (Key::new(1), path_kind!(string_interner, "u16")),
                (Key::new(2), path_kind!(string_interner, "u32")),
                (Key::new(3), path_kind!(string_interner, "u64")),
                (Key::new(4), path_kind!(string_interner, "i8")),
                (Key::new(5), path_kind!(string_interner, "i16")),
                (Key::new(6), path_kind!(string_interner, "i32")),
                (Key::new(7), path_kind!(string_interner, "i64")),
                (Key::new(8), path_kind!(string_interner, "f32")),
                (Key::new(9), path_kind!(string_interner, "f64")),
            ]),
        }
    }

    pub fn intern(&mut self, ty: TypeKind) -> Key {
        let len = self.ty_to_key.len();
        match self.ty_to_key.entry(ty.clone()) {
            Entry::Occupied(entry) => *entry.get(),
            Entry::Vacant(entry) => {
                let key = Key::new(len.try_into().unwrap());
                entry.insert(key);
                self.key_to_ty.insert(key, ty);
                key
            }
        }
        // self.try_intern(ty)
        //     .expect("type interner error: could not get or intern string")
    }

    // pub fn try_intern(&mut self, ty: TypeKind) -> Result<Key, ()> {
    // let hash = hash_type(&self.hasher, &ty);
    // let key = match get_type_entry_mut(&mut self.map, &self.types, hash, &ty) {
    //     RawEntryMut::Occupied(entry) => *entry.into_key(),
    //     RawEntryMut::Vacant(entry) => {
    //         let key = Key::new(self.types.len().try_into().unwrap());
    //         self.types.push(ty);
    //         insert_type(entry, &self.types, &self.hasher, hash, key);
    //         key
    //     }
    // };
    // Ok(key)
    // }

    pub fn resolve(&self, key: Key) -> &TypeKind {
        unsafe {
            assert!((key.0 as usize) < self.ty_to_key.len());
            assert!((key.0 as usize) < self.key_to_ty.len());
            self.key_to_ty.get(&key).unwrap()
        }
    }
}

impl Display for Interner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}\n  {}",
            "Type Interner".green(),
            self.key_to_ty
                .iter()
                .enumerate()
                .format_with("\n  ", |(idx, (_, ty)), f| f(&format_args!(
                    "{} {} {ty}",
                    idx.blue(),
                    "->".purple()
                )))
        )
    }
}

#[cfg(test)]
mod tests {
    use lasso::ThreadedRodeo;
    use once_cell::sync::Lazy;
    use tinyvec::tiny_vec;

    use crate::{intern::Key, ConcreteKind, TypeKind};

    use super::Interner;

    static INTERNER: Lazy<ThreadedRodeo> = Lazy::new(ThreadedRodeo::new);

    #[test]
    fn same_key_for_same_type() {
        let mut interner = Interner::new();
        let k0 = interner.intern(path_kind!(&INTERNER, "i8"));
        let k1 = interner.intern(path_kind!(&INTERNER, "i8"));
        assert_eq!(k0, k1);
    }

    #[test]
    fn diff_keys_for_diff_types() {
        let mut interner = Interner::new();
        let k0 = interner.intern(path_kind!(&INTERNER, "i8"));
        let k1 = interner.intern(path_kind!(&INTERNER, "i16"));
        assert_ne!(k0, k1);
    }

    #[test]
    fn preinterned() {
        let mut interner = Interner::with_preinterned(&INTERNER);
        let pre_intern_size_a = interner.ty_to_key.len();
        let pre_intern_size_b = interner.key_to_ty.len();
        assert_eq!(pre_intern_size_a, pre_intern_size_b);
        let i8_key = interner.intern(path_kind!(&INTERNER, "i8"));
        let post_intern_size_a = interner.ty_to_key.len();
        let post_intern_size_b = interner.key_to_ty.len();
        assert_eq!(post_intern_size_a, post_intern_size_b);
        assert_eq!(pre_intern_size_a, post_intern_size_a);
        assert_eq!(i8_key, Key::new(4));
    }
}

// use std::{
//     collections::hash_map::RandomState,
//     fmt::Display,
//     hash::{BuildHasher, Hash, Hasher},
// };

// use hashbrown::{
//     hash_map::{RawEntryMut, RawVacantEntryMut},
//     HashMap,
// };
// use itertools::Itertools;
// use lasso::ThreadedRodeo;
// use owo_colors::OwoColorize;
// use tinyvec::tiny_vec;

// use crate::{ConcreteKind, TypeId, TypeKind};

// #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
// pub struct Key(u32);

// impl Key {
//     pub(crate) fn new(key: u32) -> Self {
//         Self(key)
//     }

//     pub(crate) fn into_type_id(&self) -> TypeId {
//         TypeId::new(self.0 as usize)
//     }
// }

// macro_rules! path_kind {
//     ($interner:expr, $path:expr) => {
//         TypeKind::Concrete(ConcreteKind::Path(
//             tiny_vec!($interner.get_or_intern($path)),
//         ))
//     };
// }

// // macro_rules! insert_with_precomputed_hash {
// //     ($map:expr, $hash:expr, $key:expr) => {
// //         $map.raw_entry_mut()
// //             .from_hash($hash, |key: &Key| key.0 as u64 == $hash)
// //             .insert(Key::new($key), ());
// //     };
// // }

// macro_rules! insert_with_precomputed_hash {
//     ($string_interner:expr, $hasher:expr, $path:expr, $map:expr, $types:expr) => {
//         let ty = path_kind!($string_interner, "u8");
//         let hash = hash_type($hasher, &ty);
//         match get_type_entry_mut($map, $types, hash, &ty) {
//             RawEntryMut::Occupied(_) => unreachable!(),
//             RawEntryMut::Vacant(entry) => {
//                 let key = Key::new($types.len().try_into().unwrap());
//                 $types.push(ty);
//                 insert_type(entry, $types, $hasher, hash, key);
//                 key
//             }
//         };
//     };
// }

// #[derive(Debug)]
// pub(crate) struct Interner<H = RandomState> {
//     map: HashMap<Key, (), ()>,
//     types: Vec<TypeKind>,
//     hasher: H,
// }

// impl Interner {
//     pub fn new() -> Self {
//         let hasher = RandomState::new();
//         Self {
//             map: HashMap::with_hasher(()),
//             types: Vec::new(),
//             hasher,
//         }
//     }

//     pub fn with_preinterned(string_interner: &'static ThreadedRodeo) -> Self {
//         let hasher = RandomState::new();
//         let mut map = HashMap::with_capacity_and_hasher(9, ());
//         let mut types = Vec::with_capacity(10);
//         insert_with_precomputed_hash!(&string_interner, &hasher, "u8", &mut map, &mut types);
//         insert_with_precomputed_hash!(&string_interner, &hasher, "u16", &mut map, &mut types);
//         // insert_with_precomputed_hash!(&string_interner, &hasher, "u32", &mut map, &mut types);
//         // insert_with_precomputed_hash!(&string_interner, &hasher, "u64", &mut map, &mut types);
//         // insert_with_precomputed_hash!(&string_interner, &hasher, "i8", &mut map, &mut types);
//         // insert_with_precomputed_hash!(&string_interner, &hasher, "i16", &mut map, &mut types);
//         // insert_with_precomputed_hash!(&string_interner, &hasher, "i32", &mut map, &mut types);
//         // insert_with_precomputed_hash!(&string_interner, &hasher, "i64", &mut map, &mut types);
//         // insert_with_precomputed_hash!(&string_interner, &hasher, "f32", &mut map, &mut types);
//         // insert_with_precomputed_hash!(&string_interner, &hasher, "f64", &mut map, &mut types);
//         println!("{:#?}", map);

//         Self { map, types, hasher }
//     }

//     pub fn intern(&mut self, ty: TypeKind) -> Key {
//         self.try_intern(ty)
//             .expect("type interner error: could not get or intern string")
//     }

//     pub fn try_intern(&mut self, ty: TypeKind) -> Result<Key, ()> {
//         let hash = hash_type(&self.hasher, &ty);
//         let key = match get_type_entry_mut(&mut self.map, &self.types, hash, &ty) {
//             RawEntryMut::Occupied(entry) => *entry.into_key(),
//             RawEntryMut::Vacant(entry) => {
//                 let key = Key::new(self.types.len().try_into().unwrap());
//                 self.types.push(ty);
//                 insert_type(entry, &self.types, &self.hasher, hash, key);
//                 key
//             }
//         };
//         Ok(key)
//     }

//     pub fn resolve(&self, key: Key) -> &TypeKind {
//         unsafe {
//             assert!((key.0 as usize) < self.types.len());
//             self.types.get_unchecked(key.0 as usize)
//         }
//     }
// }

// #[inline]
// fn get_type_entry_mut<'a>(
//     map: &'a mut HashMap<Key, (), ()>,
//     types: &[TypeKind],
//     hash: u64,
//     target: &TypeKind,
// ) -> RawEntryMut<'a, Key, (), ()> {
//     map.raw_entry_mut().from_hash(hash, |key| {
//         // Safety: The index given by `key` will be in bounds of the types vector
//         let key_type: &TypeKind = unsafe { types.get_unchecked(key.0 as usize) };
//         target == key_type
//     })
// }

// #[inline]
// fn hash_type<H>(hasher: &H, ty: &TypeKind) -> u64
// where
//     H: BuildHasher,
// {
//     let mut state = hasher.build_hasher();
//     ty.hash(&mut state);
//     state.finish()
// }

// #[inline]
// fn insert_type(
//     entry: RawVacantEntryMut<Key, (), ()>,
//     types: &[TypeKind],
//     hasher: &RandomState,
//     hash: u64,
//     key: Key,
// ) {
//     entry.insert_with_hasher(hash, key, (), |key| {
//         // Safety: The index given by `key` will be in bounds of the types vector
//         let key_type: &TypeKind = unsafe { types.get_unchecked(key.0 as usize) };
//         // Insert the string with the given hash
//         hash_type(hasher, key_type)
//     });
// }

// impl Display for Interner {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(
//             f,
//             "{}\n  {}",
//             "Type Interner".green(),
//             self.types
//                 .iter()
//                 .enumerate()
//                 .format_with("\n  ", |(idx, ty), f| f(&format_args!(
//                     "{} {} {ty}",
//                     idx.blue(),
//                     "->".purple()
//                 )))
//         )
//     }
// }

// #[cfg(test)]
// mod tests {
//     use lasso::ThreadedRodeo;
//     use once_cell::sync::Lazy;
//     use tinyvec::tiny_vec;

//     use crate::{intern::Key, ConcreteKind, TypeKind};

//     use super::Interner;

//     static INTERNER: Lazy<ThreadedRodeo> = Lazy::new(ThreadedRodeo::new);

//     #[test]
//     fn same_key_for_same_type() {
//         let mut interner = Interner::new();
//         let k0 = interner.intern(path_kind!(&INTERNER, "i8"));
//         let k1 = interner.intern(path_kind!(&INTERNER, "i8"));
//         assert_eq!(k0, k1);
//     }

//     #[test]
//     fn diff_keys_for_diff_types() {
//         let mut interner = Interner::new();
//         let k0 = interner.intern(path_kind!(&INTERNER, "i8"));
//         let k1 = interner.intern(path_kind!(&INTERNER, "i16"));
//         assert_ne!(k0, k1);
//     }

//     #[test]
//     fn preinterned() {
//         let mut interner = Interner::with_preinterned(&INTERNER);
//         let k0 = interner.intern(path_kind!(&INTERNER, "u8"));
//         assert_eq!(k0, Key::new(0));
//     }
// }
