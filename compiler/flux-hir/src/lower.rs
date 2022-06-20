use super::*;
use crate::hir::*;
use flux_error::{FluxErrorCode, Span};
use flux_syntax::{
	ast::{self, AstNode},
	syntax_kind::SyntaxKind,
};
use flux_typesystem::{TypeEnv, TypeKind};
use text_size::{TextRange, TextSize};

mod exprs;
mod stmts;
#[cfg(test)]
mod tests;
mod types;

pub(super) struct LoweringCtx {
	pub exprs: Arena<Spanned<Expr>>,
	pub errors: Vec<FluxError>,
	tenv: TypeEnv<Type>,
	file_id: FileId,
}

impl LoweringCtx {
	pub fn new(file_id: FileId) -> Self {
		Self {
			exprs: Arena::default(),
			errors: vec![],
			tenv: TypeEnv::default(),
			file_id,
		}
	}

	pub fn lower_fn_decl(&mut self, fn_decl: ast::FnDecl) -> Result<FnDecl, FluxError> {
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

		let (mut return_type, return_id) = if let Some(return_type) = fn_decl.return_type() {
			self.lower_type(Some(return_type))?
		} else {
			(
				Spanned::new(
					Type::Unit,
					Span::new(
						TextRange::new(params_range.end(), params_range.end()),
						self.file_id,
					),
				),
				self.tenv.insert(TypeKind::Concrete(Type::Unit)),
			)
		};
		self.tenv.unify(body_id, return_id)?;
		return_type.node = type_system_reconstruction_to_hir_type(&self.tenv.reconstruct(body_id)?);

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
					let id = self.tenv.get_path_id(&[var.name.clone()]);
					var.ty.node = type_system_reconstruction_to_hir_type(&self.tenv.reconstruct(id)?);
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
			hir_params.push(Spanned::new(
				FnParam {
					mutable: param.mutable().is_some(),
					ty: self.lower_type(param.ty())?.0,
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
