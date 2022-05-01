use crate::{Expr, FnDecl, InfixOp, PrefixOp, Stmt};
use la_arena::Arena;
use pi_syntax::{ast, syntax_kind::SyntaxKind};

#[derive(Debug, Default)]
pub struct Database {
	exprs: Arena<Expr>,
}

impl Database {
	pub(crate) fn lower_fn(&mut self, ast: ast::FnDecl) -> Option<FnDecl> {
		let mut fn_block = vec![];
		let block = ast.block();
		if let Some(block) = block {
			let stmts = block.stmts();
			for stmt in stmts {
				fn_block.push(self.lower_stmt(stmt));
			}
		}
		Some(FnDecl { block: fn_block })
	}

	pub(crate) fn lower_stmt(&mut self, ast: ast::Stmt) -> Option<Stmt> {
		let result = match ast {
			ast::Stmt::VarDecl(ast) => Stmt::VarDecl {
				name: ast.name()?.text().into(),
				value: self.lower_expr(ast.value()),
			},
			_ => return None, // ast::Stmt::ExprStmt(ast) => Stmt::Expr(self.lower_expr(Some(ast))),
		};

		Some(result)
	}

	pub(crate) fn lower_expr(&mut self, ast: Option<ast::Expr>) -> Expr {
		if let Some(ast) = ast {
			match ast {
				ast::Expr::BinExpr(ast) => self.lower_binary(ast),
				ast::Expr::IntExpr(ast) => Expr::Int { n: ast.parse() },
				ast::Expr::ParenExpr(ast) => self.lower_expr(ast.expr()),
				ast::Expr::PrefixExpr(ast) => self.lower_unary(ast),
				ast::Expr::IdentExpr(ast) => Expr::Ident {
					val: ast.name().unwrap().text().into(),
				},
				_ => Expr::Missing,
			}
		} else {
			Expr::Missing
		}
	}

	fn lower_primitive_type(&mut self, ast: ast::PrimitiveType) -> Expr {
		// match ast
		Expr::Missing
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
