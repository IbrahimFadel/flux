use std::collections::HashMap;

use crate::{
	BinaryOp, Expr, FnDecl, FnParam, INType, If, InterfaceMethod, InterfaceType, PrefixOp, Stmt,
	StructField, StructType, Type, TypeDecl, UNType, VarDecl,
};
use flux_error::{filesystem::FileId, FluxError, FluxErrorCode, Span};
use flux_syntax::{
	ast::{self, AstNode, FloatExpr, IntExpr},
	syntax_kind::SyntaxKind,
};
use indexmap::IndexMap;
use la_arena::{Arena, ArenaMap, Idx};
use text_size::TextRange;

#[derive(Debug)]
pub struct Database {
	pub expr_ranges: ArenaMap<Idx<Expr>, TextRange>,
	pub exprs: Arena<Expr>,
	pub errors: Vec<FluxError>,
	file_id: FileId,
}

impl Database {
	pub fn new(file_id: FileId) -> Self {
		Self {
			expr_ranges: ArenaMap::default(),
			exprs: Arena::default(),
			errors: vec![],
			file_id,
		}
	}
}

impl Database {
	pub(crate) fn lower_fn(&mut self, ast: ast::FnDecl) -> Option<FnDecl> {
		let block = if let Some(block) = ast.body() {
			self.lower_block(block)
		} else {
			vec![]
		};
		let return_type = self.lower_type(ast.return_type())?;
		Some(FnDecl { block, return_type })
	}

	pub(crate) fn lower_ty_decl(&mut self, ast: ast::TypeDecl) -> Option<TypeDecl> {
		Some(TypeDecl {
			pub_: ast.public().is_some(),
			name: ast.name()?.text().to_string(),
			ty: self.lower_type(ast.ty())?,
		})
	}

	fn lower_type(&mut self, ast: Option<ast::Type>) -> Option<Type> {
		if let Some(ast) = ast {
			match ast {
				ast::Type::PrimitiveType(ast) => self.lower_primitive_type(ast),
				ast::Type::StructType(ast) => self.lower_struct_type(ast),
				ast::Type::InterfaceType(ast) => self.lower_interface_type(ast),
				ast::Type::IdentType(ast) => self.lower_ident_type(ast),
			}
		} else {
			Some(Type::Missing)
		}
	}

	fn lower_ident_type(&mut self, ast: ast::IdentType) -> Option<Type> {
		Some(Type::IdentType(ast.name()?.text().into()))
	}

	fn lower_interface_type(&mut self, ast: ast::InterfaceType) -> Option<Type> {
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
		Some(Type::InterfaceType(InterfaceType(hir_methods)))
	}

	fn lower_params(&mut self, ast: Vec<ast::FnParam>) -> Option<Vec<FnParam>> {
		let mut params = vec![];
		for param in ast {
			let name = if let Some(name) = param.name() {
				Some(name.text().into())
			} else {
				None
			};
			params.push(FnParam {
				mutable: param.mutable().is_some(),
				ty: self.lower_type(param.ty())?,
				name,
			});
		}
		Some(params)
	}

	fn lower_struct_type(&mut self, ast: ast::StructType) -> Option<Type> {
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
		Some(Type::StructType(StructType(hir_fields)))
	}

	fn lower_block(&mut self, ast: ast::BlockStmt) -> Vec<Option<Stmt>> {
		let mut block = vec![];
		let stmts = ast.stmts();
		for stmt in stmts {
			block.push(self.lower_stmt(stmt));
		}
		block
	}

	pub(crate) fn lower_stmt(&mut self, ast: ast::Stmt) -> Option<Stmt> {
		match ast {
			ast::Stmt::VarDecl(ref ast) => self.lower_var_decl(ast),
			ast::Stmt::IfStmt(ref ast) => self.lower_if_stmt(ast),
			_ => None,
		}
	}

	fn lower_if_stmt(&mut self, ast: &ast::IfStmt) -> Option<Stmt> {
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

		Some(Stmt::If(If::new(condition, then, else_ifs, else_)))
	}

	fn lower_var_decl(&mut self, ast: &ast::VarDecl) -> Option<Stmt> {
		Some(Stmt::VarDecl(VarDecl {
			ty: self.lower_type(ast.ty())?,
			name: ast.name()?.text().into(),
			value: self.lower_expr(ast.value()),
		}))
	}

	pub(crate) fn lower_expr(&mut self, ast: Option<ast::Expr>) -> Idx<Expr> {
		let expr = if let Some(ast) = ast {
			ast
		} else {
			return self.exprs.alloc(Expr::Missing);
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
				_ => unreachable!(),
			};
			self.exprs.alloc(expr)
		};
		self.expr_ranges.insert(idx, range);
		idx
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
				return Expr::Int { n: n.unwrap() };
			}
		} else {
			Expr::Missing
		}
	}

	fn lower_primitive_type(&mut self, ast: ast::PrimitiveType) -> Option<Type> {
		if let Some(ty) = ast.ty() {
			let first_char = &ty.text()[0..1];
			let rest_str = &ty.text()[1..];
			let bits: u32 = rest_str.parse().unwrap();
			if first_char == "u" {
				Some(Type::UNType(UNType { bits }))
			} else if first_char == "i" {
				Some(Type::INType(INType { bits }))
			} else if first_char == "f" {
				if bits == 64 {
					Some(Type::F64Type)
				} else if bits == 32 {
					Some(Type::F32Type)
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
