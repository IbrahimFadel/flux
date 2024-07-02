use std::collections::HashSet;

use flux_diagnostics::ice;
use flux_span::{Interner, Spanned, WithSpan, Word};
use flux_syntax::ast;
use flux_typesystem as ts;
use flux_typesystem::{TEnv, TypeId};
use itertools::Itertools;
use la_arena::{Arena, Idx, RawIdx};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Visibility {
    Private,
    Public,
}

#[derive(Debug)]
pub struct FnDecl {
    pub name: Spanned<Word>,
    pub visibility: Spanned<Visibility>,
    pub generic_params: Spanned<GenericParams>,
    pub params: Spanned<ParamList>,
    pub return_ty: Spanned<TypeId>,
    pub ast: Option<ast::FnDecl>,
}

impl FnDecl {
    pub fn new(
        name: Spanned<Word>,
        visibility: Spanned<Visibility>,
        generic_params: Spanned<GenericParams>,
        params: Spanned<ParamList>,
        return_ty: Spanned<TypeId>,
        ast: Option<ast::FnDecl>,
    ) -> Self {
        Self {
            name,
            visibility,
            generic_params,
            params,
            return_ty,
            ast,
        }
    }
}

#[derive(Debug)]
pub struct ModDecl {
    pub visibility: Spanned<Visibility>,
    pub name: Spanned<Word>,
}

impl ModDecl {
    pub fn new(visibility: Spanned<Visibility>, name: Spanned<Word>) -> Self {
        Self { visibility, name }
    }
}

#[derive(Debug)]
pub struct StructDecl {
    pub visibility: Spanned<Visibility>,
    pub name: Spanned<Word>,
    pub generic_params: Spanned<GenericParams>,
    pub fields: StructFieldDeclList,
}

impl StructDecl {
    pub fn new(
        visibility: Spanned<Visibility>,
        name: Spanned<Word>,
        generic_params: Spanned<GenericParams>,
        fields: StructFieldDeclList,
    ) -> Self {
        Self {
            visibility,
            name,
            generic_params,
            fields,
        }
    }
}

#[derive(Debug)]
pub struct EnumDecl {
    pub visibility: Spanned<Visibility>,
    pub name: Spanned<Word>,
    pub generic_params: Spanned<GenericParams>,
    pub variants: EnumDeclVariantList,
}

impl EnumDecl {
    pub fn new(
        visibility: Spanned<Visibility>,
        name: Spanned<Word>,
        generic_params: Spanned<GenericParams>,
        variants: EnumDeclVariantList,
    ) -> Self {
        Self {
            visibility,
            name,
            generic_params,
            variants,
        }
    }
}

#[derive(Debug)]
pub struct EnumDeclVariantList(Vec<EnumDeclVariant>);

impl EnumDeclVariantList {
    pub fn new(variants: Vec<EnumDeclVariant>) -> Self {
        Self(variants)
    }

    pub fn poisoned() -> Self {
        Self(vec![])
    }

    pub fn iter(&self) -> impl Iterator<Item = &EnumDeclVariant> {
        self.0.iter()
    }
}

#[derive(Debug)]
pub struct EnumDeclVariant {
    pub name: Spanned<Word>,
    pub ty: Option<Spanned<TypeId>>,
}

impl EnumDeclVariant {
    pub fn new(name: Spanned<Word>, ty: Option<Spanned<TypeId>>) -> Self {
        Self { name, ty }
    }
}

#[derive(Debug)]
pub struct TraitDecl {
    pub visibility: Spanned<Visibility>,
    pub name: Spanned<Word>,
    pub generic_params: Spanned<GenericParams>,
    pub assoc_type_decls: Vec<AssociatedTypeDecl>,
    pub methods: Vec<Idx<FnDecl>>,
}

impl TraitDecl {
    pub fn new(
        visibility: Spanned<Visibility>,
        name: Spanned<Word>,
        generic_params: Spanned<GenericParams>,
        assoc_type_decls: Vec<AssociatedTypeDecl>,
        methods: Vec<Idx<FnDecl>>,
    ) -> Self {
        Self {
            visibility,
            name,
            generic_params,
            assoc_type_decls,
            methods,
        }
    }
}

#[derive(Debug)]
pub struct ApplyDecl {
    pub visibility: Spanned<Visibility>,
    pub generic_params: Spanned<GenericParams>,
    pub trt: Option<Spanned<Path>>,
    pub to_ty: Spanned<TypeId>,
    pub assoc_types: Vec<AssociatedTypeDefinition>,
    pub methods: Vec<Idx<FnDecl>>,
}

impl ApplyDecl {
    pub fn new(
        visibility: Spanned<Visibility>,
        generic_params: Spanned<GenericParams>,
        trt: Option<Spanned<Path>>,
        to_ty: Spanned<TypeId>,
        assoc_types: Vec<AssociatedTypeDefinition>,
        methods: Vec<Idx<FnDecl>>,
    ) -> Self {
        Self {
            visibility,
            generic_params,
            trt,
            to_ty,
            assoc_types,
            methods,
        }
    }
}

