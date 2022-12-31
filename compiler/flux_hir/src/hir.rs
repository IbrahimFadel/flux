#[derive(Debug, Default)]
pub struct Module {
    pub functions: Arena<FnDecl>,
}

use std::{collections::HashSet, fmt::Display};

use flux_proc_macros::Locatable;
use flux_span::{Spanned, ToSpan, WithSpan};
use flux_syntax::SyntaxToken;
use itertools::Itertools;
use la_arena::{Arena, Idx};
use lasso::{Spur, ThreadedRodeo};

use crate::type_interner::TypeIdx;

pub type Name = Spanned<Spur>;

#[derive(Debug, Clone, Eq, PartialEq, Locatable)]
pub struct ModDecl {
    name: Name,
}

impl ModDecl {
    pub fn new(name: Name) -> Self {
        Self { name }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Locatable)]
pub struct StructDecl {
    visibility: Visibility,
    name: Name,
    generic_param_list: GenericParamList,
    where_clause: WhereClause,
    field_list: StructFieldList,
}

impl StructDecl {
    pub fn new(
        visibility: Visibility,
        name: Name,
        generic_param_list: GenericParamList,
        where_clause: WhereClause,
        field_list: StructFieldList,
    ) -> Self {
        Self {
            visibility,
            name,
            generic_param_list,
            where_clause,
            field_list,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Locatable)]
pub struct FnDecl {
    visibility: Visibility,
    name: Name,
    generic_param_list: GenericParamList,
    params: ParamList,
    ret_type: Spanned<TypeIdx>,
    where_clause: WhereClause,
    body: ExprIdx,
}

impl FnDecl {
    pub fn new(
        visibility: Visibility,
        name: Name,
        generic_param_list: GenericParamList,
        params: ParamList,
        ret_type: Spanned<TypeIdx>,
        where_clause: WhereClause,
        body: ExprIdx,
    ) -> Self {
        Self {
            visibility,
            name,
            generic_param_list,
            params,
            ret_type,
            where_clause,
            body,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Locatable)]
pub struct Path(Vec<Name>);

impl Path {
    /// Builds a [`Path`] from a `&str`
    pub fn from_str_static(s: Spanned<&'static str>, interner: &'static ThreadedRodeo) -> Self {
        Self(vec![s.map(|s| interner.get_or_intern_static(s))])
    }

    /// Builds a [`Path`] from an iterator over the [`SyntaxToken`]s that compose it
    ///
    /// Panics if the [`Path`] has no segments, which is considered an ICE
    pub fn from_syntax_tokens<'a>(segments: impl Iterator<Item = &'a SyntaxToken>) -> Self {
        let segments =
            segments.map(|segment| segment.text_key().at(segment.text_range().to_span()));
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
    pub fn poisoned() -> Path {
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

    pub fn iter(&self) -> impl Iterator<Item = &Spanned<Spur>> {
        self.0.iter()
    }

    /// Joins resolved segments into string separated by `::` then interns full path
    pub fn to_spur(&self, interner: &'static ThreadedRodeo) -> Spur {
        interner.get_or_intern(self.0.iter().map(|spur| interner.resolve(spur)).join("::"))
    }
}

pub type UseAlias = Name;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Visibility {
    Private,
    Public,
}

impl Display for Visibility {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Private => write!(f, "private"),
            Self::Public => write!(f, "public"),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Locatable)]
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

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn iter(&self) -> impl Iterator<Item = &Name> {
        self.0.iter()
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Locatable)]
pub struct WhereClause(Vec<WherePredicate>);

impl WhereClause {
    pub const EMPTY: Self = Self(vec![]);

    pub fn new(predicates: Vec<WherePredicate>) -> Self {
        Self(predicates)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn iter(&self) -> impl Iterator<Item = &WherePredicate> {
        self.0.iter()
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Locatable)]
pub struct WherePredicate {
    pub generic: Name,
    pub trait_restrictions: TypeBoundList,
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

#[derive(Debug, Clone, Eq, PartialEq, Locatable)]
pub struct TypeBoundList(Vec<TypeBound>);

impl TypeBoundList {
    pub const EMPTY: Self = Self(vec![]);

    pub fn new(bounds: Vec<TypeBound>) -> Self {
        Self(bounds)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn iter(&self) -> impl Iterator<Item = &TypeBound> {
        self.0.iter()
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Locatable)]
pub struct TypeBound {
    pub name: Name,
    pub args: Vec<Spanned<TypeIdx>>,
}

impl TypeBound {
    pub fn new(name: Name) -> Self {
        Self { name, args: vec![] }
    }

    pub fn with_args(name: Name, args: Vec<Spanned<TypeIdx>>) -> Self {
        Self { name, args }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Locatable)]
pub enum Type {
    Path(Path, Vec<Spanned<TypeIdx>>),
    Tuple(Vec<Spanned<TypeIdx>>),
    Generic(Spur),
    Unknown,
}

#[derive(Debug, Clone, Eq, PartialEq, Locatable)]
pub struct StructFieldList(Vec<StructField>);

impl StructFieldList {
    pub fn empty() -> Self {
        Self(vec![])
    }

    pub fn new(fields: Vec<StructField>) -> Self {
        Self(fields)
    }

    pub fn iter(&self) -> impl Iterator<Item = &StructField> {
        self.0.iter()
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Locatable)]
pub struct StructField {
    pub name: Name,
    pub ty: Spanned<TypeIdx>,
}

impl StructField {
    pub fn new(name: Name, ty: Spanned<TypeIdx>) -> Self {
        Self { name, ty }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Locatable)]
pub struct ParamList(Vec<Param>);

impl ParamList {
    pub fn new(params: Vec<Param>) -> Self {
        Self(params)
    }

    pub fn empty() -> Self {
        Self(vec![])
    }

    pub fn iter(&self) -> impl Iterator<Item = &Param> {
        self.0.iter()
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Locatable)]
pub struct Param {
    pub name: Name,
    pub ty: Spanned<TypeIdx>,
}

impl Param {
    pub fn new(name: Name, ty: Spanned<TypeIdx>) -> Self {
        Self { name, ty }
    }
}

pub type ExprIdx = Idx<Spanned<Expr>>;

#[derive(Debug, Clone, Eq, PartialEq, Locatable)]
pub enum Expr {
    Block(Block),
    Int(u64),
    Tuple(Vec<ExprIdx>),
    Let(Let),
    Poisoned,
}

#[derive(Debug, Clone, Eq, PartialEq, Locatable)]
pub struct Block(Vec<ExprIdx>);

impl Block {
    pub fn new(exprs: Vec<ExprIdx>) -> Self {
        Self(exprs)
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Locatable)]
pub struct Let {
    name: Name,
    ty: Spanned<TypeIdx>,
    val: ExprIdx,
}

impl Let {
    pub fn new(name: Name, ty: Spanned<TypeIdx>, val: ExprIdx) -> Self {
        Self { name, ty, val }
    }
}
