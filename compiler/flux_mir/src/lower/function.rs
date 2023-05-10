use std::sync::Arc;

use flux_diagnostics::ice;
use flux_hir::{hir::ExprIdx, FileItemTreeId, ModuleDefId};

use crate::{
    region::{Region, RegionData},
    value::{Lambda, Value},
};

use super::*;

struct FunctionBuilder<'a> {
    regions: Vec<Region>,
    function: &'a hir::Function,
    function_body: ExprIdx,
}

impl<'a> FunctionBuilder<'a> {
    pub fn new(function: &'a hir::Function, function_body: ExprIdx) -> Self {
        Self {
            regions: vec![],
            function,
            function_body,
        }
    }

    pub fn build(&mut self, ctx: &LoweringCtx) {
        let mut region_data = RegionData::new();

        self.function
            .params
            .iter()
            .for_each(|param| region_data.push_param(param.ty.to_type(ctx)));

        let body = self.function_body.to_value(ctx);

        println!("{:#?}", region_data);
        println!("{:#?}", body);
    }

    pub fn finish(self) -> ValRef<Lambda> {
        ValRef::new(Arc::new(Value::Lambda(Lambda {})))
    }
}

impl ToValue for FileItemTreeId<hir::Function> {
    type Variant = Lambda;

    fn to_value(&self, ctx: &LoweringCtx) -> ValRef<Self::Variant> {
        let function = &ctx.item_tree[*self];
        let body = ctx
            .bodies
            .indices
            .get(&(ctx.module_id, ModuleDefId::FunctionId(self.index)))
            .unwrap_or_else(|| ice("could not get function body when lowering to MIR"));
        let mut builder = FunctionBuilder::new(function, body.clone());
        builder.build(ctx);
        builder.finish()
    }
}
