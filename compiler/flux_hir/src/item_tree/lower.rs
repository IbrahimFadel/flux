use flux_diagnostics::Diagnostic;
use flux_span::{FileId, ToSpan, WithSpan};
use flux_syntax::ast::{self, AstNode, Root};
use flux_typesystem::{TEnv, TypeId};
use lasso::ThreadedRodeo;

use crate::hir::{
    GenericParamList, Name, TypeBound, TypeBoundList, Visibility, WhereClause, WherePredicate,
};

use super::{Function, ItemTree, ItemTreeData, LocalItemTreeId, ModItem, Struct};

pub(super) struct Context {
    interner: &'static ThreadedRodeo,
    file_id: FileId,
    data: ItemTreeData,
    diagnostics: Vec<Diagnostic>,
    tenv: TEnv,
}

const POISONED_NAME: &str = "POISONED";

impl Context {
    pub fn new(file_id: FileId, interner: &'static ThreadedRodeo) -> Self {
        Self {
            interner,
            file_id,
            data: ItemTreeData::default(),
            diagnostics: vec![],
            tenv: TEnv::new(interner),
        }
    }

    /// Lower an AST node to its HIR equivalent
    ///
    /// This exists to help clean up the lowering process due to the optional nature of the AST layer.
    /// We want certain nodes to **ALWAYS** be emitted even when there's a parsing error, but be marked as poisoned.
    /// For this reason, we can `unwrap`/`expect` safely (panics are ICEs), then carry on.
    ///
    /// If the node is poisoned, use the supplied closure to provide a poisoned value.
    /// If the node is not poisoned, use the supplied closure to carry out the regular lowering process.
    fn lower_node<N, T, P, F>(
        &mut self,
        node: Option<N>,
        poison_function: P,
        normal_function: F,
    ) -> T
    where
        N: AstNode,
        P: FnOnce(&mut Self, N) -> T,
        F: FnOnce(&mut Self, N) -> T,
    {
        let n = node.expect("internal compiler error: missing node that should always be emitted");
        if n.is_poisoned() {
            poison_function(self, n)
        } else {
            normal_function(self, n)
        }
    }

    pub(super) fn lower_module_items(mut self, root: Root) -> ItemTree {
        let mut top_level: Vec<_> = root
            .fn_decls()
            .map(|fn_decl| self.lower_fn_decl(fn_decl))
            .map(Into::into)
            .collect();
        let mut structs: Vec<_> = root
            .struct_decls()
            .map(|struct_decl| self.lower_struct_decl(struct_decl))
            .map(Into::into)
            .collect();
        top_level.append(&mut structs);
        ItemTree {
            file_id: self.file_id,
            top_level,
            data: self.data,
        }
    }

    fn lower_name(&mut self, name: Option<ast::Name>) -> Name {
        self.lower_node(
            name,
            |this, name| {
                self.interner
                    .get_or_intern_static(POISONED_NAME)
                    .at(name.range().to_span())
            },
            |this, name| {
               name.ident().expect("internal compiler error: name did not contain identifier but was not marked poisoned").text_key().at(name.range().to_span())
            },
        )
    }

    fn lower_visibility(&mut self, visibility: Option<ast::Visibility>) -> Visibility {
        self.lower_node(
            visibility,
            |_, _| Visibility::Private,
            |_, visibility| {
                visibility
                    .public()
                    .map_or(Visibility::Private, |_| Visibility::Public)
            },
        )
    }

    fn lower_generic_param_list(
        &mut self,
        generic_param_list: Option<ast::GenericParamList>,
    ) -> GenericParamList {
        if let Some(generic_param_list) = generic_param_list {
            let type_params = generic_param_list
                .type_params()
                .map(|type_param| self.lower_name(type_param.name()))
                .collect();
            GenericParamList::new(type_params)
        } else {
            GenericParamList::empty()
        }
    }

    fn lower_where_clause(
        &mut self,
        where_clause: Option<ast::WhereClause>,
        generic_param_list: &GenericParamList,
    ) -> WhereClause {
        if let Some(where_clause) = where_clause {
            let predicates = where_clause
                .predicates()
                .map(|where_predicate| {
                    self.lower_where_predicate(where_predicate, generic_param_list)
                })
                .collect();
            WhereClause::new(predicates)
        } else {
            WhereClause::EMPTY
        }
    }

