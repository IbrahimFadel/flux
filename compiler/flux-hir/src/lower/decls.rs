use flux_typesystem::r#type::ConcreteKind;
use indexmap::IndexMap;

use super::*;

use std::collections::HashSet;

impl<'a> LoweringCtx<'a> {
	pub(crate) fn lower_trait_decl(
		&mut self,
		trait_decl: ast::TraitDecl,
	) -> Result<TraitDecl, LowerError> {
		let name = self.unwrap_ident(
			trait_decl.name(),
			trait_decl.range(),
			format!("trait declaration missing name"),
		)?;

		let mut methods = HashMap::new();
		for method in trait_decl.methods() {
			let name = self.unwrap_ident(method.name(), method.range(), format!("method name"))?;
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

			let return_type = self.lower_type(method.return_ty(), &IndexMap::new())?;
			let method = TraitMethod {
				name: name.clone(),
				params,
				return_type,
			};
			methods.insert(name.inner, method);
		}
		Ok(TraitDecl { name, methods })
	}

	/// Figure out what traits are being implemented for what types without lowering the block
	pub(crate) fn apply_decl_first_pass(
		&mut self,
		apply_decl: ast::ApplyDecl,
	) -> Result<
		(
			(Option<Spanned<SmolStr>>, Spanned<Type>),
			IndexMap<SmolStr, HashSet<SmolStr>>,
			Option<ast::ApplyBlock>,
		),
		LowerError,
	> {
		let generics = match apply_decl.generics() {
			Some(generics) => self.lower_generic_list(generics, apply_decl.where_clause())?,
			None => IndexMap::new(),
		};
		let (trait_, ty): (Option<Spanned<SmolStr>>, Spanned<Type>) =
			match (apply_decl.trait_(), apply_decl.ty()) {
				(None, Some(ty)) => (None, self.lower_type(Some(ty), &generics)?),
				(Some(trait_), Some(ty)) => (
					Some(Spanned::new(
						trait_.text().into(),
						Span::new(trait_.text_range(), self.file_id.clone()),
					)),
					self.lower_type(Some(ty), &generics)?,
				),
				_ => unreachable!(),
			};
		Ok(((trait_, ty), generics, apply_decl.block()))
	}

	pub(crate) fn lower_apply_decl(
		&mut self,
		block: &Option<ast::ApplyBlock>,
		trait_: &Option<Spanned<SmolStr>>,
		ty: &Spanned<Type>,
		generics: &IndexMap<SmolStr, HashSet<SmolStr>>,
	) -> Result<ApplyDecl, LowerError> {
		let trait_decl = if let Some(trait_) = &trait_ {
			let trait_decl = match self.traits.get(&trait_.inner) {
				Some(decl) => decl,
				None => {
					return Err(LowerError::AppliedUnknownTrait {
						trt: trait_.clone(),
						ty: ty.map(|ty| self.fmt_ty(&ty)),
					});
				}
			};
			Some(*trait_decl)
		} else {
			None
		};

		let block = block.as_ref().unwrap();

		self.lower_and_validate_apply_block(block, &trait_decl, &ty, &generics)?;

		Ok(ApplyDecl {
			trait_: trait_.clone(),
			ty: ty.clone(),
			methods: vec![],
		})
	}

	fn lower_and_validate_apply_block(
		&mut self,
		apply_block: &ast::ApplyBlock,
		trait_decl: &Option<&TraitDecl>,
		ty: &Spanned<Type>,
		generics: &IndexMap<SmolStr, HashSet<SmolStr>>,
	) -> Result<(), LowerError> {
		let mut methods_implemented = HashSet::new();
		for method in apply_block.methods() {
			let method = self.lower_fn_decl(method, generics)?;

			let actual_return_ty_id = self
				.tchecker
				.tenv
				.insert(self.to_ty_kind(&method.return_type));

			if let Some(trait_decl) = trait_decl {
				if let Some(method_decl) = trait_decl.methods.get(&method.name.inner) {
					let decl_return_ty_id = self
						.tchecker
						.tenv
						.insert(self.to_ty_kind(&method_decl.return_type));

					self
						.tchecker
						.unify(
							actual_return_ty_id,
							decl_return_ty_id,
							method.return_type.span.clone(),
						)
						.map_err(LowerError::TypeError)?;

					self.validate_trait_method_implementation(method_decl, &method)?;
					methods_implemented.insert(method_decl.name.inner.as_str());
				} else {
					return Err(LowerError::AppliedUnknownMethodToTrait {
						trt: trait_decl.name.clone(),
						method: method.name.clone(),
						trt_methods: trait_decl
							.methods
							.keys()
							.map(|s| s.clone())
							.collect::<Vec<_>>(),
					});
				}
			}
		}

		if let Some(trait_decl) = trait_decl {
			let unimplemented_methods: Vec<_> = trait_decl
				.methods
				.iter()
				.filter_map(
					|(method, _)| match methods_implemented.get(method.as_str()) {
						Some(_) => None,
						None => Some(method.clone()),
					},
				)
				.collect();

			if unimplemented_methods.len() > 0 {
				return Err(LowerError::UnimplementedTraitMethods {
					trt: trait_decl.name.clone(),
					ty: ty.map(|ty| self.fmt_ty(&ty)),
					unimplemented_methods: unimplemented_methods,
				});
			}
		}

		Ok(())
	}

