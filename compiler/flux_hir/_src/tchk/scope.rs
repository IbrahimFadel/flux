use hashbrown::HashMap;
use lasso::Spur;

use super::r#type::{Type, TypeId};

pub(super) struct Scope {
    locals: HashMap<Spur, Type>,
}

impl Scope {
    pub fn new() -> Self {
        Self {
            locals: HashMap::new(),
        }
    }
}
