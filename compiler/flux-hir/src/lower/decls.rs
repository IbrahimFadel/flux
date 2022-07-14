use super::*;

use itertools::Itertools;
use std::collections::HashSet;

impl<'a> LoweringCtx<'a> {
	pub(crate) fn lower_trait_decl(
		&mut self,
		trait_decl: ast::TraitDecl,
	) -> Result<TraitDecl, FluxError> {
		let name = self.unwrap_ident(
			trait_decl.name(),
			trait_decl.range(),
			format!("trait declaration missing name"),
		)?;
		let name = Spanned::new(
			name,
			Span::new(
				trait_decl.name().unwrap().text_range(),
				self.file_id.clone(),
			),
		);

		let mut methods = HashMap::new();
		for method in trait_decl.methods() {
			let name: Spanned<SmolStr> = if let Some(name) = method.name() {
				Spanned::new(
					name.text().into(),
					Span::new(name.text_range(), self.file_id.clone()),
				)
			} else {
				return Err(FluxError::build(
					format!("trait method missing name"),
					Span::new(method.range(), self.file_id.clone()),
					FluxErrorCode::TraitMethodMissingName,
					(
						format!("trait method missing name"),
						Span::new(method.range(), self.file_id.clone()),
					),
				));
			};
			let params = self.lower_params(method.params())?;
			let params_range = match (method.lparen(), method.rparen()) {
				(Some(lparen), Some(rparen)) => {
					TextRange::new(lparen.text_range().start(), rparen.text_range().end())
				}
				(Some(lparen), _) => {
					if !params.is_empty() {
						TextRange::new(
							lparen.text_range().start(),
							params.last().unwrap().span.range.end(),
						)
					} else {
						TextRange::new(lparen.text_range().start(), lparen.text_range().end())
					}
				}
				(_, Some(rparen)) => {
					if !params.is_empty() {
						TextRange::new(params[0].span.range.end(), rparen.text_range().end())
					} else {
						TextRange::new(rparen.text_range().start(), rparen.text_range().end())
					}
				}
				_ => method.range(),
			};
			let params = Spanned::new(params, Span::new(params_range, self.file_id.clone()));

			let return_type = self.lower_type(method.return_ty())?;
			let method = TraitMethod {
				name: name.clone(),
				params,
				return_type,
			};
			methods.insert(name.node, method);
		}
		Ok(TraitDecl { name, methods })
	}

	pub(crate) fn lower_apply_decl(
		&mut self,
		apply_decl: ast::ApplyDecl,
	) -> Result<ApplyDecl, FluxError> {
		let (trait_, struct_): (Option<Spanned<SmolStr>>, SmolStr) =
			match (apply_decl.trait_(), apply_decl.struct_()) {
				(None, None) => {
					return Err(FluxError::build(
						format!("missing struct to `apply` methods to"),
						self.span(&apply_decl),
						FluxErrorCode::MissingStructToApplyMethodsTo,
						(
							format!("missing struct to `apply` methods to"),
							self.span(&apply_decl),
						),
					))
				}
				(Some(struct_), None) => (None, struct_.text().into()),
				(Some(trait_), Some(struct_)) => (
					Some(Spanned::new(
						trait_.text().into(),
						Span::new(trait_.text_range(), self.file_id.clone()),
					)),
					struct_.text().into(),
				),
				_ => unreachable!(),
			};

		if let Some(block) = &apply_decl.block() {
			if let Some(trait_) = &trait_ {
				let trait_decl = match self.traits.get(&trait_.node) {
					Some(decl) => decl,
					None => {
						return Err(
							FluxError::build(
								format!(
									"cannot apply trait `{}` to `{}`: the trait does not exist",
									trait_.node, struct_
								),
								self.span(block),
								FluxErrorCode::AppliedUnknownTrait,
								(
									format!("unknown trait `{}`", trait_.node),
									trait_.span.clone(),
								),
							)
							.with_note(format!(
								"try declaring a trait with `trait {} {{}}`",
								trait_.node
							))
							.with_note(if self.traits.len() > 0 {
								format!(
									"the following traits are available in this module: {}",
									self.traits.keys().join(", ")
								)
							} else {
								format!("no traits are available in this module")
							}),
						);
					}
				};

				self.lower_and_validate_apply_block(block, trait_decl, &struct_)?;
			}
		} else {
			return Err(FluxError::build(
				format!("missing apply block"),
				self.span(&apply_decl),
				FluxErrorCode::NoCode,
				(format!("missing apply block"), self.span(&apply_decl)),
			));
		}

		Ok(ApplyDecl {
			trait_,
			struct_,
			methods: vec![],
		})
	}

