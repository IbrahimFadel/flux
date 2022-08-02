// use std::{cell::RefCell, rc::Rc};

// use crate::mir::{Region, Value};

// pub(crate) struct MirBuilder {
// 	region: Region,
// }

// impl MirBuilder {
// 	pub fn new(region: Region) -> Self {
// 		Self { region }
// 	}

// 	// pub fn set_region(&mut self, region: Rc<RefCell<Region>>) {
// 	// 	self.region = region;
// 	// }

// 	// pub fn next_value(&self) -> VarId {
// 	// 	self.region.borrow().values.len()
// 	// }

// 	// pub fn push_input(&mut self, id: VarId) {
// 	// 	self.region.borrow_mut().values.push(Value::Var(id));
// 	// }
// }
