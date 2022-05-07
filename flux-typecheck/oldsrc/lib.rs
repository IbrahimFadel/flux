// use std::{collections::HashMap, ops::Range};

// use indexmap::IndexMap;
// use flux_ast::{
// 	ApplyBlock, BlockStmt, Expr, FnDecl, Ident, IntLit, InterfaceType, Method, PrimitiveKind,
// 	PrimitiveType, Spanned, Stmt, StructType, TypeDecl, AST,
// };
// use flux_error::{filesystem::FileId, PIError, PIErrorCode, PIErrorReporting};

// mod expr;
// mod stmt;

// struct TypecheckCtx<'a> {
// 	ast_map: &'a IndexMap<FileId, AST>,
// 	err_reporting: &'a PIErrorReporting,
// 	expecting_ty: Option<&'a Expr>,
// 	file_id: FileId,
// 	types: HashMap<String, &'a TypeDecl>,
// 	var_types: HashMap<String, &'a Expr>,
// 	cur_block: Option<BlockStmt>,
// 	struct_methods: HashMap<String, HashMap<String, &'a FnDecl>>,
// 	struct_implementations: HashMap<String, Vec<TypeDecl>>,
// }

// impl<'a> TypecheckCtx<'a> {
// 	pub fn new(ast_map: &'a IndexMap<FileId, AST>, err_reporting: &'a PIErrorReporting) -> Self {
// 		Self {
// 			ast_map,
// 			err_reporting,
// 			file_id: FileId(0),
// 			expecting_ty: None,
// 			types: HashMap::new(),
// 			var_types: HashMap::new(),
// 			cur_block: None,
// 			struct_methods: HashMap::new(),
// 			struct_implementations: HashMap::new(),
// 		}
// 	}

// 	fn error(&self, msg: String, code: PIErrorCode, labels: Vec<(String, Range<usize>)>) -> PIError {
// 		PIError::new(msg, code, labels, self.file_id)
// 	}

// 	pub fn check(&mut self, ast_map: &'a mut IndexMap<FileId, AST>) -> Option<PIError> {
// 		self.edit_interface_this_types(ast_map);

// 		for (id, ast) in ast_map.iter_mut() {
// 			self.file_id = id.clone();
// 			for ty in &ast.types {
// 				self.types.insert(ty.name.to_string(), &ty);
// 			}

// 			for apply_block in &mut ast.apply_blocks {
// 				if let Some(err) = self.check_apply(apply_block) {
// 					return Some(err);
// 				}
// 			}

// 			for f in &mut ast.functions {
// 				if let Some(err) = self.check_fn(f) {
// 					return Some(err);
// 				}
// 			}
// 		}
// 		return None;
// 	}

// 	fn edit_interface_this_types(&mut self, ast_map: &mut IndexMap<FileId, AST>) {
// 		let types = &mut ast_map.get_mut(&self.file_id).unwrap().types;
// 		for ty in types {
// 			let ty_clone = ty.clone();
// 			if let Expr::InterfaceType(interface_ty) = &mut *ty.type_ {
// 				for (_, method) in interface_ty {
// 					if method.params.len() > 0 {
// 						if *method.params[0].name == "this" {
// 							method.params[0].type_.node = Expr::PtrType(Box::from(Spanned::new(
// 								Expr::Ident((*(ty_clone.name)).clone()),
// 								ty_clone.span.clone(),
// 							)));
// 						}
// 					}
// 				}
// 			}
// 		}
// 	}

// 	fn check_apply(&mut self, apply_block: &'a mut ApplyBlock) -> Option<PIError> {
// 		// for f in &mut apply_block.methods {
// 		// 	if let Some(res) = self.check_fn(f) {
// 		// 		return Some(res);
// 		// 	}

// 		// 	// do inside self.check_fn
// 		// 	// if f.params.len() > 0 {
// 		// 	// 	if *f.params[0].name == "this" {
// 		// 	// 		f.params[0].type_ = Spanned::new(
// 		// 	// 			Expr::PtrType(Box::from(Spanned::new(
// 		// 	// 				Expr::Ident(Ident::from(*apply_block.struct_name.clone())),
// 		// 	// 				0..0,
// 		// 	// 			))),
// 		// 	// 			0..0,
// 		// 	// 		);
// 		// 	// 	}
// 		// 	// }
// 		// 	// self
// 		// 	// 	.struct_methods
// 		// 	// 	.entry(apply_block.struct_name.to_string())
// 		// 	// 	.or_insert(HashMap::new())
// 		// 	// 	.insert(f.name.to_string(), &**f);
// 		// }
// 		// if apply_block.interface_name.is_none() {
// 		// 	return None;
// 		// }