	fn lower_and_validate_apply_block(
		&mut self,
		apply_block: &ast::ApplyBlock,
		trait_decl: &TraitDecl,
		struct_: &str,
	) -> Result<(), FluxError> {
		let mut methods_implemented = HashSet::new();
		for method in apply_block.methods() {
			let method = self.lower_fn_decl(method)?;
			if let Some(method_decl) = trait_decl.methods.get(&method.name.node) {
				self.validate_trait_method_implementation(method_decl, &method)?;
				methods_implemented.insert(method_decl.name.node.as_str());
			} else {
				return Err(
					FluxError::build(
						format!(
							"trait `{}` does not have method `{}`",
							trait_decl.name.node, method.name.node
						),
						method.name.span.clone(),
						FluxErrorCode::UnknownTraitMethod,
						(
							format!(
								"no such method `{}` on trait `{}`",
								method.name.node, trait_decl.name.node
							),
							method.name.span.clone(),
						),
					)
					.with_label(
						format!("`{}` defined here", trait_decl.name.node),
						trait_decl.name.span.clone(),
					)
					.with_note(if trait_decl.methods.len() > 0 {
						format!(
							"the trait `{}` has the following methods: {}",
							trait_decl.name.node,
							trait_decl.methods.keys().join(", ")
						)
					} else {
						format!("the trait `{}` has no methods", trait_decl.name.node)
					}),
				);
			}
		}

		let unimplemented_methods: Vec<_> = trait_decl
			.methods
			.iter()
			.filter_map(
				|(method, _)| match methods_implemented.get(method.as_str()) {
					Some(_) => None,
					None => Some(method),
				},
			)
			.collect();

		if unimplemented_methods.len() > 0 {
			return Err(FluxError::build(
				format!("unimplemented trait methods"),
				Span::new(apply_block.range(), self.file_id.clone()),
				FluxErrorCode::UnimplementedTraitMethods,
				(
					format!(
						"missing implementation for the trait methods {} on struct `{}`",
						unimplemented_methods
							.iter()
							.map(|m| format!("`{}`", m.as_str()))
							.join(", "),
						struct_
					),
					Span::new(apply_block.range(), self.file_id.clone()),
				),
			));
		}

		Ok(())
	}

	fn validate_trait_method_implementation(
		&mut self,
		method_decl: &TraitMethod,
		method_impl: &FnDecl,
	) -> Result<(), FluxError> {
		let return_ty_id = self.tchecker.tenv.insert(method_decl.return_type.clone());
		let return_ty_impl_id = self.tchecker.tenv.insert(method_impl.return_type.clone());
		self.tchecker.unify(
			return_ty_id,
			return_ty_impl_id,
			method_impl.return_type.span.clone(),
		)?;

		let method_decl_params = method_decl.params.len();
		let method_impl_params = method_impl.params.len();
		if method_decl_params != method_impl_params {
			return Err(
				FluxError::build(
					format!("incorrect number of arguments supplied to trait method definition"),
					method_impl.params.span.clone(),
					FluxErrorCode::IncorrectNumberParamsInMethodImpl,
					(
						format!("incorrect number of arguments supplied to trait method definition"),
						method_impl.params.span.clone(),
					),
				)
				.with_label(
					format!(
						"the method `{}` is defined with {} parameters",
						method_decl.name.node, method_decl_params
					),
					method_decl.params.span.clone(),
				),
			);
		}

		for (i, decl_param) in method_decl.params.iter().enumerate() {
			let impl_param = &method_impl.params[i];
			let decl_id = self.tchecker.tenv.insert(decl_param.ty.clone());
			let impl_id = self.tchecker.tenv.insert(impl_param.ty.clone());
			self
				.tchecker
				.unify(decl_id, impl_id, impl_param.ty.span.clone())?;
		}

		Ok(())
	}

	pub(crate) fn lower_type_decl(&mut self, ty_decl: ast::TypeDecl) -> Result<TypeDecl, FluxError> {
		let visibility = if let Some(public) = ty_decl.public() {
			Spanned::new(
				Visibility::Public,
				Span::new(public.text_range(), self.file_id.clone()),
			)
		} else {
			Spanned::new(
				Visibility::Private,
				Span::new(
					ty_decl.first_token().unwrap().text_range(),
					self.file_id.clone(),
				),
			)
		};
		let name = if let Some(name) = ty_decl.name() {
			Spanned::new(
				name.text().into(),
				Span::new(name.text_range(), self.file_id.clone()),
			)
		} else {
			return Err(FluxError::build(
				format!("missing name in type declaration"),
				self.span(&ty_decl),
				FluxErrorCode::MissingNameTyDecl,
				(
					format!("missing name in type declaration"),
					self.span(&ty_decl),
				),
			));
		};
		let ty = self.lower_type(ty_decl.ty())?;
		Ok(TypeDecl {
			visibility,
			name,
			ty,
		})
	}