#[derive(Debug)]
pub struct UseDecl {
    pub path: Spanned<Path>,
    pub alias: Option<Spanned<Word>>,
}

impl UseDecl {
    pub fn new(path: Spanned<Path>, alias: Option<Spanned<Word>>) -> Self {
        Self { path, alias }
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct StructFieldDeclList(Vec<StructFieldDecl>);

impl StructFieldDeclList {
    pub fn new(fields: Vec<StructFieldDecl>) -> Self {
        Self(fields)
    }

    pub fn poisoned() -> Self {
        Self(vec![])
    }

    pub fn iter(&self) -> impl Iterator<Item = &StructFieldDecl> {
        self.0.iter()
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct StructFieldDecl {
    pub name: Spanned<Word>,
    pub ty: Spanned<TypeId>,
}

impl StructFieldDecl {
    pub fn new(name: Spanned<Word>, ty: Spanned<TypeId>) -> Self {
        Self { name, ty }
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct AssociatedTypeDefinition {
    pub name: Spanned<Word>,
    pub ty: Spanned<TypeId>,
}

impl AssociatedTypeDefinition {
    pub fn new(name: Spanned<Word>, ty: Spanned<TypeId>) -> Self {
        Self { name, ty }
    }
}

#[derive(Clone, PartialEq, Eq, Default, Debug)]
pub struct GenericParams {
    pub types: Arena<Spanned<Word>>,
    pub where_predicates: Vec<WherePredicate>,
}

impl GenericParams {
    const INVALID_IDX: u32 = u32::MAX;

    pub const fn invalid_idx(&self) -> Idx<Spanned<Word>> {
        Idx::from_raw(RawIdx::from_u32(Self::INVALID_IDX))
    }

    pub fn empty() -> Self {
        Self {
            types: Arena::new(),
            where_predicates: vec![],
        }
    }

    /// Combine two sets of generic parameters
    ///
    /// If there are duplicates, it will error but still provide a fallback set of generic params (self)
    pub fn union(self, other: &Spanned<Self>) -> Result<Self, (Self, Vec<Word>)> {
        let mut union = self.clone();

        let a_keys: HashSet<Word> = self.types.iter().map(|(_, name)| name.inner).collect();
        let b_keys: HashSet<Word> = other.types.iter().map(|(_, name)| name.inner).collect();
        let duplicates: Vec<_> = a_keys.intersection(&b_keys).copied().collect();

        a_keys.union(&b_keys).for_each(|key| {
            if a_keys.get(key).is_none() {
                // We need to move it into union
                let span = other
                    .types
                    .iter()
                    .find_map(|(_, name)| {
                        if name.inner == *key {
                            Some(name.span)
                        } else {
                            None
                        }
                    })
                    .unwrap_or_else(|| unreachable!());
                let idx = union.types.alloc((*key).at(span));
                other
                    .where_predicates
                    .iter()
                    .filter(|predicate| predicate.name == *key)
                    .for_each(|predicate| {
                        union.where_predicates.push(WherePredicate {
                            ty: idx,
                            name: *key,
                            bound: predicate.bound.clone(),
                        });
                    });
            }
        });

        if duplicates.is_empty() {
            Ok(union)
        } else {
            Err((self, duplicates))
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct WherePredicate {
    pub ty: Idx<Spanned<Word>>,
    pub name: Word,
    pub bound: Spanned<Path>,
}

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub struct Path {
    pub segments: Vec<Word>,
    pub generic_args: Vec<TypeId>,
}

impl Path {
    pub fn new(segments: Vec<Word>, generic_args: Vec<TypeId>) -> Self {
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

    pub fn try_get(&self, idx: usize) -> Option<&Word> {
        self.segments.get(idx)
    }

    pub fn get(&self, idx: usize) -> &Word {
        &self.segments[idx]
    }

    pub fn is_generic(&self, generic_params: &GenericParams) -> bool {
        if self.segments.len() != 1 {
            return false;
        }

        return generic_params
            .types
            .iter()
            .find(|(_, name)| name.inner == *self.get(0))
            .is_some();
    }

    pub fn to_string(&self, interner: &'static Interner) -> String {
        self.segments
            .iter()
            .map(|segment| interner.resolve(segment))
            .join("::")
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Type {
    Array(ArrayType),
    Generic(Generic),
    Path(Path),
    ThisPath(ThisPath),
    Ptr(TypeId),
    Tuple(Vec<TypeId>),
    Never,
    Unknown,
}

impl Type {
    pub const fn unit() -> Self {
        Self::Tuple(vec![])
    }
}

impl ts::Insert<Type> for TEnv {
    fn insert(&mut self, ty: Type) -> TypeId {
        match ty {
            Type::Array(_) => todo!(),
            Type::Generic(generic) => self.insert(ts::TypeKind::Generic(ts::Generic::new(
                generic.name.inner,
                generic
                    .restrictions
                    .iter()
                    .map(|restriction| restriction.inner.0.clone().to_trait_restriction())
                    .collect(),
            ))),
            Type::Path(path) => self.insert(ts::TypeKind::Concrete(ts::ConcreteKind::Path(
                path.segments,
                path.generic_args,
            ))),
            Type::Ptr(to) => self.insert(ts::TypeKind::Concrete(ts::ConcreteKind::Ptr(to))),
            Type::Tuple(types) => {
                self.insert(ts::TypeKind::Concrete(ts::ConcreteKind::Tuple(types)))
            }
            Type::Never => self.insert(ts::TypeKind::Never),
            Type::Unknown => self.insert(ts::TypeKind::Unknown),
            Type::ThisPath(this_path) => self.insert(this_path_to_tkind(this_path, None)),
            // Type::ThisPath(_) => ice("should use `insert_with_trait_ctx` for `Type::ThisPath`"),
        }
    }

    fn insert_with_trait_ctx(
        &mut self,
        ty: Type,
        assoc_types: &mut impl Iterator<Item = (Word, TypeId)>,
    ) -> TypeId {
        match ty {
            Type::ThisPath(this_path) => {
                let tid = assoc_types
                    .find_map(|(name, tid)| {
                        if name == *this_path.path.get(0) {
                            Some(tid)
                        } else {
                            None
                        }
                    })
                    .unwrap_or_else(|| {
                        ice("programmer was too lazy to make a real diagnostic!!! shame")
                    });
                todo!()
            }
            _ => self.insert(ty),
        }
    }

    // fn this_path_to_tkind(
    //     &mut self,
    //     this_path: &Path,
    //     this_type: &Spanned<Path>,
    //     apply: &Apply,
    //     file_id: FileId,
    // ) -> TypeKind {
    //     // let (trt, trait_id) = if let Some(res) = self.get_trait_with_id(this_type) {
    //     //     res
    //     // } else {
    //     //     ice("`This` couldn't be resolved");
    //     // };

    //     apply
    //         .assoc_types
    //         .iter()
    //         .find(|(name, _)| name.inner == *this_path.nth(0))
    //         .map(|(_, ty)| self.type_to_tkind(ty, file_id, None))
    //         .unwrap()
    // }
}

fn this_path_to_tkind(this_path: ThisPath, apply_decl: Option<()>) -> ts::TypeKind {
    match apply_decl {
        Some(_) => todo!(),
        None => ts::TypeKind::ThisPath(this_path.path.segments),
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ThisPath {
    pub path: Path,
    pub path_to_trait: Path,
}

impl ThisPath {
    pub fn new(path: Path, path_to_trait: Path) -> Self {
        Self {
            path,
            path_to_trait,
        }
    }

    pub fn resolve_type(&self, interner: &'static Interner) {
        println!(
            "{:#?} {:#?}",
            self.path.to_string(interner),
            self.path_to_trait.to_string(interner)
        );
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ArrayType {
    pub ty: TypeId,
    pub num: u32,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Generic {
    pub name: Spanned<Word>,
    pub restrictions: TypeBoundList,
}

impl Generic {
    pub fn new(name: Spanned<Word>, restrictions: TypeBoundList) -> Self {
        Self { name, restrictions }
    }
}

impl Path {
    pub fn to_trait_restriction(self) -> ts::TraitRestriction {
        ts::TraitRestriction::new(self.segments, self.generic_args)
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ParamList(Vec<Param>);

impl ParamList {
    pub fn poisoned() -> Self {
        Self(Vec::with_capacity(0))
    }

    pub fn new(params: Vec<Param>) -> Self {
        Self(params)
    }

    pub fn iter(&self) -> impl Iterator<Item = &Param> {
        self.0.iter()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Param {
    pub name: Spanned<Word>,
    pub ty: Spanned<TypeId>,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct TypeBoundList(Vec<Spanned<TypeBound>>);

impl TypeBoundList {
    pub fn new(type_bound_list: Vec<Spanned<TypeBound>>) -> Self {
        Self(type_bound_list)
    }

    pub fn iter(&self) -> impl Iterator<Item = &Spanned<TypeBound>> {
        self.0.iter()
    }

    pub fn as_slice(&self) -> &[Spanned<TypeBound>] {
        &self.0
    }
}

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub struct TypeBound(Path);

impl TypeBound {
    pub fn new(path: Path) -> Self {
        Self(path)
    }

    pub fn path(&self) -> &Path {
        &self.0
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct AssociatedTypeDecl {
    pub name: Spanned<Word>,
    pub type_bound_list: TypeBoundList,
}

impl AssociatedTypeDecl {
    pub fn new(name: Spanned<Word>, type_bound_list: TypeBoundList) -> Self {
        Self {
            name,
            type_bound_list,
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub struct Typed<T> {
    tid: TypeId,
    inner: T,
}

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub enum Expr {
    // Block(Block),
    // BinOp(BinOp),
    // Enum(EnumExpr),
    // Call(Call),
    // Float(f64),
    // Int(u64),
    // Tuple(Vec<ExprIdx>),
    // Path(Path),
    // Let(Let),
    // Struct(StructExpr),
    // MemberAccess(MemberAccess),
    // If(If),
    // Intrinsic(Intrinsic),
    // Str(Str),
    // Poisoned,
}

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub struct ExprIdx(Idx<Spanned<Expr>>);
