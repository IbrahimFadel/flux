use std::{borrow::BorrowMut, collections::HashMap, ops::Range};

use indexmap::IndexMap;
use pi_ast::{
	ApplyBlock, BinOp, BlockStmt, CallExpr, Expr, Field, FloatLit, FnDecl, Ident, IntLit,
	InterfaceType, Method, OpKind, PrimitiveKind, PrimitiveType, Return, Stmt, StructExpr,
	StructType, TypeDecl, Unary, VarDecl, AST,
};
use pi_error::{filesystem::FileId, *};

struct TypecheckCtx {
	expecting_ty: Expr,
	file_id: FileId,
	types: HashMap<String, TypeDecl>,
	var_types: HashMap<String, Expr>,
	cur_block: Option<BlockStmt>,
}

impl TypecheckCtx {
	fn error(&self, msg: String, code: PIErrorCode, labels: Vec<(String, Range<usize>)>) -> PIError {
		PIError::new(msg, code, labels, self.file_id)
	}

	pub fn check(&mut self, ast: &mut AST) -> Option<PIError> {
		self.edit_interface_this_types(&mut ast.types);
		for ty in &ast.types {
			self.types.insert(ty.name.val.to_string(), ty.clone());
		}

		for apply_block in &mut ast.apply_blocks {
			if let Some(err) = self.check_apply(apply_block, &mut ast.struct_implementations) {
				return Some(err);
			}
		}

		for f in &mut ast.functions {
			if let Some(err) = self.check_fn(f) {
				return Some(err);
			}
		}

		return None;
	}

	fn edit_interface_this_types(&self, types: &mut Vec<TypeDecl>) {
		for ty in types {
			if let Expr::InterfaceType(interface_ty) = &mut ty.type_ {
				for (_, method) in interface_ty {
					if method.params.len() > 0 {
						if method.params[0].name.val == "this" {
							method.params[0].type_ = Expr::PtrType(Box::from(Expr::Ident(ty.name.clone())));
						}
					}
				}
			}
		}
	}

	fn check_apply(
		&mut self,
		apply_block: &mut ApplyBlock,
		struct_implementations_map: &mut HashMap<Ident, Vec<TypeDecl>>,
	) -> Option<PIError> {
		for f in &mut apply_block.methods {
			if let Some(res) = self.check_fn(f) {
				return Some(res);
			}

			if f.params.len() > 0 {
				if f.params[0].name.val == "this" {
					f.params[0].type_ = Expr::PtrType(Box::from(Expr::Ident(Ident::new(
						0..0,
						apply_block.struct_name.val.clone(),
					))));
				}
			}
		}
		if apply_block.interface_name.is_none() {
			return None;
		}

		let interface_name = apply_block.interface_name.as_ref().unwrap();
		if let Some(ty_decl) = self.types.get(&interface_name.val.to_string()) {
			if let Expr::InterfaceType(interface_ty) = &ty_decl.type_ {
				if let Some(err) = self.compare_interface_methods_to_apply_block_methods(
					interface_name,
					interface_ty,
					&apply_block.methods,
				) {
					return Some(err);
				} else {
					struct_implementations_map
						.entry(apply_block.struct_name.clone())
						.or_insert(Vec::new())
						.push(ty_decl.clone());
				}
			} else {
				return Some(self.error(
					format!(
						"expected `{}` to be an interface",
						interface_name.val.to_string()
					),
					PIErrorCode::TypecheckExpectedTypeToBeInterface,
					vec![],
				));
			}
		} else {
			return Some(self.error(
				format!(
					"could not find type with name `{}`",
					interface_name.val.to_string()
				),
				PIErrorCode::TypecheckCouldNotFindType,
				vec![],
			));
		}

		return None;
	}

