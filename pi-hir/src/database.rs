use crate::{BinaryOp, Expr, FnDecl, INType, If, PrefixOp, Stmt, Type};
use la_arena::Arena;
use pi_error::{PIError, PIErrorCode};
use pi_syntax::{
	ast::{self, IntExpr},
	syntax_kind::SyntaxKind,
};

#[derive(Debug, Default)]
pub struct Database {
	exprs: Arena<Expr>,
	errors: Vec<PIError>,
}

impl Database {
	pub(crate) fn lower_fn(&mut self, ast: ast::FnDecl) -> Option<FnDecl> {
		let block = if let Some(block) = ast.body() {
			self.lower_block(block)
		} else {
			vec![]
		};
		let return_type = self.lower_type(ast.return_type());
		Some(FnDecl { block, return_type })
	}

	fn lower_type(&mut self, ast: Option<ast::Type>) -> Type {
		if let Some(ast) = ast {
			match ast {
				ast::Type::PrimitiveType(ast) => self.lower_primitive_type(ast),
				_ => Type::Missing,
			}
		} else {
			Type::Missing
		}
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
		Some(Stmt::VarDecl {
			ty: self.lower_type(ast.ty()),
			name: ast.name()?.text().into(),
			value: self.lower_expr(ast.value()),
		})
	}

	pub(crate) fn lower_expr(&mut self, ast: Option<ast::Expr>) -> Expr {
		if let Some(ast) = ast {
			match ast {
				ast::Expr::BinExpr(ast) => self.lower_binary(ast),
				ast::Expr::IntExpr(ast) => self.lower_int(ast),
				ast::Expr::ParenExpr(ast) => self.lower_expr(ast.expr()),
				ast::Expr::PrefixExpr(ast) => self.lower_unary(ast),
				ast::Expr::IdentExpr(ast) => Expr::Ident {
					val: ast.name().unwrap().text().into(),
				},
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
					// "08" => {
					// text = text[2..].to_owned();
					// 8
					// }
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
					PIError::default()
						.with_msg(format!(
							"could not lower int expression: {}",
							err.to_string()
						))
						.with_code(PIErrorCode::HirParseIntString),
				);
				return Expr::Missing;
			} else {
				return Expr::Int { n: n.unwrap() };
			}
		} else {
			Expr::Missing
		}
	}

	fn lower_primitive_type(&mut self, ast: ast::PrimitiveType) -> Type {
		if let Some(ty) = ast.ty() {
			let int_str = &ty.text()[1..];
			let bits: u32 = int_str.parse().unwrap();
			Type::INType(INType { bits })
		} else {
			Type::Missing
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

		Expr::Binary {
			op,
			lhs: self.exprs.alloc(lhs),
			rhs: self.exprs.alloc(rhs),
		}
	}

	fn lower_unary(&mut self, ast: ast::PrefixExpr) -> Expr {
		let op = match ast.op().unwrap().kind() {
			SyntaxKind::Minus => PrefixOp::Neg,
			_ => unreachable!(),
		};

		let expr = self.lower_expr(ast.expr());

		Expr::Prefix {
			op,
			expr: self.exprs.alloc(expr),
		}
	}
}
