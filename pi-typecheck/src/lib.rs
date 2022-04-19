use std::{borrow::BorrowMut, collections::HashMap, fmt::Result, ops::Range};

use pi_ast::{
	BinOp, BlockStmt, Expr, FloatLit, FnDecl, Ident, IntLit, OpKind, PrimitiveKind, PrimitiveType,
	Return, Stmt, StructType, TypeDecl, VarDecl, AST,
};
use pi_error::{filesystem::FileId, *};

struct TypecheckCtx<'a> {
	expecting_ty: Expr,
	errors: &'a mut Vec<PIError>,
	file_id: FileId,
	types: HashMap<String, TypeDecl>,
	var_types: HashMap<String, Expr>,
	cur_block: Option<BlockStmt>,
}

impl<'a> TypecheckCtx<'a> {
	fn error(&self, msg: String, code: PIErrorCode, labels: Vec<(String, Range<usize>)>) -> PIError {
		PIError::new(msg, code, labels, self.file_id)
	}

	pub fn check(&mut self, ast: &'a mut AST) {
		for ty in &ast.types {
			self.types.insert(ty.name.val.to_string(), ty.clone());
			// match &ty.type_ {
			// Expr::StructType(struct_ty) => {
			// 	self
			// 		.types
			// 		.insert(ty.name.val.to_string(), struct_ty.clone());
			// 	()
			// }
			// Expr::PrimitiveType(prim) => {
			// 	self.types.insert(ty.name.val.to_string(), prim.clone());
			// }
			// _ => (),
			// }
		}

		for f in &mut ast.functions {
			self.check_fn(f);
		}
	}

	fn check_fn(&mut self, f: &'a mut FnDecl) {
		self.cur_block = Some(f.block.clone());
		for stmt in &mut f.block {
			self.expecting_ty = f.ret_ty.clone();
			self.check_stmt(stmt);
		}
		self.var_types.clear();
	}

	fn check_stmt(&mut self, stmt: &mut Stmt) {
		match stmt {
			Stmt::VarDecl(var) => self.check_var(var),
			Stmt::Return(ret) => self.check_ret(ret),
			Stmt::ExprStmt(expr) => self.check_expr(expr),
			_ => (),
		}
	}

	fn check_ret(&mut self, ret: &mut Return) {
		if let Some(x) = &mut ret.val {
			self.check_expr(x);
		}
	}

	fn check_var(&mut self, var: &mut VarDecl) {
		self.expecting_ty = var.type_.clone();
		for name in &var.names {
			self
				.var_types
				.insert(name.val.to_string(), var.type_.clone());
		}

		if let Some(vals) = &mut var.values {
			for val in vals {
				self.check_expr(val);
			}
		}
	}

	fn check_expr(&mut self, expr: &mut Expr) {
		match expr {
			Expr::IntLit(int) => self.check_int_lit(int),
			Expr::FloatLit(float) => self.check_float_lit(float.borrow_mut()),
			Expr::BinOp(binop) => self.check_binop(binop),
			_ => (),
		}
	}

	fn check_binop(&mut self, binop: &mut BinOp) {
		self.check_expr(&mut *binop.x);
		if binop.op == OpKind::Eq {
			match &*binop.x {
				Expr::BinOp(b) => {
					self.expecting_ty = self.get_struct_access_type(b);
					println!("expecting: {:?}", self.expecting_ty);
				}
				_ => (),
			}
		}
		self.check_expr(&mut *binop.y);
	}

