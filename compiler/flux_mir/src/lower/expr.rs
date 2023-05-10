use std::sync::Arc;

use crate::{
    hir_op_to_mir_op,
    value::{BinOp, Block, Value},
};

use super::*;

impl ToValue for hir::Block {
    type Variant = ();

    fn to_value(&self, ctx: &LoweringCtx) -> ValRef<Self::Variant> {
        let values = self.exprs.iter().map(|expr| expr.to_value(ctx)).collect();
        let block = Value::Block(Block::new(values));
        ValRef::new(Arc::new(block))
    }
}

impl<'a> ToValue for hir::BinOp {
    type Variant = ();

    fn to_value(&self, ctx: &LoweringCtx) -> ValRef<Self::Variant> {
        let lhs = self.lhs.to_value(ctx);
        let rhs = self.rhs.to_value(ctx);
        let op = hir_op_to_mir_op(self.op.inner);
        let binop = Value::BinOp(BinOp::new(lhs, op, rhs));
        ValRef::new(Arc::new(binop))
    }
}

impl ToValue for u64 {
    type Variant = ();

    fn to_value(&self, _ctx: &LoweringCtx) -> ValRef<Self::Variant> {
        ValRef::new(Arc::new(Value::Int(*self)))
    }
}

impl ToValue for hir::Let {
    type Variant = ();

    fn to_value(&self, _ctx: &LoweringCtx) -> ValRef<Self::Variant> {
        // TODO: some sort of scopes/hashmap for let
        // but to_value should return unit value
        ValRef::new(Arc::new(Value::Tuple(vec![])))
    }
}
