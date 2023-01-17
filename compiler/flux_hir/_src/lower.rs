use crate::{
    hir::{
        Expr, FnDecl, GenericParamList, ModDecl, Module, Name, Param, ParamList, Path, StructDecl,
        StructField, StructFieldList, Type, TypeBound, TypeBoundList, UseDecl, Visibility,
        WhereClause, WherePredicate, WithType,
    },
    type_interner::{TypeIdx, TypeInterner},
};
use flux_span::{Span, Spanned, ToSpan, WithSpan};
use flux_syntax::ast::{self, AstNode};

use la_arena::Arena;
use lasso::ThreadedRodeo;

mod expr;

const POISONED_NAME: &str = "POISONED";

pub(super) struct Context {
    interner: &'static ThreadedRodeo,
    type_interner: TypeInterner,
    exprs: Arena<Spanned<Expr>>,
}

impl Context {
    pub fn new(interner: &'static ThreadedRodeo) -> Self {
        Self {
            interner,
            type_interner: TypeInterner::new(interner),
            exprs: Arena::new(),
        }
    }

    pub(super) fn lower(mut self, root: ast::Root) -> Module {
        let mut module = Module::default();
        root.fn_decls().for_each(|fn_decl| {
            let f = self.lower_fn_decl(fn_decl);
            module.functions.alloc(f);
        });
        root.mod_decls().for_each(|m| {
            let m = self.lower_mod_decl(m);
            module.mods.alloc(m);
        });
        root.use_decls().for_each(|u| {
            let u = self.lower_use_decl(u);
            module.uses.alloc(u);
        });
        root.struct_decls().for_each(|strukt| {
            let s = self.lower_struct_decl(strukt);
            module.structs.alloc(s);
        });
        module.exprs = self.exprs;
        module.types = self.type_interner;
        module
    }

    /// Lower an AST node to its HIR equivalent
    ///
    /// This exists to help clean up the lowering process due to the optional nature of the AST layer.
    /// We want certain nodes to **ALWAYS** be emitted even when there's a parsing error, but be marked as poisoned.
    /// For this reason, we can `unwrap`/`expect` safely (panics are ICEs), then carry on.
    ///
    /// If the node is poisoned, use the supplied closure to provide a poisoned value.
    /// If the node is not poisoned, use the supplied closure to carry out the regular lowering process.
    ///
    /// This method can be quite verbose and clog up code, so generally this should be used in generalizable methods such as `lower_name` or `lower_generic_param_list`, not in unique methods such as `lower_fn_decl`.
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

    fn lower_name(&mut self, name: Option<ast::Name>) -> Name {
        self.lower_node(
            name,
            |_, name| {
                self.interner
                    .get_or_intern_static(POISONED_NAME)
                    .at(name.range().to_span())
            },
            |_, name| {
                name.ident().expect("internal compiler error: name did not contain identifier but was not marked poisoned").text_key().at(name.range().to_span())
            },
        )
    }

