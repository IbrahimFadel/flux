use std::sync::Arc;

use crate::{r#type::TypeRef, value::Value};

#[derive(Debug)]
pub struct Region(Arc<RegionData>);

impl Region {
    pub fn new(data: Arc<RegionData>) -> Self {
        Self(data)
    }
}

#[derive(Debug)]
pub struct RegionData {
    parents: Vec<Region>,
    param_tys: Vec<TypeRef>,
    nodes: Vec<Value>,
}

impl RegionData {
    pub fn new() -> Self {
        Self {
            parents: vec![],
            param_tys: vec![],
            nodes: vec![],
        }
    }

    pub fn push_param(&mut self, ty: TypeRef) {
        self.param_tys.push(ty);
    }
}
