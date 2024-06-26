use crate::hir::{Generic, ThisPath, TypeBound, TypeBoundList};

use super::*;

impl<'a> LowerCtx<'a> {
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
        this_trait: &Path,
    ) -> Spanned<Type> {
        let path = self.lower_path(this_path_type.path(), generic_params);
        path.map(|path| Type::ThisPath(ThisPath::new(path, this_trait.clone())))
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
