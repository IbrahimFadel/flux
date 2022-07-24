use super::*;
use crate::hir::*;
use flux_syntax::{
	ast::{self, AstNode},
	syntax_kind::{SyntaxKind, SyntaxToken},
};
use flux_typesystem::check::TypeChecker;
use itertools::Itertools;
use text_size::{TextRange, TextSize};

mod decls;
pub mod error;
mod exprs;
mod stmts;
#[cfg(test)]
mod tests;
mod types;

pub(super) struct LoweringCtx<'a> {
	pub exprs: Arena<Spanned<Expr>>,
	pub errors: Vec<LowerError>,
	pub tchecker: TypeChecker,
	file_id: FileId,
	pub traits: HashMap<SmolStr, &'a TraitDecl>,
	pub type_decls: HashMap<SmolStr, &'a TypeDecl>,
	// Type name -> (Method name -> Function Signature)
	pub method_signatures: HashMap<String, HashMap<SmolStr, TypeId>>,
}

impl<'a> LoweringCtx<'a> {
	pub fn new(file_id: FileId) -> Self {
		Self {
			exprs: Arena::default(),
			errors: vec![],
			tchecker: TypeChecker::new(HashMap::new(), HashMap::new()),
			file_id,
			traits: HashMap::new(),
			type_decls: HashMap::new(),
			method_signatures: HashMap::new(),
		}
	}

	fn to_ty(&mut self, kind: &Spanned<TypeKind>) -> Spanned<Type> {
		let ty = match &kind.inner {
			TypeKind::Concrete(ty) => match ty {
				ConcreteKind::SInt(n) => Type::SInt(*n),
				ConcreteKind::UInt(n) => Type::UInt(*n),
				ConcreteKind::F64 => Type::F64,
				ConcreteKind::F32 => Type::F32,
				ConcreteKind::Ident(name) => Type::Ident(name.clone()),
				ConcreteKind::Tuple(types) => Type::Tuple(types.to_vec()),
				ConcreteKind::Ptr(id) => {
					let ty = self.tchecker.tenv.get_type(*id).clone();
					Type::Ptr(self.tchecker.tenv.insert(ty))
				}
				_ => todo!("{:#?}", kind.inner),
			},
			TypeKind::Int(_) => Type::Int,
			TypeKind::Float(_) => Type::Float,
			TypeKind::Generic(g) => Type::Generic(g.clone()),
			TypeKind::Unknown => Type::Unknown,
			TypeKind::Ref(id) => return self.to_ty(&self.tchecker.tenv.get_type(*id).clone()),
		};
		Spanned {
			inner: ty,
			span: kind.span.clone(),
		}
	}

	fn to_ty_kind(&self, ty: &Spanned<Type>) -> Spanned<TypeKind> {
		let kind = match &ty.inner {
			Type::SInt(n) => TypeKind::Concrete(ConcreteKind::SInt(*n)),
			Type::UInt(n) => TypeKind::Concrete(ConcreteKind::UInt(*n)),
			Type::Int => TypeKind::Int(None),
			Type::F64 => TypeKind::Concrete(ConcreteKind::F64),
			Type::F32 => TypeKind::Concrete(ConcreteKind::F32),
			Type::Ptr(id) => TypeKind::Concrete(ConcreteKind::Ptr(*id)),
			Type::Float => TypeKind::Float(None),
			Type::Ident((name, type_params)) => {
				TypeKind::Concrete(ConcreteKind::Ident((name.clone(), type_params.clone())))
			}
			Type::Unknown => TypeKind::Unknown,
			Type::Generic(name) => TypeKind::Generic(name.clone()),
			Type::Tuple(ty) => TypeKind::Concrete(ConcreteKind::Tuple(ty.clone())),
			_ => todo!("{:#?}", ty.inner),
		};
		Spanned {
			inner: kind,
			span: ty.span.clone(),
		}
	}

	pub fn fmt_ty(&self, ty: &Type) -> String {
		match ty {
			Type::SInt(n) => format!("i{n}"),
			Type::UInt(n) => format!("u{n}"),
			Type::Int => format!("int"),
			Type::F64 => format!("f64"),
			Type::F32 => format!("f32"),
			Type::Float => format!("float"),
			Type::Ptr(id) => format!(
				"*{}",
				self.tchecker.tenv.fmt_ty(&self.tchecker.tenv.get_type(*id))
			),
			Type::Struct(struct_ty) => format!(
				"{{ {} }}",
				struct_ty
					.fields
					.iter()
					.map(|(name, ty)| format!("{} {}", name, self.fmt_ty(&ty.ty)))
					.join(", ")
			),
			Type::Generic((name, restrictions)) => format!(
				"{name}{}",
				if restrictions.len() > 0 {
					format!(": {}", restrictions.iter().join(", "))
				} else {
					format!("")
				}
			),
			Type::Ident((name, type_params)) => format!(
				"{name}{}",
				if type_params.len() == 0 {
					format!("")
				} else {
					format!(
						"<{}>",
						type_params
							.iter()
							.map(|type_id| {
								let ty = self.tchecker.tenv.get_type(*type_id);
								self.tchecker.tenv.fmt_ty(&ty)
							})
							.join(", ")
					)
				}
			),
			_ => todo!("{:#?}", ty),
		}
	}

	fn unwrap_ident(
		&self,
		ident: Option<SyntaxToken>,
		range: TextRange,
		msg: String,
	) -> Result<Spanned<SmolStr>, LowerError> {
		if let Some(ident) = ident {
			Ok(Spanned::new(
				ident.text().into(),
				Span::new(ident.text_range(), self.file_id.clone()),
			))
		} else {
			Err(LowerError::Missing {
				missing: Spanned::new(msg, Span::new(range, self.file_id.clone())),
			})
		}
	}

	fn unwrap_path(
		&self,
		ident: Option<ast::PathExpr>,
		range: TextRange,
		msg: String,
	) -> Result<Path, LowerError> {
		if let Some(path) = ident {
			let path = path
				.names()
				.map(|s| {
					Spanned::new(
						SmolStr::from(s.text()),
						Span::new(s.text_range(), self.file_id.clone()),
					)
				})
				.collect();
			Ok(path)
		} else {
			Err(LowerError::Missing {
				missing: Spanned::new(msg, Span::new(range, self.file_id.clone())),
			})
		}
	}

	fn default_spanned<T>(&self, node: T) -> Spanned<T> {
		Spanned::new(
			node,
			Span::new(
				TextRange::new(TextSize::from(0), TextSize::from(0)),
				self.file_id.clone(),
			),
		)
	}

	fn span(&self, syntax: &dyn AstNode) -> Span {
		Span::new(syntax.range(), self.file_id.clone())
	}
}
