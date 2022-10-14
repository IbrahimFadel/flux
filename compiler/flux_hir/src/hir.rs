use flux_span::{Span, Spanned};
use lasso::{Spur, ThreadedRodeo};
use text_size::TextRange;
use tinyvec::TinyVec;

struct Module {
    functions: Vec<FnDecl>,
}

#[derive(Debug)]
pub struct FnDecl {
    pub name: Spanned<Spur>,
    pub param_list: Spanned<ParamList>,
    // return_ty: Type,
}

#[derive(Debug)]
pub struct ParamList(Vec<Param>);

impl ParamList {
    pub fn new(params: Vec<Param>) -> Self {
        Self(params)
    }
}

#[derive(Debug)]
pub struct Param {
    pub name: Spanned<Spur>,
    pub ty: Spanned<Type>,
}

pub enum Expr {
    Path(Path),
    Error,
}

#[derive(Debug)]
pub struct Path(Spanned<TinyVec<[Spur; 2]>>);

impl Path {
    /// Builds a [`Path`] from an iterator over it's segements, `Spanned<Spur>`
    ///
    /// Panics if the [`Path`] has no segments, which is considered an ICE
    pub fn from_segments(segments: impl Iterator<Item = Spanned<Spur>>) -> Self {
        let path = Spanned::span_iter(segments).expect("internal compiler error: empty path");
        Self(path)
    }

    pub fn get_spurs(&self) -> TinyVec<[Spur; 2]> {
        self.0.inner.clone()
    }
}

#[derive(Debug)]
pub enum Type {
    Path(Path),
    Error,
}
