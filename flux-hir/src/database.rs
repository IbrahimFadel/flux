use std::collections::HashMap;

use crate::{
	BinaryOp, Call, Expr, FnDecl, FnParam, If, Int, InterfaceMethod, InterfaceType, ModDecl,
	PrefixOp, Return, Stmt, StructField, StructType, Type, TypeDecl, UseDecl, VarDecl,
};
use flux_error::{filesystem::FileId, FluxError, FluxErrorCode, Span};
use flux_syntax::{
	ast::{self, AstNode, CallExpr, FloatExpr, IntExpr, Spanned},
	syntax_kind::SyntaxKind,
};
use indexmap::IndexMap;
use la_arena::{Arena, Idx};
use smol_str::SmolStr;
use text_size::{TextRange, TextSize};

#[derive(Debug, Clone)]
pub struct Database {
	pub exprs: Arena<Spanned<Expr>>,
	pub errors: Vec<FluxError>,
	file_id: FileId,
}

impl Database {
	pub fn new(file_id: FileId) -> Self {
		Self {
			exprs: Arena::default(),
			errors: vec![],
			file_id,
		}
	}
}

impl Database {
	pub(crate) fn lower_mod(&mut self, ast: ast::ModDecl) -> Option<ModDecl> {
		let name = ast.name()?;
		let name_str = SmolStr::from(name.text());
		let name_span = name.text_range();
		Some(ModDecl {
			name: Spanned::new(name_str, Span::new(name_span, self.file_id)),
		})
	}

	pub(crate) fn lower_use(&mut self, ast: ast::UseDecl) -> Option<UseDecl> {
		let mut path = vec![];
		let token_arr = ast.path();
		for tok in token_arr {
			path.push(Spanned::new(
				SmolStr::from(tok.text()),
				Span::new(tok.text_range(), self.file_id),
			));
		}
		Some(UseDecl { path })
	}

	pub(crate) fn lower_fn(&mut self, ast: ast::FnDecl) -> Option<FnDecl> {
		let public = if let Some(p) = ast.public() {
			Spanned::new(true, Span::new(p.text_range(), self.file_id))
		} else {
			Spanned::new(false, Span::new(ast.first_token_range(), self.file_id))
		};
		let params = self.lower_params(ast.params())?;
		let block = if let Some(block) = ast.body() {
			self.lower_block(block)
		} else {
			vec![]
		};
		let return_type = self.lower_type(ast.return_type());
		let return_type = if let Some(ty) = &return_type {
			if let Type::Missing = ty.node {
				Spanned::new(Type::VoidType, ty.span.clone())
			} else {
				return_type.unwrap()
			}
		} else {
			return_type.unwrap()
		};
		let name = if let Some(name) = ast.name() {
			Some(Spanned::new(
				SmolStr::from(name.text()),
				Span::new(name.text_range(), self.file_id),
			))
		} else {
			None
		};
		Some(FnDecl {
			public,
			name,
			params,
			block,
			return_type,
		})
	}

	pub(crate) fn lower_ty_decl(&mut self, ast: ast::TypeDecl) -> Option<TypeDecl> {
		let public = if let Some(public) = ast.public() {
			Spanned::new(true, Span::new(public.text_range(), self.file_id))
		} else {
			Spanned::new(false, Span::new(ast.first_token_range(), self.file_id))
		};
		let name = ast.name()?;
		let name = Spanned::new(
			SmolStr::from(name.text()),
			Span::new(name.text_range(), self.file_id),
		);
		Some(TypeDecl {
			public,
			name,
			ty: self.lower_type(ast.ty())?,
		})
	}

	fn lower_type(&mut self, ast: Option<ast::Type>) -> Option<Spanned<Type>> {
		if let Some(ast) = ast {
			match ast {
				ast::Type::PrimitiveType(ast) => self.lower_primitive_type(ast),
				ast::Type::StructType(ast) => self.lower_struct_type(ast),
				ast::Type::InterfaceType(ast) => self.lower_interface_type(ast),
				ast::Type::IdentType(ast) => self.lower_ident_type(ast),
			}
		} else {
			Some(Spanned::new(
				Type::Missing,
				Span::new(
					TextRange::new(TextSize::from(0), TextSize::from(0)),
					self.file_id,
				),
			))
		}
	}

	fn lower_ident_type(&mut self, ast: ast::IdentType) -> Option<Spanned<Type>> {
		Some(Spanned::new(
			Type::IdentType(ast.name()?.text().into()),
			Span::new(ast.range(), self.file_id),
		))
	}

