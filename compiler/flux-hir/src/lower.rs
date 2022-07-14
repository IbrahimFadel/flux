use self::error::TypeCheckErrHandler;

use super::*;
use crate::hir::*;
use flux_error::{FluxErrorCode, Span};
use flux_syntax::{
	ast::{self, AstNode},
	syntax_kind::{SyntaxKind, SyntaxToken},
};
use flux_typesystem::TypeChecker;
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
	pub errors: Vec<FluxError>,
	tchecker: TypeChecker<TypeCheckErrHandler>,
	file_id: FileId,
	pub traits: HashMap<SmolStr, &'a TraitDecl>,
}

impl<'a> LoweringCtx<'a> {
	pub fn new(err_handler: TypeCheckErrHandler, file_id: FileId) -> Self {
		Self {
			exprs: Arena::default(),
			errors: vec![],
			tchecker: TypeChecker::new(err_handler),
			file_id,
			traits: HashMap::new(),
		}
	}

	fn unwrap_ident(
		&self,
		ident: Option<SyntaxToken>,
		range: TextRange,
		msg: String,
	) -> Result<SmolStr, FluxError> {
		if let Some(ident) = ident {
			Ok(ident.text().into())
		} else {
			Err(FluxError::build(
				format!("trait method missing name"),
				Span::new(range, self.file_id.clone()),
				FluxErrorCode::TraitMethodMissingName,
				(
					format!("trait method missing name"),
					Span::new(range, self.file_id.clone()),
				),
			))
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

	fn default_span(&self) -> Span {
		Span::new(TextRange::default(), self.file_id.clone())
	}
}
