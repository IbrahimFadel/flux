use flux_span::Word;
use flux_syntax::ast;
use la_arena::{Arena, Idx};

// Track Spans
// pub struct SourceMap {}

// macro_rules! {
//     () => {

//     };
// }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Visibility {
    Private,
    Public,
}

#[salsa::tracked]
pub struct Function {
    #[id]
    pub name: Word,
    pub visibility: Visibility,
    pub generic_params: GenericParams,
}

#[derive(Clone, PartialEq, Eq, Default, Debug, Hash)]
pub struct GenericParams {
    pub types: Arena<Word>,
    pub where_predicates: Vec<WherePredicate>,
}

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub struct WherePredicate {
    pub ty: Idx<Word>,
    pub name: Word,
    pub bound: Path,
}

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub struct Path {
    pub segments: Vec<Word>,
    // pub generic_args: Vec<TypeId>,
}
