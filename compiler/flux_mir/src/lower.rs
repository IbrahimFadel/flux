use flux_hir::{hir, ItemTree, LoweredBodies, ModuleId};
use lasso::ThreadedRodeo;

use crate::{r#type::TypeRef, value::ValRef};

mod expr;
mod function;
mod r#type;

pub struct LoweringCtx<'a> {
    bodies: &'a LoweredBodies,
    item_tree: &'a ItemTree,
    module_id: ModuleId,
    string_interner: &'static ThreadedRodeo,
}

impl<'a> LoweringCtx<'a> {
    pub fn new(
        bodies: &'a LoweredBodies,
        item_tree: &'a ItemTree,
        module_id: ModuleId,
        string_interner: &'static ThreadedRodeo,
    ) -> Self {
        Self {
            bodies,
            item_tree,
            module_id,
            string_interner,
        }
    }
}

pub trait ToValue {
    type Variant;

    fn to_value(&self, ctx: &LoweringCtx) -> ValRef<Self::Variant>;
}

pub trait ToType {
    type Variant;

    fn to_type(&self, ctx: &LoweringCtx) -> TypeRef<Self::Variant>;
}

impl ToValue for hir::ExprIdx {
    type Variant = ();

    fn to_value(&self, ctx: &LoweringCtx) -> ValRef<Self::Variant> {
        (&ctx.bodies.exprs[self.raw()]).to_value(ctx)
    }
}

impl ToValue for hir::Expr {
    type Variant = ();

    fn to_value(&self, ctx: &LoweringCtx) -> ValRef<Self::Variant> {
        match self {
            hir::Expr::Block(block) => block.to_value(ctx),
            hir::Expr::BinOp(bin_op) => bin_op.to_value(ctx),
            hir::Expr::Enum(_) => todo!(),
            hir::Expr::Call(_) => todo!(),
            hir::Expr::Float(_) => todo!(),
            hir::Expr::Int(int) => int.to_value(ctx),
            hir::Expr::Tuple(_) => todo!(),
            hir::Expr::Path(_) => todo!(),
            hir::Expr::Let(let_e) => let_e.to_value(ctx),
            hir::Expr::Struct(_) => todo!(),
            hir::Expr::MemberAccess(_) => todo!(),
            hir::Expr::Poisoned => todo!(),
        }
    }
}
