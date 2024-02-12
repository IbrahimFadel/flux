use flux_span::Word;

use crate::r#type::TypeId;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TraitRestriction {
    pub absolute_path: Word,
    pub args: Vec<TypeId>,
}

impl TraitRestriction {
    pub fn new(absolute_path: Word, args: Vec<TypeId>) -> Self {
        Self {
            absolute_path,
            args,
        }
    }
}
