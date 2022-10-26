use crate::hir::{GenericParamList, Type};

use super::*;

// type TypeResult = Result<Spanned<Type>, LoweringDiagnostic>;
type TypeResult = Spanned<Type>;

impl LoweringCtx {
    pub(crate) fn lower_type(
        &mut self,
        ty: ast::Type,
        generic_param_list: &GenericParamList,
    ) -> TypeResult {
        match ty {
            ast::Type::PathType(path_type) => {
                self.lower_path_or_generic_type(path_type, generic_param_list)
            }
            _ => todo!(),
        }
    }

    fn lower_path_or_generic_type(
        &mut self,
        path_ty: ast::PathType,
        generic_param_list: &GenericParamList,
    ) -> TypeResult {
        // we could use `self.unwrap` but i'm pretty sure it's literally impossible for this to fail, and i'd rather this ICE than give a pretty error to the user
        let path = path_ty
            .path()
            .expect("internal compiler error: path type does not contain path");
        let path = self.lower_path(path.segments());

        let ty = if path.len() > 1 {
            Type::Path(path)
        } else if generic_param_list
            .get(path.nth(0).expect("internal compiler error: path is empty"))
        {
            Type::Generic
        } else {
            Type::Path(path)
        };

        Spanned::new(ty, self.span_node(&path_ty))
    }
}
