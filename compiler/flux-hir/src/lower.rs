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

	pub fn lower_trait_decl(&mut self, trait_decl: ast::TraitDecl) -> Result<TraitDecl, FluxError> {
		let name = self.unwrap_ident(
			trait_decl.name(),
			trait_decl.range(),
			format!("trait declaration missing name"),
		)?;

		let mut methods = HashMap::new();
		for method in trait_decl.methods() {
			let name = if let Some(name) = method.name() {
				name.text().into()
			} else {
				return Err(FluxError::build(
					format!("trait method missing name"),
					Span::new(method.range(), self.file_id.clone()),
					FluxErrorCode::TraitMethodMissingName,
					(
						format!("trait method missing name"),
						Span::new(method.range(), self.file_id.clone()),
					),
				));
			};
			let params = self.lower_params(method.params())?;
			let return_ty_id = self.lower_type(method.return_ty())?;
			let return_ty = self.tchecker.tenv.get_type(return_ty_id).into();
			let method = TraitMethod { params, return_ty };
			methods.insert(name, method);
		}
		Ok(TraitDecl { name, methods })
	}

	pub fn lower_apply_decl(&mut self, apply_block: ast::ApplyDecl) -> Result<ApplyDecl, FluxError> {
		let (trait_, struct_) = match (apply_block.trait_(), apply_block.struct_()) {
			(None, None) => {
				return Err(FluxError::build(
					format!("missing struct to `apply` methods to"),
					self.span(&apply_block),
					FluxErrorCode::MissingStructToApplyMethodsTo,
					(
						format!("missing struct to `apply` methods to"),
						self.span(&apply_block),
					),
				))
			}
			(Some(struct_), None) => (None, struct_.text().into()),
			(Some(trait_), Some(struct_)) => (Some(trait_.text().into()), struct_.text().into()),
			_ => unreachable!(),
		};

		Ok(ApplyDecl {
			trait_,
			struct_,
			methods: vec![],
		})
	}

	pub fn lower_type_decl(&mut self, ty_decl: ast::TypeDecl) -> Result<TypeDecl, FluxError> {
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
		let name = if let Some(name) = ty_decl.name() {
			Spanned::new(
				name.text().into(),
				Span::new(name.text_range(), self.file_id.clone()),
			)
		} else {
			return Err(FluxError::build(
				format!("missing name in type declaration"),
				self.span(&ty_decl),
				FluxErrorCode::MissingNameTyDecl,
				(
					format!("missing name in type declaration"),
					self.span(&ty_decl),
				),
			));
		};
		let ty = self.lower_type(ty_decl.ty())?;
		let ty = self.tchecker.tenv.get_type(ty).into();
		Ok(TypeDecl {
			visibility,
			name,
			ty,
		})
	}

	pub fn lower_fn_decl(&mut self, fn_decl: ast::FnDecl) -> Result<FnDecl, FluxError> {
		self.tchecker.tenv = TypeEnv::new();

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
			self.lower_type(Some(return_type))?
		} else {
			self.tchecker.tenv.insert(Spanned::new(
				Type::Tuple(vec![]),
				Span::new(
					TextRange::new(params_range.end(), params_range.end()),
					self.file_id.clone(),
				),
			))
		};
		self.tchecker.tenv.return_type_id = return_id;

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
			.unify(body_id, return_id, ret_ty_unification_span)?;
		let return_type = self.tchecker.tenv.get_type(body_id).into();

		let name = if let Some(name) = fn_decl.name() {
			Spanned::new(
				SmolStr::from(name.text()),
				Span::new(name.text_range(), self.file_id.clone()),
			)
		} else {
			return Err(FluxError::build(
				format!("could not lower function declaration: missing name"),
				self.span(&fn_decl),
				FluxErrorCode::CouldNotLowerNode,
				(
					format!("could not lower function declaration: missing name"),
					self.span(&fn_decl),
				),
			));
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
			visibility,
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
			let ty = self.tchecker.tenv.reconstruct(ty)?.into();
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
