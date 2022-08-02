use indexmap::IndexMap;
use tracing::{debug, instrument, trace};

use super::*;

type TypeResult = Result<Spanned<Type>, LowerError>;

impl<'a> LoweringCtx<'a> {
	#[instrument(skip(self))]
	pub(super) fn lower_type(&mut self, ty: Option<ast::Type>, generics: &GenericList) -> TypeResult {
		if let Some(ty) = ty {
			match ty {
				ast::Type::PrimitiveType(primitive_ty) => self.lower_primitive_type(primitive_ty),
				ast::Type::StructType(struct_ty) => self.lower_struct_type(struct_ty, generics),
				ast::Type::IdentType(ident_ty) => self.lower_ident_type(ident_ty, generics),
				ast::Type::TupleType(tuple_ty) => self.lower_tuple_type(tuple_ty, generics),
				ast::Type::PointerType(pointer_ty) => self.lower_pointer_type(pointer_ty, generics),
				ast::Type::EnumType(enum_ty) => self.lower_enum_type(enum_ty, generics),
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

	fn lower_struct_type(
		&mut self,
		struct_ty: ast::StructType,
		generics: &GenericList,
	) -> TypeResult {
		debug!("lowering struct type");
		let mut hir_fields = IndexMap::new();
		for field in struct_ty.fields() {
			let visibility = if field.public().is_some() {
				Visibility::Public
			} else {
				Visibility::Private
			};
			trace!(visibility = format!("{:?}", visibility));
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

	#[instrument(skip(self))]
	pub fn lower_generic_list(
		&self,
		generics: ast::GenericList,
		where_clause: Option<ast::WhereClause>,
	) -> Result<Spanned<GenericList>, LowerError> {
		let mut restrictions = IndexMap::new();
		for generic in generics.names() {
			let generic: SmolStr = generic.text().into();
			trace!(generic_name = generic.as_str());
			let mut traits = HashSet::new();
			if let Some(where_clause) = &where_clause {
				for restriction in where_clause.type_restrictions() {
					let restriction = self.lower_type_restriction(restriction)?;
					if restriction.name.inner == generic {
						trace!(restriction = restriction.trt.inner.as_str());
						traits.insert(restriction.trt.inner.clone());
					}
				}
			}
			restrictions.insert(generic, traits);
		}
		let restrictions = Spanned::new(
			restrictions,
			Span::new(generics.range(), self.file_id.clone()),
		);
		Ok(restrictions)
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

	fn lower_ident_type(&mut self, ident_ty: ast::IdentType, generics: &GenericList) -> TypeResult {
		let name = self.unwrap_ident(
			ident_ty.name(),
			ident_ty.range(),
			format!("identifier type"),
		)?;
		let type_params = if let Some(type_params) = ident_ty.type_params() {
			let params: Result<Vec<_>, _> = type_params
				.params()
				.map(|ty| self.lower_type(Some(ty), generics))
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
			match self.type_decls.get(&name.inner) {
				Some(_) => Type::Ident((name.inner, type_params)),
				_ => return Err(LowerError::UnknownType { ty: name }),
			}
		};
		Ok(Spanned::new(ty, name.span))
	}

	fn lower_tuple_type(&mut self, tuple_ty: ast::TupleType, generics: &GenericList) -> TypeResult {
		let mut types = vec![];
		for ty in tuple_ty.types() {
			let ty = self.lower_type(Some(ty), generics)?;
			types.push(self.tchecker.tenv.insert(self.to_ty_kind(&ty)));
		}
		Ok(Spanned::new(Type::Tuple(types), self.span(&tuple_ty)))
	}

	fn lower_pointer_type(
		&mut self,
		pointer_ty: ast::PointerType,
		generics: &GenericList,
	) -> TypeResult {
		let ty = self.lower_type(pointer_ty.to(), generics)?;
		let ty = Spanned::new(
			Type::Ptr(self.tchecker.tenv.insert(self.to_ty_kind(&ty))),
			Span::new(pointer_ty.range(), self.file_id.clone()),
		);
		Ok(ty)
	}

	fn lower_enum_type(&mut self, enum_type: ast::EnumType, generics: &GenericList) -> TypeResult {
		let mut fields = IndexMap::new();
		for field in enum_type.fields() {
			let name = self.unwrap_ident(field.name(), field.range(), format!("enum field name"))?;
			let ty = if let Some(ty) = field.ty() {
				Some(self.lower_type(Some(ty), generics)?)
			} else {
				None
			};

			fields.insert(name.inner, ty);
		}

		let ty = Spanned::new(Type::Enum(EnumType(fields)), self.span(&enum_type));
		Ok(ty)
	}
}
