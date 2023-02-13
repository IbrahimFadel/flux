use std::{collections::HashSet, fmt::Display};

use flux_proc_macros::Locatable;
use flux_span::{FileId, Spanned, ToSpan, WithSpan};
use flux_syntax::{ast, SyntaxToken};
use flux_typesystem::TypeId;
use itertools::Itertools;
use la_arena::{Arena, Idx};
use lasso::{Spur, ThreadedRodeo};

use crate::type_interner::TypeIdx;

pub type Name = Spanned<Spur>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ItemDefinitionId {
    ModuleId(ModuleId),
    FunctionId(FunctionId),
    StructId(StructId),
    TraitId(TraitId),
    UseId(UseId),
}

pub type ModuleId = Idx<Module>;
pub type FunctionId = Idx<FnDecl>;
pub type StructId = Idx<StructDecl>;
pub type TraitId = Idx<TraitDecl>;
pub type UseId = Idx<UseDecl>;

#[derive(Debug)]
pub struct Module {
    pub functions: Arena<FnDecl>,
    pub mods: Arena<ModDecl>,
    pub uses: Arena<UseDecl>,
    pub structs: Arena<StructDecl>,
    pub traits: Arena<TraitDecl>,
    pub exprs: Arena<Spanned<Expr>>,
    pub file_id: FileId,
    pub absolute_path: Vec<Spur>,
}

