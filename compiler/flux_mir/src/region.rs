use std::sync::Arc;

use crate::value::{TyArr, Value};

pub struct Region(Arc<RegionData>);

pub struct RegionData {
    parents: Vec<Region>,
    param_tys: TyArr,
    nodes: Vec<Value>,
}