	fn validate_trait_method_implementation(
		&mut self,
		method_decl: &TraitMethod,
		method_impl: &FnDecl,
	) -> Result<(), LowerError> {
		let return_ty_id = self
			.tchecker
			.tenv
			.insert(self.to_ty_kind(&method_decl.return_type));
		let return_ty_impl_id = self
			.tchecker
			.tenv
			.insert(self.to_ty_kind(&method_impl.return_type));
		self
			.tchecker
			.unify(
				return_ty_id,
				return_ty_impl_id,
				method_impl.return_type.span.clone(),
			)
			.map_err(LowerError::TypeError)?;

		let method_decl_params = method_decl.params.len();
		let method_impl_params = method_impl.params.len();
		if method_decl_params != method_impl_params {
			return Err(LowerError::IncorrectNumberOfParamsInTraitMethodDefinition {
				method_name: method_decl.name.inner.to_string(),
				implementation_params: method_impl.params.clone(),
				declaration_params: method_decl.params.clone(),
			});
		}

		for (i, decl_param) in method_decl.params.iter().enumerate() {
			let impl_param = &method_impl.params[i];
			let decl_id = self.tchecker.tenv.insert(self.to_ty_kind(&decl_param.ty));
			let impl_id = self.tchecker.tenv.insert(self.to_ty_kind(&impl_param.ty));
			self
				.tchecker
				.unify(decl_id, impl_id, impl_param.ty.span.clone())
				.map_err(LowerError::TypeError)?;
		}

		Ok(())
	}

	pub(crate) fn lower_type_decl(&mut self, ty_decl: ast::TypeDecl) -> Result<TypeDecl, LowerError> {
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
		let name = self.unwrap_ident(
			ty_decl.name(),
			ty_decl.range(),
			format!("type declaration name"),
		)?;
		let ty = self.lower_type(ty_decl.ty(), &IndexMap::new())?;
		Ok(TypeDecl {
			visibility,
			name,
			ty,
		})
	}

	pub(crate) fn lower_fn_decl(
		&mut self,
		fn_decl: ast::FnDecl,
		generics: &IndexMap<SmolStr, HashSet<SmolStr>>,
	) -> Result<FnDecl, LowerError> {
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
			let ty = self.lower_type(Some(return_type), generics)?;
			let id = self.tchecker.tenv.insert(self.to_ty_kind(&ty));
			id
		} else {
			self.tchecker.tenv.insert(Spanned::new(
				TypeKind::Concrete(ConcreteKind::Tuple(vec![])),
				Span::new(
					TextRange::new(params_range.end(), params_range.end()),
					self.file_id.clone(),
				),
			))
		};
		self.tchecker.tenv.return_type_id = return_id;

		let ret_ty_unification_span = if let Expr::Block(block) = &self.exprs[body].inner {
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
			.unify(body_id, return_id, ret_ty_unification_span)
			.map_err(LowerError::TypeError)?;
		let return_type: Spanned<Type> = self.to_ty(&self.tchecker.tenv.get_type(return_id));

		let name = self.unwrap_ident(
			fn_decl.name(),
			fn_decl.range(),
			format!("function declaration name"),
		)?;

		let mut var_types: HashMap<SmolStr, Spanned<Type>> = HashMap::new(); // this is necessary cus mut ref and non-mut ref?
		if let Expr::Block(block) = &self.exprs[body].inner {
			for stmt in &block.0 {
				if let Stmt::VarDecl(var) = &stmt.inner {
					let id = self
						.tchecker
						.tenv
						.get_path_id(&vec![var.name.clone()])
						.map_err(LowerError::TypeError)?;
					let ty = self
						.tchecker
						.tenv
						.reconstruct(id)
						.map_err(LowerError::TypeError)?;
					var_types.insert(var.name.inner.clone(), self.to_ty(&ty));
				}
			}
		}

		if let Expr::Block(block) = &mut self.exprs[body].inner {
			for stmt in &mut block.0 {
				if let Stmt::VarDecl(var) = &mut stmt.inner {
					var.ty = var_types.get(&var.name.inner).unwrap().clone();
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
	) -> Result<Vec<Spanned<FnParam>>, LowerError> {
		let mut hir_params = vec![];
		for param in params {
			let name = if let Some(name) = param.name() {
				Some(name.text().into())
			} else {
				None
			};
			let ty = self.lower_type(param.ty(), &IndexMap::new())?;
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