	fn get_struct_access_type(&mut self, binop: &BinOp) -> Expr {
		let mut b = binop;
		let mut field_names = vec![];
		if let Expr::Ident(rhs) = &*b.y {
			field_names.push(rhs);
		}
		while let Expr::BinOp(sub_binop) = &*b.x {
			if sub_binop.op != OpKind::Period {
				self.errors.push(self.error(
					"expected `.` operator in chained struct field access".to_owned(),
					PIErrorCode::TypecheckExpectedPeriodOpInChainedStructFieldAccess,
					vec![],
				));
			}
			if let Expr::Ident(rhs) = &*sub_binop.y {
				field_names.push(rhs);
			} else {
				self.errors.push(self.error(
					"expected rhs of `.` operator to be identifier".to_owned(),
					PIErrorCode::TypecheckExpectedRHSOfPeriodToBeIdent,
					vec![],
				));
			}
			b = sub_binop;
		}
		if let Expr::Ident(rhs) = &*b.x {
			field_names.push(rhs);
		}

		let struct_var_name = field_names.last_mut().cloned().unwrap();
		let struct_var_type_name = self.get_type_of_var_in_cur_block(&struct_var_name);
		field_names.pop();
		if let Expr::Ident(name) = struct_var_type_name {
			let struct_var_type = self.types.get(&name.val.to_string()).unwrap().type_.clone();
			if let Expr::StructType(struct_ty) = struct_var_type {
				return self.find_rightmost_field_type(&mut field_names, &struct_ty);
			} else {
				self.errors.push(self.error(
					"expected lhs of `.` operator to be a struct".to_owned(),
					PIErrorCode::TypecheckExpectedLHSOfPeriodToBeStruct,
					vec![],
				));
			}
		}
		panic!("this should be fatal");
	}

	fn find_rightmost_field_type(
		&self,
		field_names: &mut Vec<&Ident>,
		struct_ty: &StructType,
	) -> Expr {
		if field_names.len() == 0 {
			return Expr::StructType(struct_ty.clone());
		}
		let field_name = field_names.pop().unwrap();
		if let Some(field_ty) = self.get_struct_field_type(struct_ty, &field_name) {
			if let Expr::Ident(struct_type_name) = &field_ty {
				let res = self
					.types
					.get(&struct_type_name.val.to_string())
					.unwrap()
					.type_
					.clone();
				return match &res {
					Expr::StructType(struct_ty) => self.find_rightmost_field_type(field_names, &struct_ty),
					_ => res,
				};
			} else {
				return field_ty;
			}
		}
		panic!("cant thin of msg");
	}

	fn get_struct_field_type(&self, struct_ty: &StructType, field_name: &Ident) -> Option<Expr> {
		if let Some(field) = struct_ty.get(field_name) {
			return Some(field.type_.clone());
		}
		None
	}

	// fn get_struct_access_type(&mut self, binop: &BinOp) -> Expr {
	// let mut b = binop;
	// let mut b_afters = vec![binop];
	// while let Expr::BinOp(sub_b) = &*b.x {
	// 	if sub_b.op != OpKind::Period {
	// 		self.errors.push(self.error(
	// 			"expected `.` operator in chained struct field access".to_owned(),
	// 			PIErrorCode::TypecheckExpectedPeriodOpInChainedStructFieldAccess,
	// 			vec![],
	// 		));
	// 	}
	// 	b_afters.push(b);
	// 	b = sub_b;
	// }
	// let var_name = match &*b.x {
	// 	Expr::Ident(name) => name,
	// 	_ => panic!("just... ugh"),
	// };

	// let ty_name = self.get_type_of_var_in_cur_block(&var_name);
	// let mut ty = match ty_name {
	// 	Expr::Ident(name) => self
	// 		.types
	// 		.get(&name.val.to_string())
	// 		.expect("expected struct type"),
	// 	_ => panic!("ruh roh"),
	// };

	// let mut rhs = match &*b.y {
	// 	Expr::Ident(name) => name.val.to_string(),
	// 	_ => panic!(":(((("),
	// };

	// let mut final_expr = Expr::Error;
	// while b_afters.len() > 0 {
	// 	// loop {
	// 	println!("{rhs}");
	// 	let ty_expr = match &ty.type_ {
	// 		Expr::StructType(struct_ty) => self.get_struct_field_type(struct_ty, &rhs),
	// 		Expr::PrimitiveType(_) => ty.type_.clone(),
	// 		_ => panic!("unexpected type"),
	// 	};
	// 	println!("{:?}", ty_expr);
	// 	// if let Expr::StructType(_) = ty_expr {
	// 	// 	if b_afters.len() == 0 {
	// 	// 		break;
	// 	// 	}
	// 	// }
	// 	match &ty_expr {
	// 		Expr::Ident(ident) => {
	// 			rhs = self.get_ident_val_from_expr(&*b_afters.last().unwrap().y);
	// 			ty = self
	// 				.types
	// 				.get(&ident.val.to_string())
	// 				.expect("expected struct type");
	// 			b_afters.pop();
	// 		}
	// 		x => {
	// 			final_expr = x.clone();
	// 			break;
	// 		}
	// 	};
	// }
	// return final_expr;
	// }

