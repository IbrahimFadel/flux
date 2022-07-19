use indexmap::IndexMap;

use super::*;

type TypeResult = Result<Spanned<Type>, LowerError>;

impl<'a> LoweringCtx<'a> {
	pub(super) fn lower_type(
		&mut self,
		ty: Option<ast::Type>,
		generics: &HashMap<SmolStr, HashSet<SmolStr>>,
	) -> TypeResult {
		if let Some(ty) = ty {
			match ty {
				ast::Type::PrimitiveType(primitive_ty) => self.lower_primitive_type(primitive_ty),
				ast::Type::StructType(struct_ty) => self.lower_struct_type(struct_ty),
				ast::Type::IdentType(ident_ty) => self.lower_ident_type(ident_ty, generics),
				ast::Type::TupleType(tuple_ty) => self.lower_tuple_type(tuple_ty),
				ast::Type::PointerType(pointer_ty) => self.lower_pointer_type(pointer_ty),
			}
		} else {
			Ok(self.default_spanned(Type::Unknown))
		}
	}

	fn lower_primitive_type(&mut self, primitive_ty: ast::PrimitiveType) -> TypeResult {
		let ty = primitive_ty.ty().unwrap();
		let first_char = ty.text().chars().nth(0).unwrap();
		let rest_str = &ty.text()[1..];
		let bits: u32 = rest_str.parse().unwrap();
		let res = match first_char {
			'u' => Type::UInt(bits),
			'i' => Type::SInt(bits),
			'f' => {
				if bits == 32 {
					Type::F32
				} else if bits == 64 {
					Type::F64
				} else {
					todo!()
					// return Err(FluxError::build(
					// 	format!(
					// 		"could not lower primitive type: no such type as `{}`",
					// 		ty.text()
					// 	),
					// 	self.span(&primitive_ty),
					// 	FluxErrorCode::CouldNotLowerNode,
					// 	(
					// 		format!(
					// 			"could not lower primitive type: no such type as `{}`",
					// 			ty.text()
					// 		),
					// 		self.span(&primitive_ty),
					// 	),
					// ));
				}
			}
			_ => {
				todo!()
				// return Err(FluxError::build(
				// 		format!(
				// 			"could not lower primitive type: no such type as `{}`",
				// 			ty.text()
				// 		),
				// 		self.span(&primitive_ty),
				// 		FluxErrorCode::CouldNotLowerNode,
				// 		(
				// 			format!(
				// 				"could not lower primitive type: no such type as `{}`",
				// 				ty.text()
				// 			),
				// 			self.span(&primitive_ty),
				// 		),
				// 	));
			}
		};

		// let id = self
		// 	.tchecker
		// 	.tenv
		// 	.insert(Spanned::new(res, self.span(&primitive_ty)));
		// Ok(id)
		Ok(Spanned::new(res, self.span(&primitive_ty)))
	}

	fn lower_struct_type(&mut self, struct_ty: ast::StructType) -> TypeResult {
		let mut hir_fields = IndexMap::new();
		let generics = match struct_ty.generics() {
			Some(generics) => self.lower_generic_list(generics, struct_ty.where_clause())?,
			None => HashMap::new(),
		};
		for field in struct_ty.fields() {
			let visibility = if field.public().is_some() {
				Visibility::Public
			} else {
				Visibility::Private
			};
			let name = self.unwrap_ident(
				field.name(),
				struct_ty.range(),
				format!("missing name in struct type field"),
			)?;
			let ty = self.lower_type(field.type_(), &generics)?;
			hir_fields.insert(
				name.inner,
				StructTypeField {
					visibility,
					mutable: field.mutable().is_some(),
					ty,
				},
			);
		}
		let hir_fields = Spanned::new(hir_fields, self.span(&struct_ty));
		Ok(Spanned::new(
			Type::Struct(StructType(hir_fields)),
			self.span(&struct_ty),
		))
	}

	pub fn lower_generic_list(
		&self,
		generics: ast::GenericList,
		where_clause: Option<ast::WhereClause>,
	) -> Result<HashMap<SmolStr, HashSet<SmolStr>>, LowerError> {
		let mut restrictions = HashMap::new();
		for generic in generics.names() {
			let generic: SmolStr = generic.text().into();
			if let Some(where_clause) = &where_clause {
				let mut traits = HashSet::new();
				for restriction in where_clause.type_restrictions() {
					let restriction = self.lower_type_restriction(restriction)?;
					if restriction.name.inner == generic {
						traits.insert(restriction.trt.inner.clone());
					}
				}
				restrictions.insert(generic, traits);
			}
		}
		Ok(restrictions)
	}

	fn lower_where_clause(&self, where_clause: ast::WhereClause) -> Result<WhereClause, LowerError> {
		let restrictions: Result<Vec<_>, _> = where_clause
			.type_restrictions()
			.map(|restriction| self.lower_type_restriction(restriction))
			.collect();
		Ok(WhereClause(restrictions?))
	}

	fn lower_type_restriction(
		&self,
		type_restriction: ast::TypeRestriction,
	) -> Result<TypeRestriction, LowerError> {
		let name = self.unwrap_ident(
			type_restriction.name(),
			type_restriction.range(),
			format!("missing name of type parameter in type restriction"),
		)?;
		let trt = self.unwrap_ident(
			type_restriction.trait_(),
			type_restriction.range(),
			format!("missing name of trait in type restriction"),
		)?;
		Ok(TypeRestriction { name, trt })
	}

	fn lower_ident_type(
		&mut self,
		ident_ty: ast::IdentType,
		generics: &HashMap<SmolStr, HashSet<SmolStr>>,
	) -> TypeResult {
		let name = self.unwrap_ident(
			ident_ty.name(),
			ident_ty.range(),
			format!("identifier type"),
		)?;
		let type_params = if let Some(type_params) = ident_ty.type_params() {
			let params: Result<Vec<_>, _> = type_params
				.params()
				.map(|ty| self.lower_type(Some(ty), &HashMap::new()))
				.collect();
			params?
				.iter()
				.map(|param| self.tchecker.tenv.insert(self.to_ty_kind(param)))
				.collect()
		} else {
			vec![]
		};
		let ty = if let Some(restrictions) = generics.get(&name.inner) {
			Type::Generic((name.inner.clone(), restrictions.clone()))
		} else {
			Type::Ident((name.inner, type_params))
		};
		Ok(Spanned::new(ty, name.span))
	}

	fn lower_tuple_type(&mut self, tuple_ty: ast::TupleType) -> TypeResult {
		let mut types = vec![];
		for ty in tuple_ty.types() {
			let ty = self.lower_type(Some(ty), &HashMap::new())?;
			types.push(ty.inner);
		}
		Ok(Spanned::new(Type::Tuple(types), self.span(&tuple_ty)))
	}

	fn lower_pointer_type(&mut self, pointer_ty: ast::PointerType) -> TypeResult {
		let ty = self.lower_type(pointer_ty.to(), &HashMap::new())?;
		Ok(ty)
	}
}