	fn lower_interface_type(&mut self, ast: ast::InterfaceType) -> Option<Spanned<Type>> {
		let mut hir_methods = HashMap::new();
		let methods = ast.methods();
		for method in methods {
			if let Some(name) = method.name() {
				hir_methods.insert(
					name.to_string(),
					InterfaceMethod {
						public: method.public().is_some(),
						params: self.lower_params(method.params())?,
						return_ty: self.lower_type(method.return_ty())?,
					},
				);
			}
		}
		Some(Spanned::new(
			Type::InterfaceType(InterfaceType(hir_methods)),
			Span::new(ast.range(), self.file_id),
		))
	}

	fn lower_params(&mut self, ast: Vec<ast::FnParam>) -> Option<Vec<Spanned<FnParam>>> {
		let mut params = vec![];
		for param in ast {
			let name = if let Some(name) = param.name() {
				Some(name.text().into())
			} else {
				None
			};
			params.push(Spanned::new(
				FnParam {
					mutable: param.mutable().is_some(),
					ty: self.lower_type(param.ty())?,
					name,
				},
				Span::new(param.range(), self.file_id),
			));
		}
		Some(params)
	}

	fn lower_struct_type(&mut self, ast: ast::StructType) -> Option<Spanned<Type>> {
		let mut hir_fields = IndexMap::new();
		let fields = ast.fields();
		for field in fields {
			let name = field.name()?.text().to_string();
			hir_fields.insert(
				name,
				StructField {
					public: field.public().is_some(),
					mutable: field.mutable().is_some(),
					ty: self.lower_type(field.type_())?,
				},
			);
		}
		Some(Spanned::new(
			Type::StructType(StructType(hir_fields)),
			Span::new(ast.range(), self.file_id),
		))
	}

	fn lower_block(&mut self, ast: ast::BlockStmt) -> Vec<Option<Spanned<Stmt>>> {
		let mut block = vec![];
		let stmts = ast.stmts();
		for stmt in stmts {
			block.push(self.lower_stmt(stmt));
		}
		block
	}

	pub(crate) fn lower_stmt(&mut self, ast: ast::Stmt) -> Option<Spanned<Stmt>> {
		match ast {
			ast::Stmt::VarDecl(ref ast) => self.lower_var_decl(ast),
			ast::Stmt::IfStmt(ref ast) => self.lower_if_stmt(ast),
			ast::Stmt::ExprStmt(ref ast) => {
				let e = self.lower_expr(ast.expr());
				Some(Spanned::new(
					Stmt::Expr(e),
					Span::new(ast.range(), self.file_id),
				))
			}
			ast::Stmt::ReturnStmt(ref ast) => self.lower_return_stmt(ast),
			ast::Stmt::BlockStmt(_) => None,
		}
	}

	fn lower_return_stmt(&mut self, ast: &ast::ReturnStmt) -> Option<Spanned<Stmt>> {
		let value = self.lower_expr(ast.value());
		Some(Spanned::new(
			Stmt::Return(Return { value }),
			Span::new(ast.range(), self.file_id),
		))
	}

	fn lower_if_stmt(&mut self, ast: &ast::IfStmt) -> Option<Spanned<Stmt>> {
		let condition = self.lower_expr(ast.condition());
		let then = if let Some(then) = ast.then() {
			self.lower_block(then)
		} else {
			vec![]
		};
		let else_ifs = ast
			.else_ifs()
			.iter()
			.map(|else_if| self.lower_if_stmt(else_if))
			.collect();
		let else_ = if let Some(else_) = ast.else_() {
			self.lower_block(else_)
		} else {
			vec![]
		};

		Some(Spanned::new(
			Stmt::If(If::new(condition, then, else_ifs, else_)),
			Span::new(ast.range(), self.file_id),
		))
	}

	fn lower_var_decl(&mut self, ast: &ast::VarDecl) -> Option<Spanned<Stmt>> {
		Some(Spanned::new(
			Stmt::VarDecl(VarDecl {
				ty: self.lower_type(ast.ty())?,
				name: ast.name()?.text().into(),
				value: self.lower_expr(ast.value()),
			}),
			Span::new(ast.range(), self.file_id),
		))
	}

	pub(crate) fn lower_expr(&mut self, ast: Option<ast::Expr>) -> Idx<Spanned<Expr>> {
		let expr = if let Some(ast) = ast {
			ast
		} else {
			return self.exprs.alloc(Spanned::new(
				Expr::Missing,
				Span::new(
					TextRange::new(TextSize::from(0), TextSize::from(0)),
					self.file_id,
				),
			));
		};
		let range = expr.range();
		let idx = if let ast::Expr::ParenExpr(ast) = expr {
			self.lower_expr(ast.expr())
		} else {
			let expr = match expr {
				ast::Expr::BinExpr(ast) => self.lower_binary(ast),
				ast::Expr::IntExpr(ast) => self.lower_int(ast),
				ast::Expr::FloatExpr(ast) => self.lower_float(ast),
				ast::Expr::PrefixExpr(ast) => self.lower_unary(ast),
				ast::Expr::IdentExpr(ast) => Expr::Ident {
					val: ast.name().unwrap().text().into(),
				},
				ast::Expr::CallExpr(ast) => self.lower_call(ast),
				_ => unreachable!(),
			};
			self
				.exprs
				.alloc(Spanned::new(expr, Span::new(range, self.file_id)))
		};
		idx
	}

