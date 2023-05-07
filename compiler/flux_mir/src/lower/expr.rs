use super::*;
use flux_hir::hir::Typed;

impl<'a> FromHir for Typed<&'a hir::BinOp> {
    fn from_hir(&self, ctx: &LoweringCtx) -> ValRef {
        println!("HI");
        todo!()
    }
}

impl FromHir for Typed<u64> {
    fn from_hir(&self, ctx: &LoweringCtx) -> ValRef {
        todo!()
        // match ctx.tenv.reconstruct(self.tid) {
        //     flux_typesystem::TypeKind::Concrete(_) => todo!(),
        //     flux_typesystem::TypeKind::Int(_) => todo!(),
        //     flux_typesystem::TypeKind::Float(_) => todo!(),
        //     flux_typesystem::TypeKind::Ref(_) => todo!(),
        //     flux_typesystem::TypeKind::Generic(_, _) => todo!(),
        //     flux_typesystem::TypeKind::Unknown => todo!(),
        // }
    }
}