    fn lower_path<'a>(&mut self, path: Option<ast::Path>) -> Spanned<Path> {
        self.lower_node(
            path,
            |this, path| Path::poisoned().at(path.range().to_span()),
            |this, path| {
                Path::from_segments(
                    path.segments()
                        .map(|tok| tok.text_key().at(tok.text_range().to_span())),
                )
                .at(path.range().to_span())
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
    ) -> Vec<Spanned<TypeIdx>> {
        if let Some(generic_arg_list) = generic_arg_list {
            generic_arg_list
                .args()
                .map(|ty| {
                    let span = ty.range().to_span();
                    self.lower_type(Some(ty), generic_params_list, span)
                })
                .collect()
        } else {
            vec![]
        }
    }

    fn lower_type(
        &mut self,
        ty: Option<ast::Type>,
        generic_param_list: &GenericParamList,
        fallback_span: Span,
    ) -> Spanned<TypeIdx> {
        self.lower_node(
            ty,
            |this, _| {
                this.type_interner
                    .intern(Type::Tuple(vec![]))
                    .at(fallback_span)
            },
            |this, ty| match ty {
                ast::Type::ArrayType(arr) => this.lower_array_type(arr, generic_param_list),
                ast::Type::PathType(path_type) => {
                    this.lower_path_or_generic_type(path_type, generic_param_list)
                }
                ast::Type::PtrType(ptr) => this.lower_ptr_type(ptr, generic_param_list),
                ast::Type::TupleType(tuple) => this.lower_tuple_type(tuple, generic_param_list),
            },
        )
    }

    fn lower_array_type(
        &mut self,
        arr: ast::ArrayType,
        generic_param_list: &GenericParamList,
    ) -> Spanned<TypeIdx> {
        let span = arr.range().to_span();
        let ty = self.lower_type(arr.ty(), generic_param_list, span);
        let n: Spanned<u32> = self.lower_node(
            arr.n(),
            |_, _| 0.at(span),
            |_, int| {
                let span = int.range().to_span();
                let value_str = match int.v() {
                    Some(v) => self
                        .interner
                        .resolve(&v.text_key())
                        .at(v.text_range().to_span()),
                    None => "0".at(span),
                };
                value_str.map(|v| match v.parse() {
                    Ok(v) => v,
                    Err(_) => todo!(),
                })
            },
        );
        let ty = Type::Array(ty, n);
        self.type_interner.intern(ty).at(span)
    }

    fn lower_path_or_generic_type(
        &mut self,
        path_ty: ast::PathType,
        generic_param_list: &GenericParamList,
    ) -> Spanned<TypeIdx> {
        let path = self.lower_path(path_ty.path());
        let args = self.lower_generic_arg_list(path_ty.generic_arg_list(), generic_param_list);
        let ty = if path.len() > 1 {
            Type::Path(path.inner, args)
        } else if generic_param_list
            .get(path.nth(0).expect("internal compiler error: path is empty"))
        {
            Type::Generic(
                *path
                    .get_unspanned_spurs()
                    .collect::<Vec<_>>()
                    .first()
                    .unwrap(),
            )
        } else {
            Type::Path(path.inner, args)
        };

        self.type_interner.intern(ty).at(path_ty.range().to_span())
    }

    fn lower_ptr_type(
        &mut self,
        ptr: ast::PtrType,
        generic_param_list: &GenericParamList,
    ) -> Spanned<TypeIdx> {
        let span = ptr.range().to_span();
        let ty = self.lower_type(ptr.ty(), generic_param_list, span);
        let ty = Type::Ptr(ty.at(span));
        self.type_interner.intern(ty).at(span)
    }

    fn lower_tuple_type(
        &mut self,
        tuple: ast::TupleType,
        generic_param_list: &GenericParamList,
    ) -> Spanned<TypeIdx> {
        let types: Vec<_> = tuple
            .types()
            .map(|ty| {
                let span = ty.range().to_span();
                self.lower_type(Some(ty), generic_param_list, span)
            })
            .collect();
        let ty = Type::Tuple(types);
        self.type_interner.intern(ty).at(tuple.range().to_span())
    }

    fn lower_struct_field_list(
        &mut self,
        field_list: Option<ast::StructDeclFieldList>,
        generic_param_list: &GenericParamList,
    ) -> StructFieldList {
        self.lower_node(
            field_list,
            |_, _| StructFieldList::empty(),
            |this, field_list| {
                let fields = field_list
                    .fields()
                    .map(|field| this.lower_struct_field(field, generic_param_list))
                    .collect();
                StructFieldList::new(fields)
            },
        )
    }

    fn lower_struct_field(
        &mut self,
        field: ast::StructDeclField,
        generic_param_list: &GenericParamList,
    ) -> StructField {
        let ty = self.lower_type(field.ty(), generic_param_list, field.range().to_span());
        let name = self.lower_name(field.name());
        StructField::new(name, ty)
    }

    fn lower_fn_param_list(
        &mut self,
        param_list: Option<ast::ParamList>,
        generic_param_list: &GenericParamList,
    ) -> ParamList {
        self.lower_node(
            param_list,
            |_, _| ParamList::empty(),
            |this, param_list| {
                let params = param_list
                    .params()
                    .map(|param| this.lower_fn_param(param, generic_param_list))
                    .collect();
                ParamList::new(params)
            },
        )
    }

    fn lower_fn_param(
        &mut self,
        param: ast::Param,
        generic_param_list: &GenericParamList,
    ) -> Param {
        let name = self.lower_name(param.name());
        let ty = self.lower_type(param.ty(), generic_param_list, param.range().to_span());
        Param::new(name, ty)
    }

    fn lower_fn_return_type(
        &mut self,
        ty: Option<ast::Type>,
        generic_param_list: &GenericParamList,
        fallback_span: Span,
    ) -> Spanned<TypeIdx> {
        if let Some(ty) = ty {
            self.lower_type(Some(ty), generic_param_list, fallback_span)
        } else {
            self.type_interner
                .intern(Type::Tuple(vec![]))
                .at(fallback_span)
        }
    }

    fn lower_mod_decl(&mut self, mod_decl: ast::ModDecl) -> ModDecl {
        let name = self.lower_name(mod_decl.name());
        ModDecl::new(name)
    }

    fn lower_use_decl(&mut self, use_decl: ast::UseDecl) -> UseDecl {
        let path = self.lower_path(use_decl.path());
        UseDecl::new(path.inner)
    }

    fn lower_struct_decl(&mut self, struct_decl: ast::StructDecl) -> StructDecl {
        let name = self.lower_name(struct_decl.name());
        let visibility = self.lower_visibility(struct_decl.visibility());
        let generic_param_list = self.lower_generic_param_list(struct_decl.generic_param_list());
        let where_clause = self.lower_where_clause(struct_decl.where_clause(), &generic_param_list);
        let field_list =
            self.lower_struct_field_list(struct_decl.field_list(), &generic_param_list);
        StructDecl::new(
            visibility,
            name,
            generic_param_list,
            where_clause,
            field_list,
        )
    }

    fn lower_fn_decl(&mut self, fn_decl: ast::FnDecl) -> FnDecl {
        let name = self.lower_name(fn_decl.name());
        let visibility = self.lower_visibility(fn_decl.visibility());
        let generic_param_list = self.lower_generic_param_list(fn_decl.generic_param_list());
        let where_clause = self.lower_where_clause(fn_decl.where_clause(), &generic_param_list);
        let params = self.lower_fn_param_list(fn_decl.param_list(), &generic_param_list);
        let ret_type = self.lower_fn_return_type(
            fn_decl.return_type(),
            &generic_param_list,
            fn_decl.range().to_span(),
        );
        let body = self.lower_expr(fn_decl.body()).ty_unknown();
        FnDecl::new(
            visibility,
            name,
            generic_param_list,
            params,
            ret_type,
            where_clause,
            body,
        )
    }
}