	fn compare_interface_methods_to_apply_block_methods(
		&self,
		interface_name: &Ident, // only needed for error reporting
		interface_ty: &InterfaceType,
		methods: &Vec<FnDecl>,
	) -> Option<PIError> {
		if methods.len() != interface_ty.len() {
			return Some(self.error(
				format!(
					"not all methods of `{}` were implemented in apply block",
					interface_name.val.to_string(),
				),
				PIErrorCode::TypecheckNotAllInterfaceMethodsImplemented,
				vec![],
			));
		}
		for method in methods {
			if let Some(interface_method) = interface_ty.get(&method.name) {
				if let Some(err) =
					self.compare_interface_method_pubs(interface_name, interface_method, method)
				{
					return Some(err);
				} else if let Some(err) =
					self.compare_interface_method_return_types(interface_name, interface_method, method)
				{
					return Some(err);
				} else if let Some(err) =
					self.compare_interface_method_param_types(interface_name, interface_method, method)
				{
					return Some(err);
				}
			} else {
				return Some(self.error(
					format!(
						"method `{}` could not be found in interface `{}`",
						method.name.val.to_string(),
						interface_name.val.to_string(),
					),
					PIErrorCode::TypecheckCouldNotFindMethodInInterface,
					vec![],
				));
			}
		}

		return None;
	}

	fn compare_interface_method_pubs(
		&self,
		interface_name: &Ident,
		interface_method: &Method,
		method: &FnDecl,
	) -> Option<PIError> {
		let visibility1 = match interface_method.pub_ {
			true => "public",
			false => "private",
		};
		let visibility2 = match method.pub_ {
			true => "public",
			false => "private",
		};
		if visibility1 != visibility2 {
			return Some(self.error(
				format!("interface method visibilities do not match"),
				PIErrorCode::TypecheckInterfaceMethodVisibilitiesDontMatch,
				vec![
					(
						format!(
							"`{}` method `{}` defined as {}",
							interface_name.val.to_string(),
							interface_method.name.val.to_string(),
							visibility1
						),
						interface_method.pub_span.clone(),
					),
					(
						format!("but defined as {} in apply block", visibility2),
						method.pub_span.clone(),
					),
				],
			));
		}
		return None;
	}

	fn compare_interface_method_param_types(
		&self,
		interface_name: &Ident,
		interface_method: &Method,
		method: &FnDecl,
	) -> Option<PIError> {
		let mut labels: Vec<(String, Range<usize>)> = vec![(
			format!("method parameters in appy block do not match the interface method definition"),
			interface_method.params_span.clone(),
		)];
		let method_params_len = method.params.len();
		let mut i = 0;
		for interface_param in &interface_method.params {
			if method_params_len >= i + 1 {
				let i_suffix_str = match i + 1 {
					1 => "st",
					2 => "nd",
					3 => "rd",
					_ => "th",
				};
				let mutability1 = match interface_param.mut_ {
					true => "mutable",
					false => "immutable",
				};
				let mutability2 = match method.params[i].mut_ {
					true => "mutable",
					false => "immutable",
				};
				if mutability1 != mutability2 {
					labels.push((
						format!(
							"expected {}{} parameter to be {}",
							i + 1,
							i_suffix_str,
							mutability1
						),
						interface_param.mut_span.clone(),
					));
					labels.push((
						format!(
							"instead got {} {}{} parameter",
							mutability2,
							i + 1,
							i_suffix_str,
						),
						method.params[i].mut_span.clone(),
					));
				}
				if i == 0 && interface_param.name.val == "this" {
					i += 1;
					continue;
				}
				if interface_param.type_ != method.params[i].type_ {
					labels.push((
						format!(
							"expected {}{} parameter to be of type `{}`",
							i + 1,
							i_suffix_str,
							interface_param.type_
						),
						interface_param.type_span.clone(),
					));
					labels.push((
						format!(
							"instead got {}{} parameter of type `{}`",
							i + 1,
							i_suffix_str,
							method.params[i].type_
						),
						method.params[i].type_span.clone(),
					));
				}
			} else {
				labels.push((
					format!(
						"`{}` method `{}` is defined with more parameters than in the apply block",
						interface_name.val.to_string(),
						interface_method.name.val.to_string(),
					),
					interface_method.params_span.clone(),
				));
				labels.push((
					format!(
						"not enough parameters to implement `{}`",
						interface_method.name.val.to_string(),
					),
					method.params_span.clone(),
				));
			}
			i += 1;
		}

		if method_params_len > i {
			labels.push((
				"too many parameters in method definition".to_string(),
				method.params_span.clone(),
			));
		}

		if labels.len() > 1 {
			return Some(self.error(
				"method parameters in appy block do not match the interface method definition".to_owned(),
				PIErrorCode::TypecheckInterfaceMethodParamsDontMatch,
				labels,
			));
		}
		return None;
	}