	fn lower_call(&mut self, ast: CallExpr) -> Expr {
		let callee = self.lower_expr(ast.callee());
		let args = ast.args().map(|arg| self.lower_expr(Some(arg))).collect();
		Expr::Call(Call { callee, args })
	}

	fn lower_float(&mut self, ast: FloatExpr) -> Expr {
		let tok = ast.tok();
		if let Some(tok) = tok {
			let text = tok.text();
			let n = text.parse::<f64>();
			if let Some(err) = n.as_ref().err() {
				self.errors.push(
					FluxError::default()
						.with_msg(format!(
							"could not lower float expression: {}",
							err.to_string()
						))
						.with_code(FluxErrorCode::HirParseIntString)
						.with_label(
							format!("could not lower float expression"),
							Some(Span::new(ast.syntax().text_range(), self.file_id)),
						),
				);
				return Expr::Missing;
			} else {
				return Expr::Float { n: n.unwrap() };
			}
		} else {
			Expr::Missing
		}
	}

	fn lower_int(&mut self, ast: IntExpr) -> Expr {
		let tok = ast.tok();
		if let Some(tok) = tok {
			let text = tok.text();
			let mut text = text.replace('_', "");
			let base: u32 = if text.len() > 2 {
				match &text[..2] {
					"0x" => {
						text = text[2..].to_owned();
						16
					}
					"0b" => {
						text = text[2..].to_owned();
						2
					}
					_ => 10,
				}
			} else {
				10
			};
			let n = u64::from_str_radix(text.as_str(), base);
			if let Some(err) = n.as_ref().err() {
				self.errors.push(
					FluxError::default()
						.with_msg(format!(
							"could not lower int expression: {}",
							err.to_string()
						))
						.with_code(FluxErrorCode::HirParseIntString)
						.with_label(
							format!("could not lower int expression"),
							Some(Span::new(ast.syntax().text_range(), self.file_id)),
						),
				);
				return Expr::Missing;
			} else {
				return Expr::Int(Int {
					n: n.unwrap(),
					ty: Type::UNType(32),
				});
			}
		} else {
			Expr::Missing
		}
	}

	fn lower_primitive_type(&mut self, ast: ast::PrimitiveType) -> Option<Spanned<Type>> {
		if let Some(ty) = ast.ty() {
			let first_char = &ty.text()[0..1];
			let rest_str = &ty.text()[1..];
			let bits: u32 = rest_str.parse().unwrap();
			if first_char == "u" {
				Some(Spanned::new(
					Type::UNType(bits),
					Span::new(ast.range(), self.file_id),
				))
			} else if first_char == "i" {
				Some(Spanned::new(
					Type::INType(bits),
					Span::new(ast.range(), self.file_id),
				))
			} else if first_char == "f" {
				if bits == 64 {
					Some(Spanned::new(
						Type::F64Type,
						Span::new(ast.range(), self.file_id),
					))
				} else if bits == 32 {
					Some(Spanned::new(
						Type::F32Type,
						Span::new(ast.range(), self.file_id),
					))
				} else {
					None
				}
			} else {
				None
			}
		} else {
			None
		}
	}

	fn lower_binary(&mut self, ast: ast::BinExpr) -> Expr {
		let op = match ast.op().unwrap().kind() {
			SyntaxKind::Plus => BinaryOp::Add,
			SyntaxKind::Minus => BinaryOp::Sub,
			SyntaxKind::Star => BinaryOp::Mul,
			SyntaxKind::Slash => BinaryOp::Div,
			SyntaxKind::CmpEq => BinaryOp::CmpEq,
			SyntaxKind::DoubleColon => BinaryOp::DoubleColon,
			_ => unreachable!(),
		};

		let lhs = self.lower_expr(ast.lhs());
		let rhs = self.lower_expr(ast.rhs());

		Expr::Binary { op, lhs, rhs }
	}

	fn lower_unary(&mut self, ast: ast::PrefixExpr) -> Expr {
		let op = match ast.op().unwrap().kind() {
			SyntaxKind::Minus => PrefixOp::Neg,
			_ => unreachable!(),
		};

		let expr = self.lower_expr(ast.expr());

		Expr::Prefix { op, expr }
	}
}
