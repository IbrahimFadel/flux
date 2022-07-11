use indexmap::IndexMap;

use super::*;

type TypeResult = Result<Spanned<Type>, FluxError>;

impl LoweringCtx {
	pub(super) fn lower_type(&mut self, ty: Option<ast::Type>) -> TypeResult {
		if let Some(ty) = ty {
			match ty {
				ast::Type::PrimitiveType(primitive_ty) => self.lower_primitive_type(primitive_ty),
				ast::Type::StructType(struct_ty) => self.lower_struct_type(struct_ty),
				// ast::Type::InterfaceType(interface_ty) => self.lower_interface_type(interface_ty),
				ast::Type::IdentType(ident_ty) => self.lower_ident_type(ident_ty),
				ast::Type::TupleType(tuple_ty) => self.lower_tuple_type(tuple_ty),
				_ => Err(FluxError::build(
					format!("could not lower type"),
					self.span(&ty),
					FluxErrorCode::CouldNotLowerNode,
					(format!("could not lower type"), self.span(&ty)),
				)),
			}
		} else {
			Ok(self.default_spanned(Type::Unknown))
			// let id = self
			// .tchecker
			// .tenv
			// .insert(self.default_spanned(Type::Unknown));
			// Ok(id)
		}
	}

	fn lower_primitive_type(&mut self, primitive_ty: ast::PrimitiveType) -> TypeResult {
		if primitive_ty.ty().is_none() {
			return Err(FluxError::build(
				format!("could not lower primitive type"),
				self.default_span(),
				FluxErrorCode::CouldNotLowerNode,
				(
					format!("could not lower primitive type"),
					self.default_span(),
				),
			));
		}
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
					return Err(FluxError::build(
						format!(
							"could not lower primitive type: no such type as `{}`",
							ty.text()
						),
						self.span(&primitive_ty),
						FluxErrorCode::CouldNotLowerNode,
						(
							format!(
								"could not lower primitive type: no such type as `{}`",
								ty.text()
							),
							self.span(&primitive_ty),
						),
					));
				}
			}
			_ => {
				return Err(FluxError::build(
					format!(
						"could not lower primitive type: no such type as `{}`",
						ty.text()
					),
					self.span(&primitive_ty),
					FluxErrorCode::CouldNotLowerNode,
					(
						format!(
							"could not lower primitive type: no such type as `{}`",
							ty.text()
						),
						self.span(&primitive_ty),
					),
				));
			}
		};

		// let id = self
		// 	.tchecker
		// 	.tenv
		// 	.insert(Spanned::new(res, self.span(&primitive_ty)));
		// Ok(id)
		Spanned::new(res, self.span(&primitive_ty))
	}

	fn lower_struct_type(&mut self, struct_ty: ast::StructType) -> TypeResult {
		let mut hir_fields = IndexMap::new();
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
			let type_id = self.lower_type(field.type_())?;
			let ty = self.tchecker.tenv.get_type(type_id).into();
			hir_fields.insert(
				name,
				StructTypeField {
					visibility,
					mutable: field.mutable().is_some(),
					ty,
				},
			);
		}
		let hir_fields = Spanned::new(hir_fields, self.span(&struct_ty));
		let id = self.tchecker.tenv.insert(Spanned::new(
			Type::Struct(StructType(hir_fields)),
			self.span(&struct_ty),
		));
		Ok(id)
	}

	fn lower_ident_type(&mut self, ident_ty: ast::IdentType) -> TypeResult {
		let name = if let Some(name) = ident_ty.name() {
			SmolStr::from(name.text())
		} else {
			return Err(FluxError::build(
				format!("could not lower identifier type: missing text"),
				self.span(&ident_ty),
				FluxErrorCode::CouldNotLowerNode,
				(
					format!("could not lower identifier type: missing text"),
					self.span(&ident_ty),
				),
			));
		};
		let id = self.tchecker.tenv.insert(Spanned::new(
			Type::Ident(name),
			Span::new(ident_ty.range(), self.file_id.clone()),
		));
		Ok(id)
	}

	fn lower_tuple_type(&mut self, tuple_ty: ast::TupleType) -> TypeResult {
		let mut types = vec![];
		for ty in tuple_ty.types() {
			let ty = self.lower_type(Some(ty))?;
			types.push(ty);
		}
		let id = self
			.tchecker
			.tenv
			.insert(Spanned::new(Type::Tuple(types), self.span(&tuple_ty)));
		Ok(id)
	}
}