	fn compare_interface_method_return_types(
		&self,
		interface_name: &Ident,
		interface_method: &Method,
		method: &FnDecl,
	) -> Option<PIError> {
		if interface_method.ret_ty != method.ret_ty {
			return Some(self.error(
				format!(
					"expected `{}` method `{}` to return `{}`, instead got `{}`",
					interface_name.val.to_string(),
					interface_method.name.val.to_string(),
					&interface_method.ret_ty,
					&method.ret_ty
				),
				PIErrorCode::TypecheckInterfaceMethodRetTyDontMatch,
				vec![
					(
						format!("defined with type `{}` in apply block", method.ret_ty),
						method.ret_ty_span.clone(),
					),
					(
						format!(
							"method `{}` defined here",
							interface_method.name.val.to_string()
						),
						interface_method.name.span.clone(),
					),
					(
						format!(
							"`{}` return type defined as `{}`",
							interface_method.name.val.to_string(),
							interface_method.ret_ty,
						),
						interface_method.ret_ty_span.clone(),
					),
				],
			));
		}
		return None;
	}

	fn check_fn(&mut self, f: &mut FnDecl) -> Option<PIError> {
		self.cur_block = Some(f.block.clone());
		for stmt in &mut f.block {
			self.expecting_ty = f.ret_ty.clone();
			if let Some(err) = self.check_stmt(stmt) {
				return Some(err);
			}
		}
		self.var_types.clear();
		return None;
	}

	fn check_stmt(&mut self, stmt: &mut Stmt) -> Option<PIError> {
		match stmt {
			Stmt::VarDecl(var) => self.check_var(var),
			Stmt::Return(ret) => self.check_ret(ret),
			Stmt::ExprStmt(expr) => self.check_expr(expr),
			_ => None,
		}
	}

	fn check_ret(&mut self, ret: &mut Return) -> Option<PIError> {
		if let Some(x) = &mut ret.val {
			if let Some(err) = self.check_expr(x) {
				return Some(err);
			}
		}
		return None;
	}

	fn check_var(&mut self, var: &mut VarDecl) -> Option<PIError> {
		self.expecting_ty = var.type_.clone();
		for name in &var.names {
			self
				.var_types
				.insert(name.val.to_string(), var.type_.clone());
		}

		for val in &mut var.values {
			if let Some(err) = self.check_expr(val) {
				return Some(err);
			}
		}

		return None;
	}

	fn check_expr(&mut self, expr: &mut Expr) -> Option<PIError> {
		match expr {
			Expr::IntLit(int) => self.check_int_lit(int),
			Expr::FloatLit(float) => self.check_float_lit(float.borrow_mut()),
			Expr::BinOp(binop) => self.check_binop(binop),
			Expr::CallExpr(call) => self.check_call(call),
			Expr::StructExpr(struct_expr) => self.check_struct_expr(struct_expr),
			_ => None,
		}
	}

	fn check_struct_expr(&self, struct_expr: &mut StructExpr) -> Option<PIError> {
		for (name, val) in &mut struct_expr.fields {
			if val.is_none() {
				*val = Some(Box::from(Expr::Ident(name.clone())));
			}
		}

		if let Some(struct_ty_decl) = self.types.get(&struct_expr.name.val.to_string()) {
			if let Expr::StructType(struct_ty) = &struct_ty_decl.type_ {
				if let Some(err) = self.compare_struct_expr_fields_to_struct_ty(
					struct_expr,
					struct_ty,
					&struct_ty_decl.name.val.to_string(),
				) {
					return Some(err);
				}
			}
		}

		return None;
	}

