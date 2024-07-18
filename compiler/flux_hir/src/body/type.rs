use flux_span::Span;

use crate::hir::{ArrayType, Generic, ThisPath, TypeBound, TypeBoundList};

use super::*;
use flux_typesystem as ts;

impl<'a, 'pkgs> LowerCtx<'a, 'pkgs> {
    #[inline]
    pub(crate) fn insert_int_type(&mut self, span: Span) -> TypeId {
        self.tckh
            .tenv
            .insert(ts::TypeKind::Int(None).file_span(self.file_id, span))
    }

    #[inline]
    pub(crate) fn insert_unit(&mut self, span: Span) -> TypeId {
        self.tckh.tenv.insert(
            ts::TypeKind::Concrete(ts::ConcreteKind::Tuple(vec![])).file_span(self.file_id, span),
        )
    }

    #[inline]
    pub(crate) fn insert_unknown(&mut self, span: Span) -> TypeId {
        self.tckh
            .tenv
            .insert(ts::TypeKind::Unknown.file_span(self.file_id, span))
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
                ast::Type::TupleType(tuple_type) => {
                    this.lower_tuple_type(tuple_type, generic_params)
                }
                ast::Type::ArrayType(arr_type) => this.lower_arr_type(arr_type, generic_params),
                ast::Type::PtrType(ptr_type) => this.lower_ptr_type(ptr_type, generic_params),
                ast::Type::ThisPathType(this_path_type) => {
                    this.lower_this_path_type(this_path_type, generic_params)
                    // ice("should not encounter this path outside of trait method")
                }
            },
        );
        let span = ty.span;
        self.tckh.tenv.insert(ty.in_file(self.file_id)).at(span)
    }

    pub(super) fn lower_path_type(
        &mut self,
        path_type: ast::PathType,
        generic_params: &GenericParams,
    ) -> Spanned<Type> {
        let path = self.lower_path(path_type.path(), generic_params);
        let ty = if path.is_generic(generic_params) {
            let name = path.map(|path| *path.get(0));
            let key = name.inner;
            Type::Generic(Generic::new(
                name,
                get_trait_restrictions_on_generic(&key, generic_params),
            ))
        } else {
            Type::Path(path.inner)
        };
        ty.at(path_type.range().to_span())
    }

    pub(super) fn lower_this_path_type(
        &mut self,
        this_path_type: ast::ThisPathType,
        generic_params: &GenericParams,
    ) -> Spanned<Type> {
        let path = self.lower_path(this_path_type.path(), generic_params);
        path.map(|path| Type::ThisPath(ThisPath::new(path)))
    }

    fn lower_tuple_type(
        &mut self,
        tuple_type: ast::TupleType,
        generic_params: &GenericParams,
    ) -> Spanned<Type> {
        let types = tuple_type
            .types()
            .map(|ty| self.lower_type(Some(ty), generic_params).inner)
            .collect();
        Type::Tuple(types).at(tuple_type.range().to_span())
    }

    fn lower_arr_type(
        &mut self,
        arr_type: ast::ArrayType,
        generic_params: &GenericParams,
    ) -> Spanned<Type> {
        let ty = self.lower_type(arr_type.ty(), generic_params);
        let n = self.lower_int_expr(arr_type.n().unwrap());
        let n = match self.package_bodies.exprs[n.idx()] {
            Expr::Int(n) => n,
            _ => unreachable!(),
        };
        Type::Array(ArrayType::new(ty.inner, n)).at(arr_type.range().to_span())
    }

    fn lower_ptr_type(
        &mut self,
        ptr_type: ast::PtrType,
        generic_params: &GenericParams,
    ) -> Spanned<Type> {
        let to_ty = self.lower_type(ptr_type.ty(), generic_params);
        Type::Ptr(to_ty.inner).at(ptr_type.range().to_span())
    }
}

fn get_trait_restrictions_on_generic(name: &Word, generic_params: &GenericParams) -> TypeBoundList {
    let bounds = generic_params
        .where_predicates
        .iter()
        .filter_map(|predicate| {
            if predicate.name == *name {
                Some(
                    predicate
                        .bound
                        .map_ref(|bound| TypeBound::new(bound.clone())),
                )
            } else {
                None
            }
        });
    TypeBoundList::new(bounds.collect())
}
