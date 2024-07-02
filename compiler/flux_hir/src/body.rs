use flux_diagnostics::ice;
use flux_span::{Interner, Spanned, ToSpan, WithSpan, Word};
use flux_syntax::ast::{self, AstNode};
use flux_typesystem::{Insert, TEnv, TypeId};

use crate::{
    hir::{GenericParams, Path, Type},
    module::ModuleId,
    name_res::item::ItemResolver,
    POISONED_NAME,
};

mod r#type;

pub(crate) struct LowerCtx<'a> {
    // diagnostics: Vec<Diagnostic>,
    item_resolver: ItemResolver<'a>,
    pub interner: &'static Interner,
    // Every module will modify a type environment global to the package, stored in the pkg builder
    pub tenv: &'a mut TEnv,
    module_id: ModuleId,
}

impl<'a> LowerCtx<'a> {
    pub(crate) fn new(
        item_resolver: ItemResolver<'a>,
        interner: &'static Interner,
        tenv: &'a mut TEnv,
        module_id: ModuleId,
    ) -> Self {
        Self {
            // diagnostics: vec![],
            item_resolver,
            interner,
            tenv,
            module_id,
        }
    }

    pub fn lower_node<N, T, P, F>(
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
        let n = node.unwrap_or_else(|| ice("missing node that should always be emitted"));
        if n.is_poisoned() {
            poison_function(self, n)
        } else {
            normal_function(self, n)
        }
    }

    pub fn lower_optional_node<N, T, P, F>(
        &mut self,
        node: Option<N>,
        poison_function: P,
        normal_function: F,
    ) -> T
    where
        N: AstNode,
        P: FnOnce(&mut Self) -> T,
        F: FnOnce(&mut Self, N) -> T,
    {
        match node {
            Some(n) => {
                if n.is_poisoned() {
                    poison_function(self)
                } else {
                    normal_function(self, n)
                }
            }
            None => poison_function(self),
        }
    }

    pub(crate) fn lower_type(
        &mut self,
        ty: Option<ast::Type>,
        generic_params: &GenericParams,
    ) -> Spanned<TypeId> {
        let ty = self.lower_node(
            ty,
            |_, ty| Type::Unknown.at(ty.range().to_span()),
            |this, ty| match ty {
                ast::Type::PathType(path_type) => this.lower_path_type(path_type, generic_params),
                ast::Type::ThisPathType(_) => {
                    ice("should not encounter this path outside of trait method")
                }
                _ => ice("unimplemented"),
            },
        );
        self.tenv.insert(ty.inner).at(ty.span)
    }

    pub(crate) fn lower_trait_method_type(
        &mut self,
        ty: Option<ast::Type>,
        generic_params: &GenericParams,
        this_trait: &Path,
    ) -> Spanned<TypeId> {
        let ty = self.lower_node(
            ty,
            |_, ty| Type::Unknown.at(ty.range().to_span()),
            |this, ty| match ty {
                ast::Type::PathType(path_type) => this.lower_path_type(path_type, generic_params),
                ast::Type::ThisPathType(this_path_type) => {
                    this.lower_this_path_type(this_path_type, generic_params, this_trait)
                }
                _ => ice("unimplemented"),
            },
        );
        ty.map(|ty| self.tenv.insert(ty))
        // ty.map(|ty| match ty {
        //     Type::ThisPath(this_path) => {
        //         // this_path.resolve_type(self.interner);
        //         // let trt = self
        //         //     .item_resolver
        //         //     .resolve_path(&this_path.path_to_trait, self.module_id);
        //         // let assoc_types: &mut _ = todo!();
        //         // self.tenv.insert_with_trait_ctx(ty, assoc_types)
        //         todo!()
        //     }
        //     _ => self.tenv.insert(ty),
        // })
    }

    pub(crate) fn lower_name(&mut self, name: Option<ast::Name>) -> Spanned<Word> {
        self.lower_node(
            name,
            |this, name| {
                this.interner
                    .get_or_intern_static(POISONED_NAME)
                    .at(name.range().to_span())
            },
            |_, name| {
                let name = name
                    .ident()
                    .unwrap_or_else(|| ice("name parsed without identifier token"));
                let key = name.text_key().unwrap_or_else(|| ice("parsed empty name"));
                key.at(name.text_range().to_span())
            },
        )
    }

    pub(crate) fn lower_path(
        &mut self,
        path: Option<ast::Path>,
        generic_params: &GenericParams,
    ) -> Spanned<Path> {
        self.lower_node(
            path,
            |_, path| Path::poisoned().at(path.range().to_span()),
            |this, path| {
                let segments = path
                    .segments()
                    .map(|segment| {
                        segment
                            .text_key()
                            .unwrap_or_else(|| ice("text key contained no text"))
                    })
                    .collect();
                let generic_args = path
                    .generic_arg_list()
                    .map(|arg_list| {
                        arg_list
                            .args()
                            .map(|arg| this.lower_type(Some(arg), generic_params).inner)
                            .collect()
                    })
                    .unwrap_or(vec![]);
                Path::new(segments, generic_args).at(path.range().to_span())
            },
        )
    }

    // pub(crate) fn lower_expr(&mut self, expr: Option<ast::Expr>) -> Typed<ExprIdx> {
    //     todo!()
    // }
}
