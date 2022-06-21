use self::error::TypeCheckErrHandler;

use super::*;
use crate::hir::*;
use flux_error::{FluxErrorCode, Span};
use flux_syntax::{
	ast::{self, AstNode},
	syntax_kind::SyntaxKind,
};
use flux_typesystem::TypeChecker;
use text_size::{TextRange, TextSize};

pub mod error;
mod exprs;
mod stmts;
#[cfg(test)]
mod tests;
mod types;

pub(super) struct LoweringCtx {
	pub exprs: Arena<Spanned<Expr>>,
	pub errors: Vec<FluxError>,
	tchecker: TypeChecker<TypeCheckErrHandler>,
	file_id: FileId,
}

impl LoweringCtx {
	pub fn new(err_handler: TypeCheckErrHandler, file_id: FileId) -> Self {
		Self {
			exprs: Arena::default(),
			errors: vec![],
			tchecker: TypeChecker::new(err_handler),
			file_id,
		}
	}

	pub fn lower_fn_decl(&mut self, fn_decl: ast::FnDecl) -> Result<FnDecl, FluxError> {
		self.tchecker.tenv = TypeEnv::new();

		let public = if let Some(p) = fn_decl.public() {
			Spanned::new(true, Span::new(p.text_range(), self.file_id))
		} else {
			Spanned::new(
				false,
				Span::new(fn_decl.first_token().unwrap().text_range(), self.file_id),
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
		let params = Spanned::new(params, Span::new(params_range, self.file_id));

		let (body, body_id) = self.lower_expr(fn_decl.body())?;

		let return_id = if let Some(return_type) = fn_decl.return_type() {
			self.lower_type(Some(return_type))?
		} else {
			Spanned::new(
				self.tchecker.tenv.insert(Spanned::new(
					Type::Unit,
					Span::new(
						TextRange::new(params_range.end(), params_range.end()),
						self.file_id,
					),
				)),
				Span::new(
					TextRange::new(params_range.end(), params_range.end()),
					self.file_id,
				),
			)
		};
		self.tchecker.tenv.return_type_id = return_id.node;

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
			.unify(body_id, return_id.node, ret_ty_unification_span)?;
		let return_type = self.tchecker.tenv.get_type(body_id).into();

		let name = if let Some(name) = fn_decl.name() {
			Spanned::new(
				SmolStr::from(name.text()),
				Span::new(name.text_range(), self.file_id),
			)
		} else {
			return Err(FluxError::default().with_msg(format!(
				"could not lower function declaration: missing name"
			)));
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
			public,
			name,
			params,
			body,
			return_type,
		})
	}

	fn lower_params(
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
			let ty = self.tchecker.tenv.reconstruct(ty.node)?.into();
			hir_params.push(Spanned::new(
				FnParam {
					mutable: param.mutable().is_some(),
					ty,
					name,
				},
				Span::new(param.range(), self.file_id),
			));
		}
		Ok(hir_params)
	}

	fn default_spanned<T>(&self, node: T) -> Spanned<T> {
		Spanned::new(
			node,
			Span::new(
				TextRange::new(TextSize::from(0), TextSize::from(0)),
				self.file_id,
			),
		)
	}
}
