use flux_id::id;
use flux_parser::ast;
use flux_typesystem::Type;
use flux_util::{Path, Spanned, Word};

use crate::lower::item_tree::ItemTree;

use super::{
    AssociatedTypeDecl, AssociatedTypeDefinition, EnumDeclVariantList, GenericParams, ParamList,
    StructFieldDeclList,
};

#[derive(Debug, PartialEq, Eq, Copy, Clone, Hash)]
pub enum Visibility {
    Private,
    Public,
}

#[derive(Debug, Clone)]
pub struct FnDecl {
    pub name: Spanned<Word>,
    pub visibility: Spanned<Visibility>,
    pub generic_params: Spanned<GenericParams>,
    pub params: Spanned<ParamList>,
    pub return_ty: Spanned<Type>,
    pub ast: Option<ast::FnDecl>,
}

impl FnDecl {
    pub fn new(
        name: Spanned<Word>,
        visibility: Spanned<Visibility>,
        generic_params: Spanned<GenericParams>,
        params: Spanned<ParamList>,
        return_ty: Spanned<Type>,
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

#[derive(Debug, Clone)]
pub struct ModDecl {
    pub visibility: Spanned<Visibility>,
    pub name: Spanned<Word>,
}

impl ModDecl {
    pub fn new(visibility: Spanned<Visibility>, name: Spanned<Word>) -> Self {
        Self { visibility, name }
    }
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub struct TraitDecl {
    pub visibility: Spanned<Visibility>,
    pub name: Spanned<Word>,
    pub generic_params: Spanned<GenericParams>,
    pub assoc_type_decls: Vec<AssociatedTypeDecl>,
    pub methods: Vec<id::FnDecl>,
}

impl TraitDecl {
    pub fn new(
        visibility: Spanned<Visibility>,
        name: Spanned<Word>,
        generic_params: Spanned<GenericParams>,
        assoc_type_decls: Vec<AssociatedTypeDecl>,
        methods: Vec<id::FnDecl>,
    ) -> Self {
        Self {
            visibility,
            name,
            generic_params,
            assoc_type_decls,
            methods,
        }
    }

    pub(crate) fn get_method_in_item_tree<'a>(
        &'a self,
        name: Word,
        item_tree: &'a ItemTree,
    ) -> Option<&FnDecl> {
        self.methods.iter().find_map(|method_id| {
            let fn_decl = item_tree.functions.get(*method_id);
            if fn_decl.name.inner == name {
                Some(fn_decl)
            } else {
                None
            }
        })
    }
}

#[derive(Debug, Clone)]
pub struct ApplyDecl {
    pub visibility: Spanned<Visibility>,
    pub generic_params: Spanned<GenericParams>,
    pub trt: Option<Spanned<Path<Word, Type>>>,
    pub to_ty: Spanned<Type>,
    pub assoc_types: Vec<AssociatedTypeDefinition>,
    pub methods: Vec<id::FnDecl>,
}

impl ApplyDecl {
    pub fn new(
        visibility: Spanned<Visibility>,
        generic_params: Spanned<GenericParams>,
        trt: Option<Spanned<Path<Word, Type>>>,
        to_ty: Spanned<Type>,
        assoc_types: Vec<AssociatedTypeDefinition>,
        methods: Vec<id::FnDecl>,
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

#[derive(Debug, Clone)]
pub struct UseDecl {
    pub path: Spanned<Path<Word>>,
    pub alias: Option<Spanned<Word>>,
    pub all: bool,
}

impl UseDecl {
    pub fn new(path: Spanned<Path<Word>>, alias: Option<Spanned<Word>>, all: bool) -> Self {
        Self { path, alias, all }
    }
}
