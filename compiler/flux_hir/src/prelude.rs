use std::sync::OnceLock;

use flux_util::{Interner, Word};

pub(super) static PRELUDE_SRC: &'static str = include_str!("prelude/prelude.flx");

static MOD_NAME: OnceLock<Word> = OnceLock::new();

pub fn prelude_name(interner: &'static Interner) -> Word {
    interner.get_or_intern_static("prelude")
}