// 		let interface_name = apply_block.interface_name.as_ref().unwrap().clone();
// 		if let Some(ty_decl) = self.types.get(&interface_name.to_string()) {
// 			if let Expr::InterfaceType(interface_ty) = &*ty_decl.type_ {
// 				if let Some(err) =
// 					self.compare_interface_methods_to_apply_block_methods(apply_block, interface_ty)
// 				{
// 					return Some(err);
// 				} else {
// 					self
// 						.struct_implementations
// 						.entry(apply_block.struct_name.to_string())
// 						.or_insert(Vec::new())
// 						.push((*ty_decl).clone());
// 				}
// 			} else {
// 				return Some(self.error(
// 					format!(
// 						"expected `{}` to be an interface",
// 						interface_name.to_string()
// 					),
// 					PIErrorCode::TypecheckExpectedTypeToBeInterface,
// 					vec![],
// 				));
// 			}
// 		} else {
// 			return Some(self.error(
// 				format!(
// 					"could not find type with name `{}`",
// 					interface_name.to_string()
// 				),
// 				PIErrorCode::TypecheckCouldNotFindType,
// 				vec![],
// 			));
// 		}

// 		return None;
// 	}

// 	fn compare_interface_methods_to_apply_block_methods(
// 		// &self,
// 		// interface_name: &Spanned<Ident>, // only needed for error reporting
// 		// interface_ty: &InterfaceType,
// 		// methods: &Vec<Spanned<FnDecl>>,
// 		&self,
// 		apply_block: &'a ApplyBlock,
// 		interface_ty: &InterfaceType,
// 	) -> Option<PIError> {
// 		let interface_name = apply_block
// 			.interface_name
// 			.as_ref()
// 			.expect("apply block should have interface type");
// 		if apply_block.methods.len() != interface_ty.len() {
// 			return Some(self.error(
// 				format!(
// 					"not all methods of `{}` were implemented in apply block",
// 					interface_name.to_string(),
// 				),
// 				PIErrorCode::TypecheckNotAllInterfaceMethodsImplemented,
// 				vec![],
// 			));
// 		}
// 		for method in &apply_block.methods {
// 			if let Some(interface_method) = interface_ty.get(&method.name) {
// 				if let Some(err) =
// 					self.compare_interface_method_pubs(&interface_name, interface_method, method)
// 				{
// 					return Some(err);
// 				} else if let Some(err) =
// 					self.compare_interface_method_return_types(&interface_name, interface_method, method)
// 				{
// 					return Some(err);
// 				} else if let Some(err) =
// 					self.compare_interface_method_param_types(&interface_name, interface_method, method)
// 				{
// 					return Some(err);
// 				}
// 			} else {
// 				return Some(self.error(
// 					format!(
// 						"method `{}` could not be found in interface `{}`",
// 						method.name.to_string(),
// 						interface_name.to_string(),
// 					),
// 					PIErrorCode::TypecheckCouldNotFindMethodInInterface,
// 					vec![],
// 				));
// 			}
// 		}

// 		return None;
// 	}

// 	fn compare_interface_method_pubs(
// 		&self,
// 		interface_name: &Spanned<Ident>,
// 		interface_method: &Spanned<Method>,
// 		method: &Spanned<FnDecl>,
// 	) -> Option<PIError> {
// 		let visibility1 = match *interface_method.pub_ {
// 			true => "public",
// 			false => "private",
// 		};
// 		let visibility2 = match *method.pub_ {
// 			true => "public",
// 			false => "private",
// 		};
// 		if visibility1 != visibility2 {
// 			return Some(self.error(
// 				format!("interface method visibilities do not match"),
// 				PIErrorCode::TypecheckInterfaceMethodVisibilitiesDontMatch,
// 				vec![
// 					(
// 						format!(
// 							"`{}` method `{}` defined as {}",
// 							interface_name.to_string(),
// 							interface_method.name.to_string(),
// 							visibility1
// 						),
// 						interface_method.span.clone(),
// 					),
// 					(
// 						format!("but defined as {} in apply block", visibility2),
// 						method.pub_.span.clone(),
// 					),
// 				],
// 			));
// 		}
// 		return None;
// 	}