	pub(crate) fn lower_fn_decl(&mut self, fn_decl: ast::FnDecl) -> Result<FnDecl, FluxError> {
		self.tchecker.tenv = TypeEnv::new();

		let visibility = if let Some(p) = fn_decl.public() {
			Spanned::new(
				Visibility::Public,
				Span::new(p.text_range(), self.file_id.clone()),
			)
		} else {
			Spanned::new(
				Visibility::Private,
				Span::new(
					fn_decl.first_token().unwrap().text_range(),
					self.file_id.clone(),
				),
			)
		};

		let params = self.lower_params(fn_decl.params())?;
		let params_range = match (fn_decl.lparen(), fn_decl.rparen()) {
			(Some(lparen), Some(rparen)) => {
				TextRange::new(lparen.text_range().start(), rparen.text_range().end())
			}
			(Some(lparen), _) => {
				if !params.is_empty() {
					TextRange::new(
						lparen.text_range().start(),
						params.last().unwrap().span.range.end(),
					)
				} else {
					TextRange::new(lparen.text_range().start(), lparen.text_range().end())
				}
			}
			(_, Some(rparen)) => {
				if !params.is_empty() {
					TextRange::new(params[0].span.range.end(), rparen.text_range().end())
				} else {
					TextRange::new(rparen.text_range().start(), rparen.text_range().end())
				}
			}
			_ => fn_decl.range(),
		};
		let params = Spanned::new(params, Span::new(params_range, self.file_id.clone()));

		let (body, body_id) = self.lower_expr(fn_decl.body())?;

		let return_id = if let Some(return_type) = fn_decl.return_type() {
			let ty = self.lower_type(Some(return_type))?;
			self.tchecker.tenv.insert(ty)
		} else {
			self.tchecker.tenv.insert(Spanned::new(
				Type::Tuple(vec![]),
				Span::new(
					TextRange::new(params_range.end(), params_range.end()),
					self.file_id.clone(),
				),
			))
		};
		self.tchecker.tenv.return_type_id = return_id;

		let ret_ty_unification_span = if let Expr::Block(block) = &self.exprs[body].node {
			if block.0.len() > 0 {
				block.0.last().unwrap().span.clone()
			} else {
				self.exprs[body].span.clone()
			}
		} else {
			self.exprs[body].span.clone()
		};
		self
			.tchecker
			.unify(body_id, return_id, ret_ty_unification_span)?;
		let return_type: Spanned<Type> = self.tchecker.tenv.get_type(return_id).into();

		let name = if let Some(name) = fn_decl.name() {
			Spanned::new(
				SmolStr::from(name.text()),
				Span::new(name.text_range(), self.file_id.clone()),
			)
		} else {
			return Err(FluxError::build(
				format!("could not lower function declaration: missing name"),
				self.span(&fn_decl),
				FluxErrorCode::CouldNotLowerNode,
				(
					format!("could not lower function declaration: missing name"),
					self.span(&fn_decl),
				),
			));
		};

		if let Expr::Block(block) = &mut self.exprs[body].node {
			for stmt in &mut block.0 {
				if let Stmt::VarDecl(var) = &mut stmt.node {
					let id = self.tchecker.tenv.get_path_id(&[var.name.clone()]);
					var.ty = self.tchecker.tenv.reconstruct(id)?.into();
				}
			}
		}

		Ok(FnDecl {
			visibility,
			name,
			params,
			body,
			return_type,
		})
	}

	pub(crate) fn lower_params(
		&mut self,
		params: impl Iterator<Item = ast::FnParam>,
	) -> Result<Vec<Spanned<FnParam>>, FluxError> {
		let mut hir_params = vec![];
		for param in params {
			let name = if let Some(name) = param.name() {
				Some(name.text().into())
			} else {
				None
			};
			let ty = self.lower_type(param.ty())?;
			hir_params.push(Spanned::new(
				FnParam {
					mutable: param.mutable().is_some(),
					ty,
					name,
				},
				Span::new(param.range(), self.file_id.clone()),
			));
		}
		Ok(hir_params)
	}
}
