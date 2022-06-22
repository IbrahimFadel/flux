use flux_typesystem::TypeId;

use super::*;

type TypeResult = Result<Spanned<TypeId>, FluxError>;

impl LoweringCtx {
	pub(super) fn lower_type(&mut self, ty: Option<ast::Type>) -> TypeResult {
		if let Some(ty) = ty {
			match ty {
				ast::Type::PrimitiveType(primitive_ty) => self.lower_primitive_type(primitive_ty),
				// ast::Type::StructType(struct_ty) => self.lower_struct_type(struct_ty),
				// ast::Type::InterfaceType(interface_ty) => self.lower_interface_type(interface_ty),
				ast::Type::IdentType(ident_ty) => self.lower_ident_type(ident_ty),
				_ => Err(FluxError::build(
					format!("could not lower type"),
					self.span(&ty),
					FluxErrorCode::CouldNotLowerNode,
					(format!("could not lower type"), self.span(&ty)),
				)),
			}
		} else {
			let id = self
				.tchecker
				.tenv
				.insert(self.default_spanned(Type::Unknown));
			Ok(self.default_spanned(id))
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

		let id = self
			.tchecker
			.tenv
			.insert(Spanned::new(res, self.span(&primitive_ty)));
		Ok(
			// Spanned::new(res, Span::new(primitive_ty.range(), self.file_id)),
			Spanned::new(id, self.span(&primitive_ty)),
		)
	}

	// fn lower_struct_type(&mut self, struct_ty: ast::StructType) -> TypeResult {
	// 	let mut hir_fields = IndexMap::new();
	// 	for field in struct_ty.fields() {
	// 		let name = if let Some(name) = field.name() {
	// 			SmolStr::from(name.text())
	// 		} else {
	// 			self.errors.push(
	// 				FluxError::default().with_msg(format!("could not lower struct type: field has no name")),
	// 			);
	// 			SmolStr::from("missing")
	// 		};
	// 		hir_fields.insert(
	// 			name,
	// 			StructTypeField {
	// 				public: field.public().is_some(),
	// 				mutable: field.mutable().is_some(),
	// 				ty: self.lower_type(field.type_())?,
	// 			},
	// 		);
	// 	}
	// 	Spanned::new(
	// 		Type::Struct(StructType(Spanned::new(
	// 			hir_fields,
	// 			Span::new(struct_ty.range(), self.file_id),
	// 		))),
	// 		Span::new(struct_ty.range(), self.file_id),
	// 	)
	// }

	// fn lower_interface_type(&mut self, interface_ty: ast::InterfaceType) -> Spanned<Type> {
	// 	let mut hir_methods = HashMap::new();
	// 	for method in interface_ty.methods() {
	// 		let name = if let Some(name) = method.name() {
	// 			SmolStr::from(name.text())
	// 		} else {
	// 			self.errors.push(FluxError::default().with_msg(format!(
	// 				"could not lower interface type: method has no name"
	// 			)));
	// 			SmolStr::from("missing")
	// 		};
	// 		hir_methods.insert(
	// 			name,
	// 			InterfaceMethod {
	// 				public: method.public().is_some(),
	// 				params: self.lower_params(method.params()),
	// 				return_ty: self.lower_type(method.return_ty()),
	// 			},
	// 		);
	// 	}
	// 	Spanned::new(
	// 		Type::Interface(InterfaceType(hir_methods)),
	// 		Span::new(interface_ty.range(), self.file_id),
	// 	)
	// }

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
		Ok(Spanned::new(
			id,
			Span::new(ident_ty.range(), self.file_id.clone()),
		))
	}
}
