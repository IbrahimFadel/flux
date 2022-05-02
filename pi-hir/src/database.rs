use crate::{Expr, FnDecl, INType, InfixOp, PrefixOp, Stmt, Type};
use la_arena::Arena;
use pi_syntax::{generated::ast, syntax_kind::SyntaxKind};

#[derive(Debug, Default)]
pub struct Database {
	exprs: Arena<Expr>,
}

impl Database {
	pub(crate) fn lower_fn(&mut self, ast: ast::FnDecl) -> Option<FnDecl> {
		let mut fn_block = vec![];
		let block = ast.body();
		if let Some(block) = block {
			let stmts = block.stmts();
			for stmt in stmts {
				fn_block.push(self.lower_stmt(stmt));
			}
		}
		let return_type = self.lower_type(ast.return_type());
		Some(FnDecl {
			block: fn_block,
			return_type,
		})
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

	pub(crate) fn lower_stmt(&mut self, ast: ast::Stmt) -> Option<Stmt> {
		match ast {
			ast::Stmt::VarDecl(ast) => self.lower_var_decl(ast),
			_ => None,
		}
	}

	fn lower_var_decl(&mut self, ast: ast::VarDecl) -> Option<Stmt> {
		Some(Stmt::VarDecl {
			ty: self.lower_type(ast.ty()),
			name: ast.name()?.text()?.text().into(),
			value: self.lower_expr(ast.value()),
		})
	}

	pub(crate) fn lower_expr(&mut self, ast: Option<ast::Expr>) -> Expr {
		if let Some(ast) = ast {
			match ast {
				ast::Expr::BinExpr(ast) => self.lower_binary(ast),
				ast::Expr::IntExpr(ast) => Expr::Int { n: ast.parse() },
				ast::Expr::ParenExpr(ast) => self.lower_expr(ast.expr()),
				ast::Expr::PrefixExpr(ast) => self.lower_unary(ast),
				ast::Expr::IdentExpr(ast) => Expr::Ident {
					val: ast.text().unwrap().text().into(),
				},
				_ => Expr::Missing,
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
			SyntaxKind::Plus => InfixOp::Add,
			SyntaxKind::Minus => InfixOp::Sub,
			SyntaxKind::Star => InfixOp::Mul,
			SyntaxKind::Slash => InfixOp::Div,
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
