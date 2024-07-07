use flux_span::Word;

use crate::r#type::TypeId;

#[derive(Debug, Clone)]
pub struct TraitApplication {
    pub assoc_types: Vec<(Word, TypeId)>,
}

impl TraitApplication {
    pub fn new(assoc_types: Vec<(Word, TypeId)>) -> Self {
        Self { assoc_types }
    }
}

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ApplicationId(usize);

impl ApplicationId {
    pub const fn new(id: usize) -> Self {
        Self(id)
    }

    pub fn raw(&self) -> usize {
        self.0
    }
}

#[derive(Debug, Clone)]
pub struct TraitRestriction {
    pub absolute_path: Vec<Word>,
    pub args: Vec<TypeId>,
}

impl TraitRestriction {
    pub fn new(absolute_path: Vec<Word>, args: Vec<TypeId>) -> Self {
        Self {
            absolute_path,
            args,
        }
    }
}