	fn compare_struct_expr_fields_to_struct_ty(
		&self,
		struct_expr: &mut StructExpr,
		struct_ty: &IndexMap<Ident, Field>,
		struct_ty_name: &String,
	) -> Option<PIError> {
		if struct_expr.fields.len() != struct_ty.len() {
			return Some(self.error(
				"struct expression does not have the same number of fields as the type it is constructing".to_owned(),
				PIErrorCode::TypecheckStructExprDiffNumberFieldsAsStructTy,
				vec![
					("incorrect number of fields in struct expression".to_owned(), struct_expr.fields_span.clone())
				],
			));
		}

		for (name, field) in struct_ty {
			if let Some(struct_expr_val_opt) = struct_expr.fields.get_mut(name) {
				let struct_expr_val = struct_expr_val_opt
					.as_mut()
					.expect("internal compiler error");
				let res = match &mut **struct_expr_val {
					Expr::IntLit(int) => match &field.type_ {
						Expr::PrimitiveType(prim) => match prim.kind {
							PrimitiveKind::I64 | PrimitiveKind::U64 => {
								int.bits = 64;
								None
							}
							PrimitiveKind::I32 | PrimitiveKind::U32 => {
								int.bits = 32;
								None
							}
							PrimitiveKind::I16 | PrimitiveKind::U16 => {
								int.bits = 16;
								None
							}
							PrimitiveKind::I8 | PrimitiveKind::U8 => {
								int.bits = 8;
								None
							}
							_ => Some(self.error(
								format!(
									"expected struct expression's field value to be of type `{:?}`",
									prim.kind
								),
								PIErrorCode::CodegenUnknownIdentType,
								vec![],
							)),
						},
						_ => Some(self.error(
							format!(
								"expected struct expression's field value to be of type `{}`",
								field.type_
							),
							PIErrorCode::CodegenUnknownIdentType,
							vec![],
						)),
					},
					Expr::FloatLit(float) => match &field.type_ {
						Expr::PrimitiveType(prim) => match prim.kind {
							PrimitiveKind::F64 => {
								float.bits = 64;
								None
							}
							PrimitiveKind::F32 => {
								float.bits = 32;
								None
							}
							_ => Some(self.error(
								format!(
									"expected struct expression's field value to be of type `{:?}`",
									prim.kind
								),
								PIErrorCode::CodegenUnknownIdentType,
								vec![],
							)),
						},
						_ => Some(self.error(
							format!(
								"expected struct expression's field value to be of type `{}`",
								field.type_
							),
							PIErrorCode::CodegenUnknownIdentType,
							vec![],
						)),
					},
					Expr::StructExpr(sub_struct_expr) => self.check_struct_expr(sub_struct_expr),
					_ => None,
				};
				if let Some(err) = res {
					return Some(err);
				}
			} else {
				return Some(self.error(
					format!(
						"could not find field `{}` in struct expression",
						name.val.to_string()
					),
					PIErrorCode::TypecheckCouldNotFindFieldInStructExpr,
					vec![
						(
							format!(
								"expected field `{}` in `{}` struct expression",
								name.val.to_string(),
								struct_ty_name,
							),
							name.span.clone(),
						),
						(
							"instead got these fields".to_owned(),
							struct_expr.fields_span.clone(),
						),
					],
				));
			}
		}

		return None;
	}

	fn check_call(&self, call: &mut CallExpr) -> Option<PIError> {
		// prepend pointer to `this` if it's a method call
		if let Expr::BinOp(binop) = &*call.callee {
			match &*binop.x {
				Expr::Ident(_) => {
					call.args.splice(
						..0,
						[Box::from(Expr::Unary(Unary::new(
							OpKind::Ampersand,
							binop.x.clone(),
						)))],
					);
				}
				_ => (),
			}
		}

		return None;
	}

	fn check_binop(&mut self, binop: &mut BinOp) -> Option<PIError> {
		if let Some(err) = self.check_expr(&mut *binop.x) {
			return Some(err);
		}
		if binop.op == OpKind::Eq {
			match &*binop.x {
				Expr::BinOp(b) => {
					let (expr, err) = self.get_struct_access_type(b);
					if let Some(err) = err {
						return Some(err);
					}
					self.expecting_ty = expr;
				}
				_ => (),
			}
		}
		if let Some(err) = self.check_expr(&mut *binop.y) {
			return Some(err);
		}
		return None;
	}