    fn lower_where_predicate(
        &mut self,
        where_predicate: ast::WherePredicate,
        generic_param_list: &GenericParamList,
    ) -> WherePredicate {
        let generic = self.lower_name(where_predicate.name());
        let trait_restrictions = self.lower_node(
            where_predicate.type_bound_list(),
            |_, _| TypeBoundList::EMPTY,
            |this, type_bound_list| this.lower_type_bound_list(type_bound_list, generic_param_list),
        );
        WherePredicate::with_trait_restrictions(generic, trait_restrictions)
    }

    fn lower_type_bound_list(
        &mut self,
        type_bound_list: ast::TypeBoundList,
        generic_param_list: &GenericParamList,
    ) -> TypeBoundList {
        TypeBoundList::new(
            type_bound_list
                .type_bounds()
                .map(|type_bound| self.lower_type_bound(type_bound, generic_param_list))
                .collect(),
        )
    }

    fn lower_type_bound(
        &mut self,
        type_bound: ast::TypeBound,
        generic_params_list: &GenericParamList,
    ) -> TypeBound {
        let name = self.lower_name(type_bound.trait_name());
        let generic_arg_list =
            self.lower_generic_arg_list(type_bound.generic_arg_list(), generic_params_list);
        TypeBound::with_args(name, generic_arg_list)
    }

    fn lower_generic_arg_list(
        &mut self,
        generic_arg_list: Option<ast::GenericArgList>,
        generic_params_list: &GenericParamList,
    ) -> Vec<TypeId> {
        self.lower_node(
            generic_arg_list,
            |_, _| vec![],
            |this, generic_arg_list| {
                generic_arg_list
                    .args()
                    .map(|ty| this.lower_type(Some(ty), generic_params_list))
                    .collect()
            },
        )
    }

    fn lower_type(
        &mut self,
        ty: Option<ast::Type>,
        generic_param_list: &GenericParamList,
    ) -> TypeId {
        self.lower_node(
            ty,
            |this, ty| {
                this.tenv
                    .insert_unknown(ty.range().to_span().in_file(this.file_id))
            },
            |this, ty| match ty {
                ast::Type::PathType(path_type) => {
                    this.lower_path_or_generic_type(path_type, generic_param_list)
                }
                _ => todo!(),
            },
        )
    }

    fn lower_path_or_generic_type(
        &mut self,
        path_ty: ast::PathType,
        generic_param_list: &GenericParamList,
    ) -> TypeId {
        let path = path_ty
            .path()
            .expect("internal compiler error: path type does not contain path");
        // let path = self.lower_path(path.segments());
        // let args = self.lower_generic_arg_list(path_ty.generic_arg_list(), generic_param_list);
        // let ty = if path.len() > 1 {
        //     Type::Path(path, args)
        // } else if generic_param_list
        //     .get(path.nth(0).expect("internal compiler error: path is empty"))
        // {
        //     Type::Generic(*path.get_unspanned_spurs().first().unwrap())
        // } else {
        //     Type::Path(path, args)
        // };

        // self.types.alloc(ty.at(path_ty.range().to_span()))
        todo!()
    }

    fn lower_struct_decl(&mut self, struct_decl: ast::StructDecl) -> LocalItemTreeId<Struct> {
        let name = self.lower_name(struct_decl.name());
        let visibility = self.lower_visibility(struct_decl.visibility());
        let generic_param_list = self.lower_generic_param_list(struct_decl.generic_param_list());
        let where_clause = self.lower_where_clause(struct_decl.where_clause(), &generic_param_list);
        let strukt = Struct {
            name,
            visibility,
            ast: struct_decl,
        };
        self.data.structs.alloc(strukt).into()
    }

    fn lower_fn_decl(&mut self, fn_decl: ast::FnDecl) -> LocalItemTreeId<Function> {
        let name = self.lower_name(fn_decl.name());
        let visibility = self.lower_visibility(fn_decl.visibility());
        let generic_param_list = self.lower_generic_param_list(fn_decl.generic_param_list());
        let where_clause = self.lower_where_clause(fn_decl.where_clause(), &generic_param_list);
        let function = Function {
            name,
            visibility,
            ast: fn_decl,
        };
        self.data.functions.alloc(function).into()
    }
}
