use flux_diagnostics::ice;
use flux_parser::ast::{self, AstNode};
use flux_typesystem::{Generic, ThisCtx, Type};
use flux_util::{Interner, Path, Spanned, ToSpan, WithSpan, Word};
use tracing::warn;

use crate::def::GenericParams;

use super::lower_node;

pub(super) struct LoweringCtx {
    this_ctx: ThisCtx,
    interner: &'static Interner,
}

impl LoweringCtx {
    const POISONED_NAME: &'static str = "poisoned";

    pub(super) fn new(this_ctx: ThisCtx, interner: &'static Interner) -> Self {
        Self { this_ctx, interner }
    }

    pub(super) fn lower_name(&self, name: Option<ast::Name>) -> Spanned<Word> {
        lower_node(
            self,
            name,
            |this, name| {
                this.interner
                    .get_or_intern_static(Self::POISONED_NAME)
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

    pub(super) fn lower_path(
        &self,
        path: Option<ast::Path>,
        generic_params: &GenericParams,
    ) -> Spanned<Path<Word, Type>> {
        lower_node(
            self,
            path,
            |_, path| Path::empty().at(path.range().to_span()),
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

    pub(super) fn lower_type(
        &self,
        ty: Option<ast::Type>,
        generic_params: &GenericParams,
    ) -> Spanned<Type> {
        lower_node(
            self,
            ty,
            |_, ty| Type::Unknown.at(ty.range().to_span()),
            |this, ty| match ty {
                ast::Type::PathType(path_type) => this.lower_path_type(path_type, generic_params),
                ast::Type::TupleType(tuple_type) => {
                    this.lower_tuple_type(tuple_type, generic_params)
                }
                ast::Type::ArrayType(arr_type) => this.lower_arr_type(arr_type, generic_params),
                ast::Type::PtrType(ptr_type) => this.lower_ptr_type(ptr_type, generic_params),
                ast::Type::ThisPathType(this_path_type) => {
                    this.lower_this_path_type(this_path_type, generic_params)
                }
            },
        )
    }

    fn lower_path_type(
        &self,
        path_type: ast::PathType,
        generic_params: &GenericParams,
    ) -> Spanned<Type> {
        let path = self.lower_path(path_type.path(), generic_params).inner;
        let ty = if generic_params.is_path_generic(&path) {
            let name = path.get_nth(0);
            Type::Generic(Generic::new(
                *name,
                generic_params.get_bounds_on_generic(name),
            ))
        } else {
            Type::path(path)
        };
        ty.at(path_type.range().to_span())
    }

    fn lower_tuple_type(
        &self,
        tuple_type: ast::TupleType,
        generic_params: &GenericParams,
    ) -> Spanned<Type> {
        let types = tuple_type
            .types()
            .map(|ty| self.lower_type(Some(ty), generic_params).inner)
            .collect();
        Type::tuple(types).at(tuple_type.range().to_span())
    }

    fn lower_arr_type(
        &self,
        arr_type: ast::ArrayType,
        generic_params: &GenericParams,
    ) -> Spanned<Type> {
        let ty = self.lower_type(arr_type.ty(), generic_params).inner;
        warn!("hehehehhe unwrap go brr");
        let n: u64 = self
            .interner
            .resolve(&arr_type.n().unwrap().v().unwrap().text_key().unwrap())
            .parse()
            .unwrap();
        Type::array(ty, n).at(arr_type.range().to_span())
    }

    fn lower_ptr_type(
        &self,
        ptr_type: ast::PtrType,
        generic_params: &GenericParams,
    ) -> Spanned<Type> {
        let ty = self.lower_type(ptr_type.ty(), generic_params).inner;
        Type::ptr(ty).at(ptr_type.range().to_span())
    }

    fn lower_this_path_type(
        &self,
        this_path_type: ast::ThisPathType,
        generic_params: &GenericParams,
    ) -> Spanned<Type> {
        let path = self.lower_path(this_path_type.path(), generic_params);
        path.map(|path| Type::this_path(path, self.this_ctx.clone()))
    }
}
