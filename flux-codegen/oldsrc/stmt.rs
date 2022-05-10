// use super::*;

// impl<'a> Codegen<'a> {
// 	pub fn block_stmt(&mut self, block: &BlockStmt) -> Option<FluxError> {
// 		for stmt in block {
// 			if let Some(err) = self.stmt(stmt) {
// 				return Some(err);
// 			}
// 		}
// 		return None;
// 	}

// 	fn stmt(&mut self, stmt: &Stmt) -> Option<FluxError> {
// 		match stmt {
// 			Stmt::If(if_) => self.if_stmt(if_),
// 			Stmt::VarDecl(var) => self.var_decl(var),
// 			Stmt::Return(ret) => self.ret_stmt(ret),
// 			Stmt::ExprStmt(expr) => self.expr(expr).1,
// 			_ => Some(self.error(
// 				format!(
// 					"internal compiler error: could not codegen statement: {:?}",
// 					stmt
// 				),
// 				FluxErrorCode::CodegenCouldNotCodegenStmt,
// 				vec![],
// 			)),
// 		}
// 	}

// 	fn if_stmt(&mut self, if_: &If) -> Option<FluxError> {
// 		let (cond, err) = self.expr(&if_.condition);
// 		if let Some(err) = err {
// 			return Some(err);
// 		}

// 		unsafe {
// 			let f = self
// 				.tmp_data
// 				.cur_fn
// 				.expect("expected to be in function: internal compiler error");
// 			let then = LLVMAppendBasicBlock(f, str_to_cstring("then"));
// 			let else_ = LLVMCreateBasicBlockInContext(self.ctx, str_to_cstring("else"));
// 			let mut cont: LLVMBasicBlockRef = ptr::null_mut();
// 			if if_.else_.is_some() {
// 				cont = LLVMCreateBasicBlockInContext(self.ctx, str_to_cstring("continue"));
// 				self.tmp_data.merging_bb = Some(cont);
// 			} else {
// 				self.tmp_data.merging_bb = Some(else_);
// 			}

// 			LLVMBuildCondBr(self.builder, cond, then, else_);

// 			self.tmp_data.cur_bb = Some(then);
// 			LLVMPositionBuilderAtEnd(self.builder, then);

// 			let name = get_basicblock_name(then);
// 			let initial_scope = self.symbol_table.cur_scope.clone();
// 			self
// 				.symbol_table
// 				.scope_conns
// 				.push((initial_scope.clone(), name.clone()));
// 			self
// 				.symbol_table
// 				.scope_values
// 				.insert(name.clone(), HashMap::new());
// 			self.symbol_table.cur_scope = name;

// 			if let Some(err) = self.block_stmt(&if_.then) {
// 				return Some(err);
// 			}
// 			if if_.else_.is_some() {
// 				LLVMBuildBr(self.builder, cont);
// 			} else {
// 				LLVMBuildBr(self.builder, else_);
// 			}

// 			self.tmp_data.cur_bb = Some(else_);
// 			LLVMPositionBuilderAtEnd(self.builder, else_);

// 			if let Some(else_block) = &if_.else_ {
// 				let name = get_basicblock_name(else_);
// 				self
// 					.symbol_table
// 					.scope_conns
// 					.push((initial_scope, name.clone()));
// 				self
// 					.symbol_table
// 					.scope_values
// 					.insert(name.clone(), HashMap::new());
// 				self.symbol_table.cur_scope = name;

// 				LLVMAppendExistingBasicBlock(f, else_);
// 				self.block_stmt(else_block);

// 				LLVMBuildBr(self.builder, cont);
// 				LLVMAppendExistingBasicBlock(f, cont);
// 				self.tmp_data.cur_bb = Some(cont);
// 				LLVMPositionBuilderAtEnd(self.builder, cont);
// 			} else {
// 				LLVMAppendExistingBasicBlock(f, else_);
// 				LLVMPositionBuilderAtEnd(self.builder, else_);
// 			}
// 		}

// 		return None;
// 	}

// 	fn ret_stmt(&mut self, ret: &Return) -> Option<FluxError> {
// 		unsafe {
// 			match &ret.val {
// 				Some(x) => {
// 					let (e, err) = self.expr(&x);
// 					if let Some(err) = err {
// 						return Some(err);
// 					}
// 					LLVMBuildRet(self.builder, e);
// 					return None;
// 				}
// 				None => {
// 					LLVMBuildRetVoid(self.builder);
// 					return None;
// 				}
// 			};
// 		}
// 	}
// }
