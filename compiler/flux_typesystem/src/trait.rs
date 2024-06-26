use flux_span::Word;

use crate::r#type::TypeId;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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
