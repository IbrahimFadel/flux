use std::collections::HashMap;

use flux_span::FileId;
use la_arena::{Arena, Idx};
use lasso::Spur;

pub type ModuleId = Idx<ModuleData>;

pub struct ModuleTree {
    root: ModuleId,
    modules: Arena<ModuleData>,
}

impl ModuleTree {
    pub fn new() -> Self {
        todo!()
        // Self { root: , modules: () }
    }
}

pub struct ModuleData {
    parent: Option<ModuleId>,
    children: HashMap<Spur, ModuleId>,
    file_id: FileId,
}

impl ModuleData {
    pub fn new(
        parent: Option<ModuleId>,
        children: HashMap<Spur, ModuleId>,
        file_id: FileId,
    ) -> Self {
        Self {
            parent,
            children,
            file_id,
        }
    }
}
