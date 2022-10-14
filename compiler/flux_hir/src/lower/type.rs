use crate::hir::Type;

use super::*;

// type TypeResult = Result<Spanned<Type>, LoweringDiagnostic>;
type TypeResult = Spanned<Type>;

impl LoweringCtx {
    pub(crate) fn lower_type(&mut self, ty: ast::Type) -> TypeResult {
        match ty {
            ast::Type::PathType(path_type) => self.lower_path_type(path_type),
            _ => todo!(),
        }
    }

    fn lower_path_type(&mut self, path_ty: ast::PathType) -> TypeResult {
        // we could use `self.unwrap` but i'm pretty sure it's literally impossible for this to fail, and i'd rather this ICE than give a pretty error to the user
        let path = path_ty
            .path()
            .expect("internal compiler error: path type does not contain path");
        let ty = Type::Path(self.lower_path(path.segments()));
        Spanned::new(ty, self.span_node(&path_ty))
    }
}