// 	fn compare_interface_method_param_types(
// 		&self,
// 		interface_name: &Spanned<Ident>,
// 		interface_method: &Spanned<Method>,
// 		method: &Spanned<FnDecl>,
// 	) -> Option<PIError> {
// 		let mut labels: Vec<(String, Range<usize>)> = vec![(
// 			format!("method parameters in appy block do not match the interface method definition"),
// 			interface_method.params.span.clone(),
// 		)];
// 		let method_params_len = method.params.len();
// 		let mut i = 0;
// 		for interface_param in &*interface_method.params {
// 			if method_params_len >= i + 1 {
// 				let i_suffix_str = match i + 1 {
// 					1 => "st",
// 					2 => "nd",
// 					3 => "rd",
// 					_ => "th",
// 				};
// 				let mutability1 = match *interface_param.mut_ {
// 					true => "mutable",
// 					false => "immutable",
// 				};
// 				let mutability2 = match *method.params[i].mut_ {
// 					true => "mutable",
// 					false => "immutable",
// 				};
// 				if mutability1 != mutability2 {
// 					labels.push((
// 						format!(
// 							"expected {}{} parameter to be {}",
// 							i + 1,
// 							i_suffix_str,
// 							mutability1
// 						),
// 						interface_param.mut_.span.clone(),
// 					));
// 					labels.push((
// 						format!(
// 							"instead got {} {}{} parameter",
// 							mutability2,
// 							i + 1,
// 							i_suffix_str,
// 						),
// 						method.params[i].mut_.span.clone(),
// 					));
// 				}
// 				if i == 0 && *interface_param.name == "this" {
// 					i += 1;
// 					continue;
// 				}
// 				if interface_param.type_ != method.params[i].type_ {
// 					labels.push((
// 						format!(
// 							"expected {}{} parameter to be of type `{}`",
// 							i + 1,
// 							i_suffix_str,
// 							*interface_param.type_
// 						),
// 						interface_param.type_.span.clone(),
// 					));
// 					labels.push((
// 						format!(
// 							"instead got {}{} parameter of type `{}`",
// 							i + 1,
// 							i_suffix_str,
// 							*method.params[i].type_
// 						),
// 						method.params[i].type_.span.clone(),
// 					));
// 				}
// 			} else {
// 				labels.push((
// 					format!(
// 						"`{}` method `{}` is defined with more parameters than in the apply block",
// 						interface_name.to_string(),
// 						interface_method.name.to_string(),
// 					),
// 					interface_method.params.span.clone(),
// 				));
// 				labels.push((
// 					format!(
// 						"not enough parameters to implement `{}`",
// 						interface_method.name.to_string(),
// 					),
// 					method.params.span.clone(),
// 				));
// 			}
// 			i += 1;
// 		}

// 		if method_params_len > i {
// 			labels.push((
// 				"too many parameters in method definition".to_string(),
// 				method.params.span.clone(),
// 			));
// 		}

// 		if labels.len() > 1 {
// 			return Some(self.error(
// 				"method parameters in appy block do not match the interface method definition".to_owned(),
// 				PIErrorCode::TypecheckInterfaceMethodParamsDontMatch,
// 				labels,
// 			));
// 		}
// 		return None;
// 	}

// 	fn compare_interface_method_return_types(
// 		&self,
// 		interface_name: &Ident,
// 		interface_method: &Method,
// 		method: &FnDecl,
// 	) -> Option<PIError> {
// 		if interface_method.ret_ty != method.ret_ty {
// 			return Some(self.error(
// 				format!(
// 					"expected `{}` method `{}` to return `{}`, instead got `{}`",
// 					interface_name.to_string(),
// 					interface_method.name.to_string(),
// 					&*interface_method.ret_ty,
// 					&*method.ret_ty
// 				),
// 				PIErrorCode::TypecheckInterfaceMethodRetTyDontMatch,
// 				vec![
// 					(
// 						format!("defined with type `{}` in apply block", *method.ret_ty),
// 						method.ret_ty.span.clone(),
// 					),
// 					(
// 						format!(
// 							"method `{}` defined here",
// 							interface_method.name.to_string()
// 						),
// 						interface_method.name.span.clone(),
// 					),
// 					(
// 						format!(
// 							"`{}` return type defined as `{}`",
// 							interface_method.name.to_string(),
// 							*interface_method.ret_ty,
// 						),
// 						interface_method.ret_ty.span.clone(),
// 					),
// 				],
// 			));
// 		}
// 		return None;
// 	}

