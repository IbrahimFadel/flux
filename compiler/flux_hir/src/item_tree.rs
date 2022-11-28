use std::fmt::Formatter;
use std::hash::{Hash, Hasher};
use std::ops::Index;
use std::{fmt, marker::PhantomData};

use flux_diagnostics::Diagnostic;
use flux_span::{FileId, Spanned};
use flux_syntax::ast::{self, AstNode};
use flux_syntax::SyntaxNode;
use flux_typesystem::TEnv;
use la_arena::{Arena, Idx};
use lasso::ThreadedRodeo;

use crate::hir::{
    GenericParamList, Name, ParamList, Path, StructFieldList, Type, TypeIdx, UseAlias, Visibility,
    WhereClause,
};

use self::lower::Context;

mod lower;
#[cfg(test)]
mod tests;

#[derive(Debug)]
pub struct ItemTree {
    file_id: FileId,
    pub top_level: Vec<ModItem>,
    data: ItemTreeData,
}

pub fn generate_item_tree(
    file_id: FileId,
    root: SyntaxNode,
    interner: &'static ThreadedRodeo,
) -> (ItemTree, Vec<Diagnostic>, Arena<Spanned<Type>>) {
    let ctx = Context::new(file_id, interner);
    let root =
        ast::Root::cast(root).expect("internal compiler error: root node should always cast");
    ctx.lower_module_items(root)
}

#[derive(Default, Debug, Eq, PartialEq)]
struct ItemTreeData {
    mods: Arena<Mod>,
    uses: Arena<Use>,
    functions: Arena<Function>,
    structs: Arena<Struct>,
    types: Arena<Type>,
    // visibilities: ItemVisibilities,
}

/// Trait implemented by all item nodes in the item tree.
pub trait ItemTreeNode: Clone {
    // type Source: AstNode + Into<ast::ModuleItem>;
    type Source: AstNode;

    /// Returns the AST id for this instance
    // fn ast_id(&self) -> FileAstId<Self::Source>;

    /// Looks up an instance of `Self` in an item tree.
    fn lookup(tree: &ItemTree, index: Idx<Self>) -> &Self;

    /// Downcasts a `ModItem` to a `FileItemTreeId` specific to this type
    fn id_from_mod_item(mod_item: ModItem) -> Option<LocalItemTreeId<Self>>;

    /// Upcasts a `FileItemTreeId` to a generic ModItem.
    fn id_to_mod_item(id: LocalItemTreeId<Self>) -> ModItem;
}

/// The typed Id of an item in an `ItemTree`
pub struct LocalItemTreeId<N: ItemTreeNode> {
    pub index: Idx<N>,
    _p: PhantomData<N>,
}

impl<N: ItemTreeNode> Clone for LocalItemTreeId<N> {
    fn clone(&self) -> Self {
        Self {
            index: self.index,
            _p: PhantomData,
        }
    }
}
impl<N: ItemTreeNode> Copy for LocalItemTreeId<N> {}

impl<N: ItemTreeNode> PartialEq for LocalItemTreeId<N> {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index
    }
}
impl<N: ItemTreeNode> Eq for LocalItemTreeId<N> {}

impl<N: ItemTreeNode> Hash for LocalItemTreeId<N> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.index.hash(state)
    }
}

impl<N: ItemTreeNode> fmt::Debug for LocalItemTreeId<N> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.index.fmt(f)
    }
}

macro_rules! mod_items {
    ( $( $typ:ident in $fld:ident -> $ast:ty ),+ $(,)?) => {
        #[derive(Debug,Copy,Clone,Eq,PartialEq,Hash)]
        pub enum ModItem {
            $(
                $typ(LocalItemTreeId<$typ>),
            )+
        }

        $(
            impl From<LocalItemTreeId<$typ>> for ModItem {
                fn from(id: LocalItemTreeId<$typ>) -> ModItem {
                    ModItem::$typ(id)
                }
            }
        )+

        $(
            impl ItemTreeNode for $typ {
                type Source = $ast;

                // fn ast_id(&self) -> FileAstId<Self::Source> {
                //     self.ast_id
                // }

                fn lookup(tree: &ItemTree, index: Idx<Self>) -> &Self {
                    &tree.data.$fld[index]
                }

                fn id_from_mod_item(mod_item: ModItem) -> Option<LocalItemTreeId<Self>> {
                    if let ModItem::$typ(id) = mod_item {
                        Some(id)
                    } else {
                        None
                    }
                }

                fn id_to_mod_item(id: LocalItemTreeId<Self>) -> ModItem {
                    ModItem::$typ(id)
                }
            }

            impl Index<Idx<$typ>> for ItemTree {
                type Output = $typ;

                fn index(&self, index: Idx<$typ>) -> &Self::Output {
                    &self.data.$fld[index]
                }
            }
        )+
    };
}

mod_items! {
    Function in functions -> ast::FnDecl,
    Struct in structs -> ast::StructDecl,
    Mod in mods -> ast::ModDecl,
    // TypeAlias in type_aliases -> ast::TypeAliasDef,
    Use in uses -> ast::UseDecl,
}

impl<T: ItemTreeNode> Into<LocalItemTreeId<T>> for Idx<T> {
    fn into(self) -> LocalItemTreeId<T> {
        LocalItemTreeId {
            index: self,
            _p: PhantomData,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Use {
    pub path: Path,
    pub alias: Option<UseAlias>,
    pub ast: ast::UseDecl,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Mod {
    pub name: Name,
    pub ast: ast::ModDecl,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Function {
    pub visibility: Visibility,
    pub name: Name,
    pub generic_param_list: GenericParamList,
    pub params: ParamList,
    pub ret_type: TypeIdx,
    pub where_clause: WhereClause,
    pub ast: ast::FnDecl,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Struct {
    pub visibility: Visibility,
    pub name: Name,
    pub generic_param_list: GenericParamList,
    pub where_clause: WhereClause,
    pub field_list: StructFieldList,
    pub ast: ast::StructDecl,
}

// #[derive(Debug, Clone, Eq, PartialEq)]
// pub enum Type {
//     Path(Path),
//     Generic(Name),
// }
