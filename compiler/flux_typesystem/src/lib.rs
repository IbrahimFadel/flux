pub mod diagnostics;
mod fmt;
mod resolve;
mod scope;
mod tenv;
mod r#trait;
mod r#type;
mod unify;

use std::sync::OnceLock;

use flux_util::{Interner, Path, Word};
pub use r#trait::{ThisCtx, TraitApplication};
pub use r#type::{
    ConcreteKind, FnSignature, Restriction, TraitRestriction, Type, TypeKind, Typed, WithType,
};
pub use resolve::TraitResolver;
pub use tenv::TEnv;

static INT_PATHS: OnceLock<[Path<Word, Type>; 8]> = OnceLock::new();

pub fn int_paths(interner: &'static Interner) -> &'static [Path<Word, Type>; 8] {
    INT_PATHS.get_or_init(|| {
        [
            interner.get_or_intern_static("s64"),
            interner.get_or_intern_static("s32"),
            interner.get_or_intern_static("s16"),
            interner.get_or_intern_static("s8"),
            interner.get_or_intern_static("u64"),
            interner.get_or_intern_static("u32"),
            interner.get_or_intern_static("u16"),
            interner.get_or_intern_static("u8"),
        ]
        .map(|name| Path::new(vec![name], vec![]))
    })
}