// 	fn check_fn(&mut self, f: &'a mut FnDecl) -> Option<PIError> {
// 		self.cur_block = Some(f.block.clone());
// 		for param in &*f.params {
// 			self.var_types.insert(param.name.to_string(), &param.type_);
// 		}
// 		for stmt in &mut f.block {
// 			self.expecting_ty = Some(&f.ret_ty);
// 			if let Some(err) = self.check_stmt(stmt) {
// 				return Some(err);
// 			}
// 		}
// 		self.var_types.clear();
// 		return None;
// 	}

// 	fn get_struct_field_type(
// 		&self,
// 		struct_ty: &'a StructType,
// 		field_name: &Spanned<Ident>,
// 	) -> Option<&'a Expr> {
// 		if let Some(field) = struct_ty.get(field_name) {
// 			return Some(&field.type_);
// 		}
// 		None
// 	}

// 	fn get_type_of_var_in_cur_block(&self, name: &Spanned<Ident>) -> (&'a Expr, Option<PIError>) {
// 		let res = self.var_types.get(&name.to_string());
// 		if let Some(ty) = res {
// 			return (ty, None);
// 		}
// 		(
// 			&Expr::Error,
// 			Some(self.error(
// 				format!("could not get type of variable `{}`", name.to_string()),
// 				PIErrorCode::TypecheckCouldNotGetTypeOfVar,
// 				vec![(
// 					format!("could not get type of variable `{}`", name.to_string()),
// 					name.span.clone(),
// 				)],
// 			)),
// 		)
// 	}
// }

// fn primitive_kind_to_bits(prim: &PrimitiveKind) -> u8 {
// 	match prim {
// 		PrimitiveKind::U64 | PrimitiveKind::I64 | PrimitiveKind::F64 => 64,
// 		PrimitiveKind::U32 | PrimitiveKind::I32 | PrimitiveKind::F32 => 32,
// 		PrimitiveKind::U16 | PrimitiveKind::I16 => 16,
// 		PrimitiveKind::U8 | PrimitiveKind::I8 => 8,
// 		_ => 32,
// 	}
// }

// fn primitive_kind_to_signedness(prim: &PrimitiveKind) -> bool {
// 	match prim {
// 		PrimitiveKind::U64 | PrimitiveKind::U32 | PrimitiveKind::U16 | PrimitiveKind::U8 => false,
// 		PrimitiveKind::I64
// 		| PrimitiveKind::I32
// 		| PrimitiveKind::I16
// 		| PrimitiveKind::I8
// 		| PrimitiveKind::F64
// 		| PrimitiveKind::F32 => true,
// 		_ => true,
// 	}
// }

// pub fn typecheck_ast<'a>(
// 	file_ast_map: &mut IndexMap<FileId, AST>,
// 	err_reporting: &'a PIErrorReporting,
// ) -> Option<PIError> {
// 	let new_ast_map = file_ast_map.clone();
// 	let mut ctx = TypecheckCtx::new(&new_ast_map, err_reporting);
// 	return ctx.check(file_ast_map);
// }

// use std::collections::HashMap;

// use indexmap::IndexMap;
// use flux_ast::{Expr, PrimitiveKind, PrimitiveType, Spanned, Stmt, AST};
// use flux_error::{filesystem::FileId, PIErrorReporting};

// pub type TypeId = usize;

// #[derive(Debug, Clone)]
// enum TypeInfo {
// 	Unknown,
// 	Ref(TypeId),
// 	Int,
// 	I64,
// 	I32,
// 	I16,
// 	I8,
// 	Float,
// 	F64,
// 	F32,
// }

// #[derive(Debug)]
// struct Engine {
// 	id_counter: usize,
// 	vars: HashMap<TypeId, TypeInfo>,
// }

// impl Engine {
// 	pub fn new() -> Self {
// 		Self {
// 			id_counter: 0,
// 			vars: HashMap::new(),
// 		}
// 	}

