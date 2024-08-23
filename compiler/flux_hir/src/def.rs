use std::collections::HashSet;

use flux_id::{id, ids, Map};
use flux_typesystem::{Type, Typed};
use flux_util::{Path, Spanned, WithSpan, Word};

pub mod expr;
pub mod item;

ids!(GParam);

#[derive(Clone, PartialEq, Eq, Default, Debug)]
pub struct GenericParams {
    pub types: Map<GParam, Spanned<Word>>,
    pub where_predicates: Vec<WherePredicate>,
}

impl GenericParams {
    pub(crate) const INVALID_ID: GParam = GParam::new(u32::MAX);

    pub fn new(types: Map<GParam, Spanned<Word>>, where_predicates: Vec<WherePredicate>) -> Self {
        Self {
            types,
            where_predicates,
        }
    }

    pub fn empty() -> Self {
        Self {
            types: Map::new(),
            where_predicates: vec![],
        }
    }

    pub fn is_path_generic<A>(&self, path: &Path<Word, A>) -> bool {
        if path.len() != 1 {
            return false;
        }

        return self
            .types
            .iter()
            .find(|(_, name)| name.inner == *path.get_nth(0))
            .is_some();
    }

    // pub fn get_bounds_on_generic(&self, name: &Word) -> Vec<TraitRestriction> {
    //     let bounds = self.where_predicates.iter().filter_map(|predicate| {
    //         if predicate.name == *name {
    //             Some(predicate.bound.inner.clone())
    //         } else {
    //             None
    //         }
    //     });
    //     bounds.collect()
    // }

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
                let idx = union.types.insert((*key).at(span));
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
    pub ty: GParam,
    pub name: Word,
    pub bound: Spanned<Path<Word, Type>>,
}

impl WherePredicate {
    pub fn new(ty: GParam, name: Word, bound: Spanned<Path<Word, Type>>) -> Self {
        Self { ty, name, bound }
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
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

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Param {
    pub name: Spanned<Word>,
    pub ty: Spanned<Type>,
}

impl Param {
    pub fn new(name: Spanned<Word>, ty: Spanned<Type>) -> Self {
        Self { name, ty }
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct StructFieldDeclList(Vec<StructFieldDecl>);

impl StructFieldDeclList {
    pub fn new(fields: Vec<StructFieldDecl>) -> Self {
        Self(fields)
    }

    pub fn empty() -> Self {
        Self(vec![])
    }

    pub fn iter(&self) -> impl Iterator<Item = &StructFieldDecl> {
        self.0.iter()
    }

    pub fn find(&self, name: Word) -> Option<&StructFieldDecl> {
        self.iter().find(|field| field.name.inner == name)
    }

    pub fn contains(&self, name: Word) -> bool {
        self.iter().find(|field| field.name.inner == name).is_some()
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct StructFieldDecl {
    pub name: Spanned<Word>,
    pub ty: Spanned<Type>,
}

impl StructFieldDecl {
    pub fn new(name: Spanned<Word>, ty: Spanned<Type>) -> Self {
        Self { name, ty }
    }
}

#[derive(Debug, Clone)]
pub struct EnumDeclVariantList(Vec<EnumDeclVariant>);

impl EnumDeclVariantList {
    pub fn new(variants: Vec<EnumDeclVariant>) -> Self {
        Self(variants)
    }
}

#[derive(Debug, Clone)]
pub struct EnumDeclVariant {
    pub name: Spanned<Word>,
    pub ty: Option<Spanned<Type>>,
}

impl EnumDeclVariant {
    pub fn new(name: Spanned<Word>, ty: Option<Spanned<Type>>) -> Self {
        Self { name, ty }
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

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct TypeBound(Path<Word, Type>);

impl TypeBound {
    pub fn new(path: Path<Word, Type>) -> Self {
        Self(path)
    }

    pub fn path(&self) -> &Path<Word, Type> {
        &self.0
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct AssociatedTypeDefinition {
    pub name: Spanned<Word>,
    pub ty: Spanned<Type>,
}

impl AssociatedTypeDefinition {
    pub fn new(name: Spanned<Word>, ty: Spanned<Type>) -> Self {
        Self { name, ty }
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct StructExprFieldList(Vec<StructExprField>);

impl StructExprFieldList {
    pub fn new(fields: Vec<StructExprField>) -> Self {
        Self(fields)
    }

    pub fn empty() -> Self {
        Self(vec![])
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct StructExprField {
    name: Spanned<Word>,
    val: id::Expr,
}

impl StructExprField {
    pub fn new(name: Spanned<Word>, val: id::Expr) -> Self {
        Self { name, val }
    }
}
