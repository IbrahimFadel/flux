// use std::collections::HashMap;

// use pi_ast::{ApplyBlock, Expr, FnDecl, Ident, InterfaceType, Method, Spanned};
// use pi_error::{PIErrorCode, Span};

// use crate::{FnCtx, PIResult};

// impl<'ctx> FnCtx<'ctx> {
// 	pub fn check_apply(&mut self, apply: &'ctx mut ApplyBlock) -> PIResult {
// 		for f in &mut apply.methods {
// 			self.check_apply_method(f, apply.struct_name.to_string())?;
// 		}

// 		if apply.interface_name.is_none() {
// 			return Ok(());
// 		}

// 		let interface_name = apply.interface_name.as_ref().unwrap().clone();
// 		if let Some(ty_decl) = self.type_decls.get(&interface_name.to_string()) {
// 			if let Expr::InterfaceType(interface_ty) = &*ty_decl.type_ {
// 				self.compare_interface_methods_to_apply_block_methods(apply, interface_ty)?;
// 				self
// 					.struct_implementations
// 					.entry(apply.struct_name.to_string())
// 					.or_insert(Vec::new())
// 					.push((*ty_decl).clone());
// 			} else {
// 				return Err(self.error(
// 					format!(
// 						"expected `{}` to be an interface",
// 						interface_name.to_string()
// 					),
// 					PIErrorCode::TypecheckExpectedTypeToBeInterface,
// 					vec![],
// 				));
// 			}
// 		} else {
// 			return Err(self.error(
// 				format!(
// 					"could not find type with name `{}`",
// 					interface_name.to_string()
// 				),
// 				PIErrorCode::TypecheckCouldNotFindType,
// 				vec![],
// 			));
// 		}

// 		Ok(())
// 	}

// 	pub fn check_apply_method(&mut self, method: &'ctx mut FnDecl, struct_name: String) -> PIResult {
// 		let len = method.params.len();
// 		for param in &mut *method.params {
// 			if len > 0 {
// 				if param.name.as_str() == "this" {
// 					param.type_ = Spanned::new(
// 						Expr::PtrType(Box::from(Spanned::new(
// 							Expr::Ident(Ident::from(struct_name.clone())),
// 							Span::new(0..0, self.file_id),
// 						))),
// 						Span::new(0..0, self.file_id),
// 					);
// 				}
// 			}

// 			self.var_types.insert(param.name.to_string(), &param.type_);
// 		}
// 		for stmt in &mut method.block {
// 			self.expecting_ty = Some(&method.ret_ty);
// 			self.check_stmt(stmt)?;
// 		}

// 		// self
// 		// 	.struct_methods
// 		// 	.entry(struct_name.clone())
// 		// 	.or_insert(HashMap::new())
// 		// 	.insert(method.name.to_string(), &method);

// 		self.var_types.clear();
// 		Ok(())
// 	}

// 	fn compare_interface_methods_to_apply_block_methods(
// 		&self,
// 		apply_block: &'ctx ApplyBlock,
// 		interface_ty: &InterfaceType,
// 	) -> PIResult {
// 		let interface_name = apply_block
// 			.interface_name
// 			.as_ref()
// 			.expect("apply block should have interface type");
// 		if apply_block.methods.len() != interface_ty.len() {
// 			return Err(self.error(
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
// 				self.compare_interface_method_pubs(&interface_name, interface_method, method)?;
// 				self.compare_interface_method_return_types(&interface_name, interface_method, method)?;
// 				self.compare_interface_method_param_types(&interface_name, interface_method, method)?;
// 			} else {
// 				return Err(self.error(
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

// 		Ok(())
// 	}

// 	fn compare_interface_method_pubs(
// 		&self,
// 		interface_name: &Spanned<Ident>,
// 		interface_method: &Spanned<Method>,
// 		method: &Spanned<FnDecl>,
// 	) -> PIResult {
// 		let visibility1 = match *interface_method.pub_ {
// 			true => "public",
// 			false => "private",
// 		};
// 		let visibility2 = match *method.pub_ {
// 			true => "public",
// 			false => "private",
// 		};
// 		if visibility1 != visibility2 {
// 			return Err(self.error(
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
// 		Ok(())
// 	}

// 	fn compare_interface_method_param_types(
// 		&self,
// 		interface_name: &Spanned<Ident>,
// 		interface_method: &Spanned<Method>,
// 		method: &Spanned<FnDecl>,
// 	) -> PIResult {
// 		let mut labels: Vec<(String, Span)> = vec![(
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
// 			return Err(self.error(
// 				"method parameters in appy block do not match the interface method definition".to_owned(),
// 				PIErrorCode::TypecheckInterfaceMethodParamsDontMatch,
// 				labels,
// 			));
// 		}
// 		Ok(())
// 	}

// 	fn compare_interface_method_return_types(
// 		&self,
// 		interface_name: &Ident,
// 		interface_method: &Method,
// 		method: &FnDecl,
// 	) -> PIResult {
// 		if interface_method.ret_ty != method.ret_ty {
// 			return Err(self.error(
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
// 		Ok(())
// 	}
// }
