use lasso::Spur;
use std::collections::HashMap;

use crate::TypeId;

#[derive(Debug, Default, Clone)]
pub struct Scope {
    locals: HashMap<Spur, TypeId>,
}

impl Scope {
    pub fn new() -> Self {
        Self {
            locals: HashMap::new(),
        }
    }

    pub fn insert_local(&mut self, name: Spur, ty: TypeId) {
        self.locals.insert(name, ty);
    }

    pub fn try_get_local(&self, name: &Spur) -> Option<TypeId> {
        self.locals.get(name).cloned()
    }

    pub fn get_local(&self, name: &Spur) -> TypeId {
        self.locals[name]
    }
}
