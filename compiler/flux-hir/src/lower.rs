use super::*;
use crate::hir::*;
use flux_syntax::{
	ast::{self, AstNode},
	syntax_kind::{SyntaxKind, SyntaxToken},
};
use flux_typesystem::check::TypeChecker;
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
	tchecker: TypeChecker,
	file_id: FileId,
	pub traits: HashMap<SmolStr, &'a TraitDecl>,
}

impl<'a> LoweringCtx<'a> {
	pub fn new(file_id: FileId) -> Self {
		Self {
			exprs: Arena::default(),
			errors: vec![],
			tchecker: TypeChecker::new(),
			file_id,
			traits: HashMap::new(),
		}
	}

	fn to_ty(&self, kind: &Spanned<TypeKind>) -> Spanned<Type> {
		let ty = match &kind.inner {
			TypeKind::Concrete(ty) => match ty {
				ConcreteKind::SInt(n) => Type::SInt(*n),
				ConcreteKind::UInt(n) => Type::UInt(*n),
				ConcreteKind::F64 => Type::F64,
				ConcreteKind::F32 => Type::F32,
				ConcreteKind::Ident(name) => Type::Ident(name.clone()),
				ConcreteKind::Tuple(types) => Type::Tuple(
					types
						.iter()
						.map(|id| self.to_ty(&self.tchecker.tenv.get_type(*id)).inner)
						.collect::<Vec<_>>(),
				),
				_ => todo!(),
			},
			TypeKind::Int(_) => Type::Int,
			TypeKind::Float(_) => Type::Float,
			TypeKind::Unknown => Type::Unknown,
			_ => todo!(),
		};
		Spanned {
			inner: ty,
			span: kind.span.clone(),
		}
	}

	fn unwrap_ident(
		&self,
		ident: Option<SyntaxToken>,
		range: TextRange,
		msg: String,
	) -> Result<SmolStr, LowerError> {
		if let Some(ident) = ident {
			Ok(ident.text().into())
		} else {
			todo!()
			// Err(FluxError::build(
			// 	format!("trait method missing name"),
			// 	Span::new(range, self.file_id.clone()),
			// 	FluxErrorCode::TraitMethodMissingName,
			// 	(
			// 		format!("trait method missing name"),
			// 		Span::new(range, self.file_id.clone()),
			// 	),
			// ))
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
