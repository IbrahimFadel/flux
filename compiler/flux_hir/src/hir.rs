use std::collections::HashSet;

use flux_span::{Span, Spanned};
use flux_syntax::SyntaxToken;
use flux_typesystem::TypeId;
use itertools::Itertools;
use la_arena::Idx;
use lasso::{Spur, ThreadedRodeo};
use tinyvec::TinyVec;

struct Module {
    functions: Vec<FnDecl>,
}

pub type Name = Spanned<Spur>;

#[derive(Debug)]
pub struct FnDecl {
    name: Name,
    generic_param_list: GenericParamList,
    param_list: Spanned<ParamList>,
    return_ty: Spanned<Type>,
    where_clause: WhereClause,
    body: ExprIdx,
}

impl FnDecl {
    pub fn new(
        name: Name,
        generic_param_list: GenericParamList,
        param_list: Spanned<ParamList>,
        return_ty: Spanned<Type>,
        where_clause: WhereClause,
        body: ExprIdx,
    ) -> Self {
        Self {
            name,
            generic_param_list,
            param_list,
            return_ty,
            where_clause,
            body,
        }
    }
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Path(Vec<Name>);

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
        Self(segments.collect())
    }

    /// Build a default path
    ///
    /// This is used for poisoned values
    pub fn poisoned(span: Span) -> Path {
        Self(vec![])
        // Self(Spanned::new(tiny_vec!(), span))
    }

    /// Get the `TinyVec` of `Spanned<Spur>`s that represent the [`Path`]
    pub fn get_spurs(&self) -> Vec<Name> {
        self.0.clone()
    }

    /// Get the `TinyVec` of `Spur`s that represent the [`Path`]
    pub fn get_unspanned_spurs(&self) -> Vec<Spur> {
        self.0.iter().map(|name| name.inner).collect()
    }

    /// Format the path to a string
    pub fn to_string(&self, interner: &'static ThreadedRodeo) -> String {
        self.0.iter().map(|spur| interner.resolve(spur)).join("::")
    }

    pub fn nth(&self, n: usize) -> Option<&Name> {
        self.0.get(n)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }
}

#[derive(Debug, Clone)]
pub enum Type {
    Path(Path),
    Tuple(TinyVec<[TypeId; 2]>),
    Generic,
    Error,
}

#[derive(Debug)]
pub struct StructDecl {
    name: Name,
    generic_param_list: GenericParamList,
    where_clause: WhereClause,
    field_list: StructFieldList,
}

impl StructDecl {
    pub fn new(
        name: Name,
        generic_param_list: GenericParamList,
        where_clause: WhereClause,
        field_list: StructFieldList,
    ) -> Self {
        Self {
            name,
            generic_param_list,
            where_clause,
            field_list,
        }
    }
}

#[derive(Debug)]
pub struct GenericParamList(HashSet<Name>);

impl GenericParamList {
    pub fn empty() -> Self {
        Self(HashSet::new())
    }

    pub fn new(params: HashSet<Name>) -> Self {
        Self(params)
    }

    pub fn get(&self, path: &Name) -> bool {
        self.0.contains(path)
    }
}

#[derive(Debug)]
pub struct WhereClause(Vec<WherePredicate>);

impl WhereClause {
    pub const EMPTY: Self = Self(vec![]);

    pub fn new(predicates: Vec<WherePredicate>) -> Self {
        Self(predicates)
    }
}

#[derive(Debug)]
pub struct WherePredicate {
    generic: Name,
    trait_restrictions: TypeBoundList,
}

impl WherePredicate {
    pub fn new(generic: Name) -> Self {
        Self {
            generic,
            trait_restrictions: TypeBoundList::EMPTY,
        }
    }

    pub fn with_trait_restrictions(generic: Name, trait_restrictions: TypeBoundList) -> Self {
        Self {
            generic,
            trait_restrictions,
        }
    }
}

#[derive(Debug)]
pub struct TypeBoundList(Vec<TypeBound>);

impl TypeBoundList {
    pub const EMPTY: Self = Self(vec![]);

    pub fn new(bounds: Vec<TypeBound>) -> Self {
        Self(bounds)
    }
}

#[derive(Debug)]
pub struct TypeBound {
    name: Name,
    args: Vec<Spanned<Type>>,
}

impl TypeBound {
    pub fn new(name: Name) -> Self {
        Self { name, args: vec![] }
    }

    pub fn with_args(name: Name, args: Vec<Spanned<Type>>) -> Self {
        Self { name, args }
    }
}

#[derive(Debug)]
pub struct StructFieldList(Vec<StructField>);

impl StructFieldList {
    pub fn empty() -> Self {
        Self(vec![])
    }

    pub fn new(fields: Vec<StructField>) -> Self {
        Self(fields)
    }
}

#[derive(Debug)]
pub struct StructField {
    name: Name,
    ty: Spanned<Type>,
}

impl StructField {
    pub fn new(name: Name, ty: Spanned<Type>) -> Self {
        Self { name, ty }
    }
}
