use flux_span::{Span, Spanned};
use flux_syntax::SyntaxToken;
use flux_typesystem::TypeId;
use la_arena::Idx;
use lasso::Spur;
use tinyvec::{tiny_vec, TinyVec};

struct Module {
    functions: Vec<FnDecl>,
}

pub type Name = Spanned<Spur>;

#[derive(Debug)]
pub struct FnDecl {
    pub name: Name,
    pub param_list: Spanned<ParamList>,
    // return_ty: Type,
}

#[derive(Debug)]
pub struct ParamList(Vec<Param>);

impl ParamList {
    pub fn new(params: Vec<Param>) -> Self {
        Self(params)
    }

    pub fn iter(&self) -> impl Iterator<Item = &Param> {
        self.0.iter()
    }
}

#[derive(Debug)]
pub struct Param {
    pub name: Name,
    pub ty: Spanned<Type>,
}

pub type ExprIdx = Idx<Spanned<Expr>>;

#[derive(Debug)]
pub enum Expr {
    Path(Path),
    Block(Block),
    Int(Int),
    Float(Float),
    Call(Call),
    Error,
}

#[derive(Debug)]
pub struct Call {
    pub path: Spanned<Path>,
    pub args: Vec<ExprIdx>,
}

#[derive(Debug)]
pub struct Int(u64);

impl Int {
    pub fn new(int: u64) -> Self {
        Self(int)
    }
}

#[derive(Debug)]
pub struct Float(f64);

impl Float {
    pub fn new(float: f64) -> Self {
        Self(float)
    }
}

#[derive(Debug)]
pub struct Block(Vec<Stmt>);

impl Block {
    pub fn new(stmts: Vec<Stmt>) -> Self {
        Self(stmts)
    }
}

#[derive(Debug)]
pub enum Stmt {
    LetStmt(LetStmt),
    ExprStmt(ExprIdx),
}

#[derive(Debug)]
pub struct LetStmt {
    pub name: Name,
    pub ty: TypeId,
    pub value: ExprIdx,
}

#[derive(Debug, Clone)]
pub struct Path(TinyVec<[Spur; 2]>);

impl Path {
    /// Builds a [`Path`] from an iterator over the [`SyntaxToken`]s that compose it
    ///
    /// Panics if the [`Path`] has no segments, which is considered an ICE
    pub fn from_syntax_tokens<'a>(segments: impl Iterator<Item = &'a SyntaxToken>) -> Self {
        let segments = segments
            .map(|segment| Spanned::new(segment.text_key(), Span::new(segment.text_range())));
        Self::from_segments(segments)
    }

    /// Builds a [`Path`] from an iterator over its segements, `Spanned<Spur>`
    ///
    /// Panics if the [`Path`] has no segments, which is considered an ICE
    pub fn from_segments(segments: impl Iterator<Item = Name>) -> Self {
        Self(segments.map(|name| name.inner).collect())
        // let path = Spanned::span_iter(segments).expect("internal compiler error: empty path");
        // Self(path)
    }

    /// Build a default path
    ///
    /// This is used for poisoned values
    pub fn poisoned(span: Span) -> Path {
        Self(tiny_vec!())
        // Self(Spanned::new(tiny_vec!(), span))
    }

    /// Get the `TinyVec` of `Spur`s that represent the [`Path`]
    pub fn get_spurs(&self) -> TinyVec<[Spur; 2]> {
        self.0.clone()
        // self.0.inner.clone()
    }
}

#[derive(Debug, Clone)]
pub enum Type {
    Path(Path),
    Tuple(TinyVec<[TypeId; 2]>),
    Error,
}
