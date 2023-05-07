use flux_hir::{
    hir::{self, Typed, WithType},
    LoweredBodies,
};

use crate::value::ValRef;

mod expr;

pub struct LoweringCtx<'a> {
    bodies: &'a LoweredBodies,
}

pub trait FromHir {
    fn from_hir(&self, ctx: &LoweringCtx) -> ValRef;
}

impl FromHir for Typed<hir::ExprIdx> {
    fn from_hir(&self, ctx: &LoweringCtx) -> ValRef {
        (&ctx.bodies.exprs[self.raw()])
            .with_type(self.tid)
            .from_hir(ctx)
    }
}

impl<'a> FromHir for Typed<&'a hir::Expr> {
    fn from_hir(&self, ctx: &LoweringCtx) -> ValRef {
        match &self.expr {
            hir::Expr::BinOp(bin_op) => bin_op.with_type(self.tid).from_hir(ctx),
            hir::Expr::Int(int) => int.with_type(self.tid).from_hir(ctx),
            _ => todo!("{:#?}", self),
        }
    }
}
