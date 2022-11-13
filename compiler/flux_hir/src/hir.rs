use std::collections::HashSet;
extern crate flux_proc_macros;
use flux_proc_macros::Locatable;
use flux_span::{Span, Spanned, WithSpan};
use flux_syntax::SyntaxToken;
use flux_typesystem::TypeId;
use itertools::Itertools;
use la_arena::Idx;
use lasso::{Spur, ThreadedRodeo};
use tinyvec::TinyVec;

pub type Name = Spanned<Spur>;

#[derive(Debug)]
pub enum Visibility {
    Private,
    Public,
}

pub type FnDeclFirstPass = (
    Name,
    Spanned<GenericParamList>,
    Spanned<ParamList>,
    TypeIdx,
    WhereClause,
);

#[derive(Debug, Locatable)]
pub struct FnDecl {
    name: Name,
    param_list: Spanned<ParamList>,
    return_ty: TypeIdx,
    where_clause: WhereClause,
    body: ExprIdx,
}

impl FnDecl {
    pub fn new(
        name: Name,
        param_list: Spanned<ParamList>,
        return_ty: TypeIdx,
        where_clause: WhereClause,
        body: ExprIdx,
    ) -> Self {
        Self {
            name,
            param_list,
            return_ty,
            where_clause,
            body,
        }
    }
}

#[derive(Debug, Locatable)]
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
    pub ty: TypeIdx,
}

pub type ExprIdx = Idx<Spanned<Expr>>;

#[derive(Debug, Locatable)]
pub enum Expr {
    Path(Path),
    Block(Block),
    Int(Int),
    Float(Float),
    Call(Call),
    Struct(Struct),
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
pub struct Struct {
    path: Path,
    args: Vec<TypeIdx>,
    fields: Vec<StructExprFieldAssignment>,
}

impl Struct {
    pub fn new(path: Path, args: Vec<TypeIdx>, fields: Vec<StructExprFieldAssignment>) -> Self {
        Self { path, args, fields }
    }
}

#[derive(Debug)]
pub struct StructExprFieldAssignment((Name, ExprIdx));

impl StructExprFieldAssignment {
    pub fn new(name: Name, val: ExprIdx) -> Self {
        Self((name, val))
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
    pub ty: TypeIdx,
    pub value: ExprIdx,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Locatable)]
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
}

#[derive(Debug, Clone, Locatable)]
pub enum Type {
    Path(Path, Vec<TypeIdx>),
    Tuple(TinyVec<[TypeId; 2]>),
    Generic(Spur),
    Error,
}

pub type TypeIdx = Idx<Spanned<Type>>;

#[derive(Debug)]
pub struct StructDecl {
    name: Name,
    where_clause: WhereClause,
    field_list: Spanned<StructFieldList>,
}

impl StructDecl {
    pub fn new(
        name: Name,
        where_clause: WhereClause,
        field_list: Spanned<StructFieldList>,
    ) -> Self {
        Self {
            name,
            where_clause,
            field_list,
        }
    }
}

#[derive(Debug, Locatable)]
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

    pub fn iter(&self) -> impl Iterator<Item = &Name> {
        self.0.iter()
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
    args: Vec<TypeIdx>,
}

impl TypeBound {
    pub fn new(name: Name) -> Self {
        Self { name, args: vec![] }
    }

    pub fn with_args(name: Name, args: Vec<TypeIdx>) -> Self {
        Self { name, args }
    }
}

#[derive(Debug, Locatable)]
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

#[derive(Debug)]
pub struct StructField {
    pub name: Name,
    pub ty: TypeIdx,
}

impl StructField {
    pub fn new(name: Name, ty: TypeIdx) -> Self {
        Self { name, ty }
    }
}

#[derive(Debug)]
pub struct TraitDecl {
    name: Name,
    where_clause: WhereClause,
    associated_types: Vec<AssociatedType>,
    methods: Vec<TraitMethod>,
}

impl TraitDecl {
    pub fn new(
        name: Name,
        where_clause: WhereClause,
        associated_types: Vec<AssociatedType>,
        methods: Vec<TraitMethod>,
    ) -> Self {
        Self {
            name,
            where_clause,
            associated_types,
            methods,
        }
    }
}

pub type AssociatedType = Name;

#[derive(Debug)]
pub struct TraitMethod {
    name: Name,
    param_list: Spanned<ParamList>,
    return_ty: TypeIdx,
    where_clause: WhereClause,
}

impl TraitMethod {
    pub fn new(
        name: Name,
        param_list: Spanned<ParamList>,
        return_ty: TypeIdx,
        where_clause: WhereClause,
    ) -> Self {
        Self {
            name,
            param_list,
            return_ty,
            where_clause,
        }
    }
}

#[derive(Debug)]
pub struct EnumDecl {
    name: Name,
    where_clause: WhereClause,
    variants: Vec<EnumVariant>,
}

impl EnumDecl {
    pub fn new(name: Name, where_clause: WhereClause, variants: Vec<EnumVariant>) -> Self {
        Self {
            name,
            where_clause,
            variants,
        }
    }
}

#[derive(Debug)]
pub struct EnumVariant((Name, Option<TypeIdx>));

impl EnumVariant {
    pub fn new(name: Name, ty: Option<TypeIdx>) -> Self {
        Self((name, ty))
    }
}

#[derive(Debug)]
pub struct ApplyDecl {
    trt: Option<(Path, Vec<TypeIdx>)>,
    to_ty: TypeIdx,
    where_clause: WhereClause,
    associated_types: Vec<AssociatedTypeDef>,
    methods: Vec<FnDecl>,
}

impl ApplyDecl {
    pub fn new(
        trt: Option<(Path, Vec<TypeIdx>)>,
        to_ty: TypeIdx,
        where_clause: WhereClause,
        associated_types: Vec<AssociatedTypeDef>,
        methods: Vec<FnDecl>,
    ) -> Self {
        Self {
            trt,
            to_ty,
            where_clause,
            associated_types,
            methods,
        }
    }
}

#[derive(Debug)]
pub struct AssociatedTypeDef((Name, TypeIdx));

impl AssociatedTypeDef {
    pub fn new(name: Name, ty: TypeIdx) -> Self {
        Self((name, ty))
    }
}

#[derive(Debug)]
pub struct UseDecl((Path, Option<Name>));

impl UseDecl {
    pub fn new(path: Path, alias: Option<Name>) -> Self {
        Self((path, alias))
    }
}

#[derive(Debug)]
pub struct ModDecl(pub (Visibility, Name));

impl ModDecl {
    pub fn new(visibility: Visibility, name: Name) -> Self {
        Self((visibility, name))
    }
}
