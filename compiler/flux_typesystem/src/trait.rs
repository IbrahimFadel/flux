use flux_id::id;
use flux_span::Word;

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub struct TraitRestriction {
    pub absolute_path: Vec<Word>,
    pub args: Vec<id::Ty>,
}

impl TraitRestriction {
    pub fn new(absolute_path: Vec<Word>, args: Vec<id::Ty>) -> Self {
        Self {
            absolute_path,
            args,
        }
    }
}