	// #[inline(always)]
	// fn get_ident_val_from_expr(&self, e: &Expr) -> String {
	// 	match e {
	// 		Expr::Ident(ident) => ident.val.to_string(),
	// 		_ => panic!("expected identifier expression"),
	// 	}
	// }

	// index map
	// name -> FieldData
	// Field Data { mut: bool, type: Expr }

	// #[inline(always)]
	// fn get_struct_field_type(&self, struct_ty: &StructType, field_name: &String) -> Expr {
	// 	println!("{:?} {}", struct_ty, field_name);
	// 	for field in struct_ty {
	// 		if field.name.val == field_name {
	// 			return field.type_.clone();
	// 		}
	// 	}
	// 	panic!("bruh");
	// }

	fn get_type_of_var_in_cur_block(&self, name: &Ident) -> &Expr {
		self
			.var_types
			.get(&name.val.to_string())
			.expect("expected var with name")
	}

	fn check_float_lit(&mut self, float: &mut FloatLit) {
		if let Some(prim) = TypecheckCtx::type_is_primitive(&self.expecting_ty) {
			let expected_bits = TypecheckCtx::primitive_kind_to_bits(&prim.kind);
			if float.bits != expected_bits {
				float.bits = expected_bits;
			}
		}
	}

	fn check_int_lit(&mut self, int: &mut IntLit) {
		if let Some(prim) = TypecheckCtx::type_is_primitive(&self.expecting_ty) {
			let expected_bits = TypecheckCtx::primitive_kind_to_bits(&prim.kind);
			let expected_signed = TypecheckCtx::primitive_kind_to_signedness(&prim.kind);
			if int.bits != expected_bits {
				int.bits = expected_bits;
			}
			if expected_signed == false && int.signed == true {
				let mut labels = vec![("expected unsigned integer".to_owned(), int.val_span.clone())];
				if expected_signed == false {
					labels.push((format!("unexpected `-`"), int.sign_span.clone()))
				}
				self.errors.push(self.error(
					format!("expected unsigned integer but got signed integer",),
					PIErrorCode::TypecheckUnexpectedSignednessInIntLit,
					labels,
				));
			}
		}
	}

	fn type_is_primitive(ty: &Expr) -> Option<&PrimitiveType> {
		match ty {
			Expr::PrimitiveType(prim) => Some(prim),
			_ => None,
		}
	}

	fn primitive_kind_to_bits(prim: &PrimitiveKind) -> u8 {
		match prim {
			PrimitiveKind::U64 | PrimitiveKind::I64 | PrimitiveKind::F64 => 64,
			PrimitiveKind::U32 | PrimitiveKind::I32 | PrimitiveKind::F32 => 32,
			PrimitiveKind::U16 | PrimitiveKind::I16 => 16,
			PrimitiveKind::U8 | PrimitiveKind::I8 => 8,
			_ => 32,
		}
	}

	fn primitive_kind_to_signedness(prim: &PrimitiveKind) -> bool {
		match prim {
			PrimitiveKind::U64 | PrimitiveKind::U32 | PrimitiveKind::U16 | PrimitiveKind::U8 => false,
			PrimitiveKind::I64
			| PrimitiveKind::I32
			| PrimitiveKind::I16
			| PrimitiveKind::I8
			| PrimitiveKind::F64
			| PrimitiveKind::F32 => true,
			_ => true,
		}
	}
}

pub fn typecheck_ast(file_ast_map: &mut HashMap<FileId, AST>) -> Vec<PIError> {
	let entry_fileid: FileId = FileId(0);
	let ast = file_ast_map
		.get_mut(&entry_fileid)
		.expect("could not get file");
	let mut errors = vec![];
	let mut ctx = TypecheckCtx {
		expecting_ty: Expr::PrimitiveType(PrimitiveType::new(PrimitiveKind::I32)),
		errors: &mut errors,
		file_id: entry_fileid,
		types: HashMap::new(),
		var_types: HashMap::new(),
		cur_block: None,
	};

	ctx.check(ast);

	return errors;
}