	fn get_struct_access_type(&mut self, binop: &BinOp) -> (Expr, Option<PIError>) {
		let mut b = binop;
		let mut field_names = vec![];
		if let Expr::Ident(rhs) = &*b.y {
			field_names.push(rhs);
		}
		while let Expr::BinOp(sub_binop) = &*b.x {
			if sub_binop.op != OpKind::Period {
				return (
					Expr::Error,
					Some(self.error(
						"expected `.` operator in chained struct field access".to_owned(),
						PIErrorCode::TypecheckExpectedPeriodOpInChainedStructFieldAccess,
						vec![],
					)),
				);
			}
			if let Expr::Ident(rhs) = &*sub_binop.y {
				field_names.push(rhs);
			} else {
				return (
					Expr::Error,
					Some(self.error(
						"expected rhs of `.` operator to be identifier".to_owned(),
						PIErrorCode::TypecheckExpectedRHSOfPeriodToBeIdent,
						vec![],
					)),
				);
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
				let (expr, err) = self.find_rightmost_field_type(&mut field_names, &struct_ty);
				if let Some(err) = err {
					return (Expr::Error, Some(err));
				} else {
					return (expr, None);
				}
			} else {
				return (
					Expr::Error,
					Some(self.error(
						"expected lhs of `.` operator to be a struct".to_owned(),
						PIErrorCode::TypecheckExpectedLHSOfPeriodToBeStruct,
						vec![],
					)),
				);
			}
		}
		panic!("this should be fatal");
	}

	fn find_rightmost_field_type(
		&self,
		field_names: &mut Vec<&Ident>,
		struct_ty: &StructType,
	) -> (Expr, Option<PIError>) {
		if field_names.len() == 0 {
			return (Expr::StructType(struct_ty.clone()), None);
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
					_ => (res, None),
				};
			} else {
				return (field_ty, None);
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

	fn get_type_of_var_in_cur_block(&self, name: &Ident) -> &Expr {
		self
			.var_types
			.get(&name.val.to_string())
			.expect("expected var with name")
	}

	fn check_float_lit(&mut self, float: &mut FloatLit) -> Option<PIError> {
		if let Expr::PrimitiveType(prim) = &self.expecting_ty {
			let expected_bits = TypecheckCtx::primitive_kind_to_bits(&prim.kind);
			if float.bits != expected_bits {
				float.bits = expected_bits;
			}
		}
		return None;
	}

	fn check_int_lit(&mut self, int: &mut IntLit) -> Option<PIError> {
		if let Expr::PrimitiveType(prim) = &self.expecting_ty {
			self.reassign_int_lit_bits(int, prim);
		} else if let Expr::Ident(ident) = &self.expecting_ty {
			let ty = self
				.types
				.get(&ident.val.to_string())
				.expect("expected type");
			if let Expr::PrimitiveType(prim) = &ty.type_ {
				self.reassign_int_lit_bits(int, prim);
			}
		}
		return None;
	}

	fn reassign_int_lit_bits(&self, int: &mut IntLit, prim: &PrimitiveType) -> Option<PIError> {
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
			return Some(self.error(
				format!("expected unsigned integer but got signed integer",),
				PIErrorCode::TypecheckUnexpectedSignednessInIntLit,
				labels,
			));
		}
		return None;
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

pub fn typecheck_ast(file_ast_map: &mut HashMap<FileId, AST>) -> Option<PIError> {
	let entry_fileid: FileId = FileId(0);
	let ast = file_ast_map
		.get_mut(&entry_fileid)
		.expect("could not get file");
	let mut ctx = TypecheckCtx {
		expecting_ty: Expr::PrimitiveType(PrimitiveType::new(PrimitiveKind::I32)),
		file_id: entry_fileid,
		types: HashMap::new(),
		var_types: HashMap::new(),
		cur_block: None,
	};

	return ctx.check(ast);
}