// 	pub fn insert(&mut self, info: TypeInfo) -> TypeId {
// 		self.id_counter += 1;
// 		let id = self.id_counter;
// 		self.vars.insert(id, info);
// 		return id;
// 	}

// 	pub fn unify(&mut self, a: TypeId, b: TypeId) -> Result<(), String> {
// 		use TypeInfo::*;
// 		match (self.vars[&a].clone(), self.vars[&b].clone()) {
// 			(Ref(a), _) => self.unify(a, b),
// 			(_, Ref(b)) => self.unify(a, b),
// 			(Unknown, _) => {
// 				self.vars.insert(a, TypeInfo::Ref(b));
// 				Ok(())
// 			}
// 			(_, Unknown) => {
// 				self.vars.insert(b, TypeInfo::Ref(a));
// 				Ok(())
// 			}
// 			(F64, Float) | (F32, Float) | (I64, Int) | (I32, Int) | (I16, Int) | (I8, Int) => {
// 				self.vars.insert(b, Ref(a));
// 				Ok(())
// 			}
// 			(a, b) => Err(format!("Conflict between {:?} and {:?}", a, b)),
// 		}
// 	}

// 	pub fn reconstruct(&self, id: TypeId) -> Result<Expr, String> {
// 		use TypeInfo::*;
// 		match self.vars[&id] {
// 			Unknown => Err(format!("cannot infer type")),
// 			Ref(id) => self.reconstruct(id),
// 			Int => Ok(Expr::PrimitiveType(PrimitiveType::new(PrimitiveKind::I32))),
// 			I64 => Ok(Expr::PrimitiveType(PrimitiveType::new(PrimitiveKind::I64))),
// 			I32 => Ok(Expr::PrimitiveType(PrimitiveType::new(PrimitiveKind::I32))),
// 			I16 => Ok(Expr::PrimitiveType(PrimitiveType::new(PrimitiveKind::I16))),
// 			I8 => Ok(Expr::PrimitiveType(PrimitiveType::new(PrimitiveKind::I8))),
// 			Float => Ok(Expr::PrimitiveType(PrimitiveType::new(PrimitiveKind::F32))),
// 			F64 => Ok(Expr::PrimitiveType(PrimitiveType::new(PrimitiveKind::F64))),
// 			F32 => Ok(Expr::PrimitiveType(PrimitiveType::new(PrimitiveKind::F32))),
// 		}
// 	}

// 	pub fn expr_to_type_info(ty: &Spanned<Expr>) -> TypeInfo {
// 		match &**ty {
// 			// Types
// 			Expr::PrimitiveType(prim) => match prim.kind {
// 				PrimitiveKind::I64 => TypeInfo::I64,
// 				PrimitiveKind::I32 => TypeInfo::I32,
// 				PrimitiveKind::I16 => TypeInfo::I16,
// 				PrimitiveKind::I8 => TypeInfo::I8,
// 				PrimitiveKind::F64 => TypeInfo::F64,
// 				PrimitiveKind::F32 => TypeInfo::F32,
// 				_ => TypeInfo::Unknown,
// 			},
// 			// Values
// 			Expr::IntLit(_) => TypeInfo::Int,
// 			Expr::FloatLit(_) => TypeInfo::Float,
// 			_ => TypeInfo::Unknown,
// 		}
// 	}
// }

// pub fn typecheck_ast<'a>(
// 	file_ast_map: &mut IndexMap<FileId, AST>,
// 	err_reporting: &'a PIErrorReporting,
// ) {
// 	let mut engine = Engine::new();

// 	let id = FileId(0);
// 	let ast = &file_ast_map[&id];

// 	for f in &ast.functions {
// 		for stmt in &f.block {
// 			match &**stmt {
// 				Stmt::VarDecl(var) => {
// 					let var_ty = Engine::expr_to_type_info(&var.type_);
// 					let var_tid = engine.insert(var_ty.clone());

// 					for v in &var.values {
// 						let v_tid = engine.insert(Engine::expr_to_type_info(v));
// 						let res = engine.unify(var_tid, v_tid);
// 						if res.is_err() {
// 							println!("{:?}", res.err());
// 						}
// 						let res = engine.reconstruct(v_tid);
// 					}
// 				}
// 				_ => (),
// 			}
// 		}
// 	}

// 	println!("{:?}", engine);
// }
