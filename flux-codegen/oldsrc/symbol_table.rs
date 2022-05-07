// use std::collections::HashMap;

// use llvm_sys::prelude::LLVMValueRef;

// #[derive(Debug)]
// pub struct SymbolTable {
// 	pub scope_conns: Vec<(String, String)>,
// 	pub scope_values: HashMap<String, HashMap<String, LLVMValueRef>>,
// 	pub cur_scope: String,
// }

// impl SymbolTable {
// 	pub fn new() -> Self {
// 		Self {
// 			scope_conns: vec![],
// 			scope_values: HashMap::from([(String::from("entry"), HashMap::new())]),
// 			cur_scope: String::from("entry"),
// 		}
// 	}

// 	pub fn clear(&mut self) {
// 		self.scope_conns.clear();
// 		self.scope_values = HashMap::from([(String::from("entry"), HashMap::new())]);
// 		self.cur_scope = String::from("entry");
// 	}

// 	pub fn get_value_in_scope(&self, scope: &String, name: &String) -> Option<LLVMValueRef> {
// 		let t = self.scope_values.get(scope);
// 		match t {
// 			Some(val_map) => match val_map.get(name) {
// 				Some(x) => Some(*x),
// 				_ => None,
// 			},
// 			_ => None,
// 		}
// 	}

// 	pub fn set_value_in_cur_scope(&mut self, name: String, val: LLVMValueRef) {
// 		if let Some(scope) = self.scope_values.get_mut(&self.cur_scope) {
// 			scope.insert(name, val);
// 		}
// 	}

// 	pub fn get_scopes_outside_scope(&self, scope: &String) -> Vec<String> {
// 		let mut scopes = vec![];
// 		for conn in &self.scope_conns {
// 			if conn.1 == *scope {
// 				scopes.push(conn.0.clone());
// 				scopes.append(&mut self.get_scopes_outside_scope(&conn.0));
// 			}
// 		}
// 		return scopes;
// 	}

// 	pub fn find_val_in_scope(&self, scope: &String, name: &String) -> Option<LLVMValueRef> {
// 		let v = self.get_value_in_scope(&scope, &name);
// 		if v.is_none() {
// 			let outer_scopes = self.get_scopes_outside_scope(&scope);
// 			for s in outer_scopes {
// 				if let Some(x) = self.find_val_in_scope(&s, name) {
// 					return Some(x);
// 				}
// 			}
// 			None
// 		} else {
// 			return v;
// 		}
// 	}
// }
