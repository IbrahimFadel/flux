use flux_id::{id, Map};

use crate::r#type::Type;

pub struct TEnv {
    types: Map<id::Ty, Type>,
}

impl TEnv {
    pub fn insert(&mut self, ty: Type) -> id::Ty {
        self.types.insert(ty)
    }
}
