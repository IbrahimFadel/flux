use std::{borrow::Borrow, ops::Deref};

use flux_diagnostics::ice;
use flux_proc_macros::Locatable;
use flux_span::{Spanned, ToSpan, WithSpan};
use flux_syntax::ast;
use hashbrown::HashSet;
use itertools::Itertools;
use la_arena::{Arena, Idx, RawIdx};
use lasso::{Spur, ThreadedRodeo};
use text_size::{TextRange, TextSize};

use crate::FunctionId;

#[cfg(test)]
mod pp;

pub type Name = Spanned<Spur>;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Locatable)]
pub enum Visibility {
    Private,
    Public,
}

#[derive(Debug, Clone, Locatable)]
pub enum Item {
    Apply(Apply),
    Enum(Enum),
    Function(Function),
    Struct(Struct),
    Trait(Trait),
}

impl TryFrom<Item> for Function {
    type Error = ();
    fn try_from(value: Item) -> Result<Self, Self::Error> {
        match value {
            Item::Function(f) => Ok(f),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Apply {
    pub visibility: Spanned<Visibility>,
    pub generic_params: Spanned<GenericParams>,
    pub trt: Option<Spanned<Path>>,
    pub ty: TypeIdx,
    pub assoc_types: Vec<(Name, TypeIdx)>,
    pub methods: Spanned<Vec<FunctionId>>,
}

#[derive(Debug, Clone)]
pub struct Enum {}

#[derive(Debug, Clone, Locatable)]
pub struct Function {
    pub visibility: Spanned<Visibility>,
    pub name: Name,
    pub generic_params: Spanned<GenericParams>,
    pub params: Spanned<Params>,
    pub ret_ty: TypeIdx,
    pub ast: Option<ast::FnDecl>, // Trait methods will use this `Function` type but won't have the ast field
}

#[derive(Debug, Clone)]
pub struct Mod {
    pub visibility: Visibility,
    pub name: Name,
}

#[derive(Debug, Clone, Locatable)]
pub struct Struct {
    pub visibility: Spanned<Visibility>,
    pub name: Name,
    pub generic_params: Spanned<GenericParams>,
    pub fields: StructFields,
}

#[derive(Clone, PartialEq, Eq, Default, Debug, Hash, Locatable)]
pub struct StructFields {
    pub fields: Vec<StructField>,
}

impl StructFields {
    pub fn poisoned() -> Self {
        Self { fields: vec![] }
    }
}

#[derive(Clone, PartialEq, Eq, Debug, Hash, Locatable)]
pub struct StructField {
    pub name: Name,
    pub ty: TypeIdx,
}

#[derive(Debug, Clone)]
pub struct Trait {
    pub visibility: Spanned<Visibility>,
    pub name: Name,
    pub generic_params: Spanned<GenericParams>,
    pub assoc_types: Vec<Name>,
    pub methods: Spanned<Vec<FunctionId>>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Use {
    pub visibility: Visibility,
    pub path: Spanned<Path>,
    pub alias: Option<Name>,
}

#[derive(Debug, Clone, Locatable)]
pub struct Params(Vec<Param>);

impl Params {
    pub fn new(params: Vec<Param>) -> Self {
        Self(params)
    }
}

impl Deref for Params {
    type Target = Vec<Param>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone)]
pub struct Param {
    pub name: Name,
    pub ty: TypeIdx,
}

#[derive(Clone, PartialEq, Eq, Default, Debug, Hash, Locatable)]
pub struct GenericParams {
    pub types: Arena<Spanned<Spur>>,
    pub where_predicates: WherePredicates,
}

impl GenericParams {
    pub fn new() -> Self {
        Self {
            types: Arena::new(),
            where_predicates: WherePredicates(vec![]),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            types: Arena::with_capacity(capacity),
            where_predicates: WherePredicates(Vec::with_capacity(capacity)),
        }
    }

    pub fn poisoned() -> Self {
        Self {
            types: Arena::default(),
            where_predicates: WherePredicates(Vec::with_capacity(0)),
        }
    }

    pub fn invalid_idx(&self) -> Idx<Spanned<Spur>> {
        Idx::from_raw(RawIdx::from(self.types.len() as u32))
    }

    /*
     * Combine two generic parameter lists
     *
     * Duplicate generics are considered an error, but will still return the combined list along with said duplicates
     */
    pub fn combine(a: &GenericParams, b: &GenericParams) -> Result<Self, (Self, Vec<Spur>)> {
        let a_names: HashSet<Spur> = a.types.iter().map(|(_, name)| name.inner).collect();
        let b_names: HashSet<Spur> = b.types.iter().map(|(_, name)| name.inner).collect();
        let duplicates: Vec<Spur> = a_names.intersection(&b_names).copied().collect();
        let mut generic_params = GenericParams::with_capacity(duplicates.len());
        let combined = a_names.union(&b_names);
        combined.for_each(|name| {
            let predicate = a
                .where_predicates
                .0
                .iter()
                .find(|predicate| predicate.name.inner == *name)
                .or_else(|| {
                    b.where_predicates
                        .0
                        .iter()
                        .find(|predicate| predicate.name.inner == *name)
                });

            let (_, name) = a
                .types
                .iter()
                .find(|(_, n)| n.inner == *name)
                .or_else(|| b.types.iter().find(|(_, n)| n.inner == *name))
                .unwrap_or_else(|| ice("could not find generic parameter when combining"));

            let new_idx = generic_params.types.alloc(name.clone());
            if let Some(predicate) = predicate {
                generic_params.where_predicates.0.push(WherePredicate {
                    ty: new_idx,
                    name: predicate.name.clone(),
                    bound: predicate.bound.clone(),
                });
            }
        });
        match duplicates.is_empty() {
            true => Ok(generic_params),
            false => Err((generic_params, duplicates)),
        }
    }

    pub fn unused<S: Borrow<Spur>>(&self, mut used: impl Iterator<Item = S>) -> Vec<Spur> {
        let mut unused = vec![];
        for (_, generic) in self.types.iter() {
            if used.find(|name| name.borrow() == &generic.inner).is_none() {
                unused.push(generic.inner)
            }
        }
        unused
    }
}

#[derive(Clone, PartialEq, Eq, Default, Debug, Hash, Locatable)]
pub struct WherePredicates(pub Vec<WherePredicate>);

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub struct WherePredicate {
    pub ty: Idx<Spanned<Spur>>,
    pub name: Spanned<Spur>,
    pub bound: Spanned<Path>,
}

#[derive(Clone, PartialEq, Eq, Debug, Hash, Locatable)]
pub struct Path {
    pub segments: Vec<Spur>,
    pub generic_args: Vec<TypeIdx>,
}

impl Path {
    pub fn new(segments: Vec<Spur>, generic_args: Vec<TypeIdx>) -> Self {
        Self {
            segments,
            generic_args,
        }
    }

    pub fn poisoned() -> Self {
        Self {
            segments: vec![],
            generic_args: vec![],
        }
    }

    pub fn from_static_str(s: &'static str, string_interner: &'static ThreadedRodeo) -> Self {
        Self {
            segments: s
                .split("::")
                .map(|s| string_interner.get_or_intern_static(s))
                .collect(),
            generic_args: vec![],
        }
    }

    pub fn get_segments(&self) -> impl Iterator<Item = &Spur> {
        self.segments.iter()
    }

    pub fn to_spur(&self, string_interner: &'static ThreadedRodeo) -> Spur {
        let s = self
            .segments
            .iter()
            .map(|spur| string_interner.resolve(spur))
            .join("::");
        string_interner.get_or_intern(s)
    }

    pub fn to_string(&self, string_interner: &'static ThreadedRodeo) -> String {
        self.segments
            .iter()
            .map(|spur| string_interner.resolve(spur))
            .join("::")
    }

    pub fn spanned_segment(
        path: &Spanned<Path>,
        idx: usize,
        string_interner: &'static ThreadedRodeo,
    ) -> Option<Spanned<Spur>> {
        let mut iter = path.get_segments().peekable();
        let mut start: usize = path.span.range.start().into();
        for _ in 0..idx {
            start += iter
                .next()
                .map(|spur| string_interner.resolve(spur).len())?;
            if iter.peek().is_some() {
                start += 2; // "::"
            }
        }
        let (spur, end) = iter
            .next()
            .map(|spur| (spur, string_interner.resolve(spur).len()))?;
        let end = start + end;

        Some(Spanned::new(
            *spur,
            TextRange::new(TextSize::from(start as u32), TextSize::from(end as u32)).to_span(),
        ))
    }
}

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub struct ExprIdx(Idx<Spanned<Expr>>);

impl ExprIdx {
    pub fn new(idx: Idx<Spanned<Expr>>) -> Self {
        Self(idx)
    }

    pub fn raw(&self) -> Idx<Spanned<Expr>> {
        self.0
    }
}

impl From<Idx<Spanned<Expr>>> for ExprIdx {
    fn from(value: Idx<Spanned<Expr>>) -> Self {
        ExprIdx(value)
    }
}

impl TypeIdx {
    pub fn new(idx: Idx<Spanned<Type>>) -> Self {
        Self(idx)
    }

    pub fn raw(&self) -> Idx<Spanned<Type>> {
        self.0
    }
}

impl From<Idx<Spanned<Type>>> for TypeIdx {
    fn from(value: Idx<Spanned<Type>>) -> Self {
        TypeIdx(value)
    }
}

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub struct TypeIdx(Idx<Spanned<Type>>);

#[derive(Clone, PartialEq, Eq, Debug, Hash, Locatable)]
pub enum Type {
    Array(TypeIdx, u32),
    Generic(Spur, Vec<Spanned<Path>>),
    Path(Path),
    Ptr(TypeIdx),
    Tuple(Vec<TypeIdx>),
    Unknown,
}

#[derive(Clone, PartialEq, Debug, Locatable)]
pub enum Expr {
    Block(Block),
    Call(Call),
    Float(f64),
    Int(u64),
    // Tuple(Vec<ExprIdx>),
    Path(Path),
    Let(Let),
    Struct(StructExpr),
    MemberAccess(MemberAccess),
    Poisoned,
}

impl TryFrom<Expr> for Path {
    type Error = ();

    fn try_from(value: Expr) -> Result<Self, Self::Error> {
        match value {
            Expr::Path(path) => Ok(path),
            _ => Err(()),
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub struct Block {
    pub exprs: Vec<ExprIdx>,
}

#[derive(Clone, PartialEq, Eq, Debug, Hash, Locatable)]
pub struct Call {
    pub callee: ExprIdx,
    pub args: Vec<ExprIdx>,
}

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub struct Let {
    pub name: Name,
    pub ty: TypeIdx,
    pub val: ExprIdx,
}

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub struct StructExpr {
    pub path: Spanned<Path>,
    pub fields: Vec<StructExprField>,
}

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub struct StructExprField {
    pub name: Name,
    pub val: ExprIdx,
}

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub struct MemberAccess {
    pub lhs: ExprIdx,
    pub rhs: Name,
}

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub enum MemberAccessKind {
    Field,
    Method,
}