impl Module {
    pub fn new(file_id: FileId, absolute_path: Vec<Spur>) -> Self {
        Self {
            functions: Arena::new(),
            mods: Arena::new(),
            uses: Arena::new(),
            structs: Arena::new(),
            traits: Arena::new(),
            exprs: Arena::new(),
            file_id,
            absolute_path,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Locatable)]
pub struct ModDecl {
    pub name: Name,
}

impl ModDecl {
    pub fn new(name: Name) -> Self {
        Self { name }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Locatable)]
pub struct UseDecl {
    pub path: Path,
}

impl UseDecl {
    pub fn new(path: Path) -> Self {
        Self { path }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Locatable)]
pub struct StructDecl {
    visibility: Visibility,
    pub name: Name,
    generic_param_list: GenericParamList,
    where_clause: WhereClause,
    pub field_list: Spanned<StructDeclFieldList>,
}

impl StructDecl {
    pub fn new(
        visibility: Visibility,
        name: Name,
        generic_param_list: GenericParamList,
        where_clause: WhereClause,
        field_list: Spanned<StructDeclFieldList>,
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
pub struct TraitDecl {
    visibility: Visibility,
    pub name: Name,
    associated_types: Vec<AssociatedTypeDecl>,
    methods: MethodList,
}

impl TraitDecl {
    pub fn new(
        visibility: Visibility,
        name: Name,
        associated_types: Vec<AssociatedTypeDecl>,
        methods: MethodList,
    ) -> Self {
        Self {
            visibility,
            name,
            associated_types,
            methods,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Locatable)]
pub struct MethodList(Vec<Method>);

impl MethodList {
    pub fn new(methods: Vec<Method>) -> Self {
        Self(methods)
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Locatable)]
pub struct Method {
    name: Name,
    generic_param_list: GenericParamList,
    param_list: ParamList,
    return_type: TypeIdx,
    pub ast: ast::TraitMethodDecl,
}

impl Method {
    pub fn new(
        name: Name,
        generic_param_list: GenericParamList,
        param_list: ParamList,
        return_type: TypeIdx,
        ast: ast::TraitMethodDecl,
    ) -> Self {
        Self {
            name,
            generic_param_list,
            param_list,
            return_type,
            ast,
        }
    }
}

pub type AssociatedTypeDecl = Name;

#[derive(Debug, Clone, Eq, PartialEq, Locatable)]
pub struct FnDecl {
    pub visibility: Visibility,
    pub name: Name,
    generic_param_list: GenericParamList,
    pub params: ParamList,
    pub ret_type: Spanned<TypeIdx>,
    pub where_clause: WhereClause,
    // pub body: Typed<ExprIdx>,
    pub ast: ast::FnDecl,
}

impl FnDecl {
    pub fn new(
        visibility: Visibility,
        name: Name,
        generic_param_list: GenericParamList,
        params: ParamList,
        ret_type: Spanned<TypeIdx>,
        where_clause: WhereClause,
        // body: Typed<ExprIdx>,
        ast: ast::FnDecl,
    ) -> Self {
        Self {
            visibility,
            name,
            generic_param_list,
            params,
            ret_type,
            where_clause,
            ast, // body,
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

    pub fn poisoned() -> Self {
        Self(vec![])
    }

    /// Get an iterator over the `Spur`s that represent the [`Path`]
    pub fn get_unspanned_spurs(&self) -> impl Iterator<Item = Spur> + '_ {
        self.0.iter().map(|name| name.inner)
    }

    pub fn nth(&self, n: usize) -> Option<&Name> {
        self.0.get(n)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn last(&self) -> Option<&Name> {
        self.0.last()
    }

    pub fn push(&mut self, name: Name) {
        self.0.push(name);
    }

    pub fn to_spur(&self, interner: &'static ThreadedRodeo) -> Spur {
        interner.get_or_intern(
            self.0
                .iter()
                .map(|name| interner.resolve(&name.inner))
                .join("::"),
        )
    }

    pub fn append(mut self, path: &mut Path) {
        self.0.append(&mut path.0);
    }
}

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
pub struct GenericParamList(HashSet<Spur>);

impl GenericParamList {
    pub fn empty() -> Self {
        Self(HashSet::new())
    }

    pub fn new(params: HashSet<Spur>) -> Self {
        Self(params)
    }

    pub fn get(&self, path: &Spur) -> bool {
        self.0.contains(path)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = &Spur> {
        self.0.iter()
    }

    pub fn combine(a: &GenericParamList, b: &GenericParamList) -> Result<Self, Vec<Spur>> {
        let x = a.0.union(&b.0).cloned().collect();
        let duplicates: Vec<_> = a.0.intersection(&b.0).cloned().collect();
        if duplicates.is_empty() {
            Ok(Self(x))
        } else {
            Err(duplicates)
        }
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

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
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

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
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
    Array(Spanned<TypeIdx>, Spanned<u32>),
    Ptr(Spanned<TypeIdx>),
    Generic(Spur),
    Unknown,
}

#[derive(Debug, Clone, Eq, PartialEq, Locatable)]
pub struct StructDeclFieldList(Vec<StructDeclField>);

impl StructDeclFieldList {
    pub fn empty() -> Self {
        Self(vec![])
    }

    pub fn new(fields: Vec<StructDeclField>) -> Self {
        Self(fields)
    }

    pub fn iter(&self) -> impl Iterator<Item = &StructDeclField> {
        self.0.iter()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Locatable)]
pub struct StructDeclField {
    pub name: Name,
    pub ty: Spanned<TypeIdx>,
}

impl StructDeclField {
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

#[derive(Debug, Clone, Eq, PartialEq, Locatable)]
pub struct Typed<T> {
    pub inner: T,
    pub ty: Type,
}

impl<T> Typed<T> {
    pub fn new(inner: T, ty: Type) -> Self {
        Self { inner, ty }
    }
}

pub trait WithType: Sized {
    fn ty_unknown(self) -> Typed<Self> {
        Typed::new(self, Type::Unknown)
    }
}

pub type ExprIdx = Idx<Spanned<Expr>>;

impl WithType for ExprIdx {}

#[derive(Debug, Clone, PartialEq, Locatable)]
pub enum Expr {
    Block(Block),
    Call(Call),
    Float(f64),
    Int(u64),
    Tuple(Vec<ExprIdx>),
    Path(Path),
    Let(Let),
    Struct(Struct),
    Poisoned,
}

#[derive(Debug, Clone, Eq, PartialEq, Locatable)]
pub struct Block(Vec<(ExprIdx, TypeId)>);

impl Block {
    pub fn new(exprs: Vec<(ExprIdx, TypeId)>) -> Self {
        Self(exprs)
    }

    pub fn iter(&self) -> impl Iterator<Item = &(ExprIdx, TypeId)> {
        self.0.iter()
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Locatable)]
pub struct Call {
    path: ExprIdx,
    args: Vec<ExprIdx>,
}

impl Call {
    pub fn new(path: ExprIdx, args: Vec<ExprIdx>) -> Self {
        Self { path, args }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Locatable)]
pub struct Let {
    pub name: Name,
    pub ty: Spanned<TypeIdx>,
    pub val: ExprIdx,
}

impl Let {
    pub fn new(name: Name, ty: Spanned<TypeIdx>, val: ExprIdx) -> Self {
        Self { name, ty, val }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Locatable)]
pub struct Struct {
    pub path: Path,
    pub field_list: StructFieldList,
}

impl Struct {
    pub fn new(path: Path, field_list: StructFieldList) -> Self {
        Self { path, field_list }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Locatable)]
pub struct StructFieldList(Vec<StructField>);

impl StructFieldList {
    pub const EMPTY: Self = Self(vec![]);

    pub fn new(field_list: Vec<StructField>) -> Self {
        Self(field_list)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = &StructField> {
        self.0.iter()
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Locatable)]
pub struct StructField {
    pub name: Name,
    val: ExprIdx,
    pub ty: TypeId,
}

impl StructField {
    pub fn new(name: Name, val: ExprIdx, ty: TypeId) -> Self {
        Self { name, val, ty }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ExpectedPathType {
    Any,
    Local,
    Variable,
    Function,
}
