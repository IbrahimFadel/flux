use std::collections::HashSet;

use flux_proc_macros::Locatable;
use flux_span::{Spanned, WithSpan};
use flux_typesystem::TypeId;
use lasso::Spur;

pub type Name = Spanned<Spur>;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Path;
pub type UseAlias = Name;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Visibility {
    Private,
    Public,
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
}

#[derive(Debug, Clone, Eq, PartialEq, Locatable)]
pub struct WhereClause(Vec<WherePredicate>);

impl WhereClause {
    pub const EMPTY: Self = Self(vec![]);

    pub fn new(predicates: Vec<WherePredicate>) -> Self {
        Self(predicates)
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Locatable)]
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

#[derive(Debug, Clone, Eq, PartialEq, Locatable)]
pub struct TypeBoundList(Vec<TypeBound>);

impl TypeBoundList {
    pub const EMPTY: Self = Self(vec![]);

    pub fn new(bounds: Vec<TypeBound>) -> Self {
        Self(bounds)
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Locatable)]
pub struct TypeBound {
    name: Name,
    args: Vec<TypeId>,
}

impl TypeBound {
    pub fn new(name: Name) -> Self {
        Self { name, args: vec![] }
    }

    pub fn with_args(name: Name, args: Vec<TypeId>) -> Self {
        Self { name, args }
    }
}
