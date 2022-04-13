use pi_ast::{BlockStmt, Expr, If, Mod, Return, Stmt, VarDecl};
use pi_error::PIErrorCode;
use pi_lexer::token::TokenKind;

use super::Parser;

impl<'a> Parser<'a> {
	pub fn mod_stmt(&mut self, pub_: bool) -> Mod {
		self.next();
		let name = self.ident();
		self.expect(
			TokenKind::Semicolon,
			self.error(
				"expected `;` after mod statement".to_owned(),
				PIErrorCode::ParseExpectedSemicolonAfterModStmt,
				vec![],
			),
		);
		if self.tok().kind == TokenKind::Semicolon {
			self.next();
		}
		Mod::new(pub_, name)
	}

	pub fn block(&mut self) -> BlockStmt {
		self.expect(
			TokenKind::LBrace,
			self.error(
				"expected `{` at beginning of block".to_owned(),
				PIErrorCode::ParseExpectedLBraceInBlock,
				vec![(
					format!(
						"expected `{{` at beginning of block, instead got {}",
						Parser::tok_val(self.program, self.tok())
					),
					self.tok().span.clone(),
				)],
			),
		);
		if self.tok().kind == TokenKind::LBrace {
			self.next();
		}

		let mut stmts = vec![];
		while self.tok().kind != TokenKind::RBrace && self.tok().kind != TokenKind::EOF {
			let stmt = self.stmt();
			if stmt == Stmt::Error {
				break;
			}
			stmts.push(stmt);
		}

		self.expect(
			TokenKind::RBrace,
			self.error(
				"expected `}` at end of block".to_owned(),
				PIErrorCode::ParseExpectedRBraceInBlock,
				vec![(
					format!(
						"expected `}}` at end of block, instead got {}",
						Parser::tok_val(self.program, self.tok())
					),
					self.tok().span.clone(),
				)],
			),
		);
		if self.tok().kind == TokenKind::RBrace {
			self.next();
		}

		return stmts;
	}

	fn stmt(&mut self) -> Stmt {
		if self.tok().kind > TokenKind::TypesBegin && self.tok().kind < TokenKind::TypesEnd {
			return self.var_decl_stmt();
		} else if self.tok().kind == TokenKind::Return {
			return self.return_stmt();
		} else if self.tok().kind == TokenKind::If {
			return self.if_stmt();
		} else {
			let expr = self.expr();
			self.expect(
				TokenKind::Semicolon,
				self.error(
					"expected `;` after expression".to_owned(),
					PIErrorCode::ParseExpectedSemicolonAfterExpr,
					vec![],
				),
			);
			if self.tok().kind == TokenKind::Semicolon {
				self.next();
			}
			if expr == Expr::Error {
				return Stmt::Error;
			}
			return Stmt::ExprStmt(expr);
		}
	}

	fn if_stmt(&mut self) -> Stmt {
		self.next();
		let condition = self.expr();
		let then = self.block();
		let mut else_ = None;
		if self.tok().kind == TokenKind::Else {
			self.next();
			if self.tok().kind == TokenKind::If {
				let if_ = self.if_stmt();
				else_ = Some(vec![if_]);
			} else {
				else_ = Some(self.block());
			}
		}
		Stmt::If(If::new(Box::from(condition), then, else_))
	}

	fn return_stmt(&mut self) -> Stmt {
		self.next();
		if self.tok().kind == TokenKind::Semicolon {
			self.next();
			return Stmt::Return(Return::new(None));
		}
		let expr = self.expr();
		self.expect(
			TokenKind::Semicolon,
			self.error(
				"expected `;` after return statement".to_owned(),
				PIErrorCode::ParseExpectedSemicolonAfterReturnStmt,
				vec![],
			),
		);
		if self.tok().kind == TokenKind::Semicolon {
			self.next();
		}
		Stmt::Return(Return::new(Some(expr)))
	}

	fn var_decl_stmt(&mut self) -> Stmt {
		let ty = self.type_expr();

		if self.tok().kind != TokenKind::Ident {
			self.errors.push(self.error(
				"expected identifier in variable declaration".to_owned(),
				PIErrorCode::ParseExpectedIdentVarDecl,
				vec![(
					format!(
						"expected identifier in variable declaration, instead got `{}`",
						Parser::tok_val(self.program, self.tok())
					),
					self.tok().span.clone(),
				)],
			));
		}

		let names_start = self.tok().span.start;
		let mut names_end = self.tok().span.end;
		let mut names = vec![];
		while self.tok().kind != TokenKind::Eq && self.tok().kind != TokenKind::Semicolon {
			names_end = self.tok().span.clone().end;
			let name = self.ident();
			names.push(name);
			if self.tok().kind != TokenKind::Eq && self.tok().kind != TokenKind::Semicolon {
				self.expect(
					TokenKind::Comma,
					self.error(
						"expected `,` in variable declaration identifier list".to_owned(),
						PIErrorCode::ParseExpectedCommaInVarDeclIdentList,
						vec![(
							format!(
								"expected `,` in variable declaration identifier list, instead got `{}`",
								Parser::tok_val(self.program, self.tok())
							),
							self.tok().span.clone(),
						)],
					),
				);
				if self.tok().kind == TokenKind::Comma {
					self.next();
				}
			}
		}

		if self.tok().kind == TokenKind::Semicolon {
			self.next();
			return Stmt::VarDecl(VarDecl::new(ty, names, None));
		}
		self.expect(
				TokenKind::Eq,
				self.error(
					"expected either `;` or `=` after variable declaration identifier list".to_owned(),
					PIErrorCode::ParseExpectedSemicolonEqVarDeclIdentList,
					vec![
						(format!("expected either `;` or `=` after variable declaration identifier list, instead got `{}`", Parser::tok_val(self.program, self.tok())), self.tok().span.clone())
					],
				),
			);
		self.next();

		let values_start = self.tok().span.start;
		let mut values_end = self.tok().span.end;
		let mut values = vec![];
		while self.tok().kind != TokenKind::Semicolon {
			values_end = self.tok().span.end;
			let val = self.expr();
			values.push(val);
			if self.tok().kind == TokenKind::Comma {
				self.next();
				if self.tok().kind == TokenKind::Semicolon {
					self.errors.push(self.error(
						"expected expression after comma in variable declaration value list".to_owned(),
						PIErrorCode::ParseExpectedExprAfterCommaVarDeclValueList,
						vec![(
							format!(
								"expected expression after comma in variable declaration value list, instead got `{}`",
								Parser::tok_val(self.program, self.tok())
							),
							self.tok().span.clone(),
						)],
					));
					break;
				}
			} else if self.tok().kind != TokenKind::Semicolon {
				self.errors.push(self.error(
					"expected `;` after variable declaration".to_owned(),
					PIErrorCode::ParseExpectedSemicolonAfterVarDecl,
					vec![(
						format!(
							"expected `;` after variable declaration, instead got `{}`",
							Parser::tok_val(self.program, self.tok())
						),
						self.tok().span.clone(),
					)],
				));
				break;
			}
		}
		if self.tok().kind == TokenKind::Semicolon {
			self.next();
		}

		if names.len() > 1 && values.len() > 1 && values.len() != names.len() {
			if names.len() > values.len() {
				self.errors.push(self.error("more variables than values in variable declaration".to_owned(), PIErrorCode::ParseMoreIdentsThanValsVarDecl, vec![
				(format!("more variables than values in variable declaration: {} values assigned to {} variables", values.len(), names.len()), names_start..self.tok().span.end),
				(format!("{} variables declared", names.len()), names_start..names_end),
				(format!("{} values assigned", values.len()), values_start..values_end),
				("(hint) you can assign one value to multiple variables".to_owned(), names_start..self.tok().span.end),
			]));
			} else {
				self.errors.push(self.error("more values than variables in variable declaration".to_owned(), PIErrorCode::ParseMoreValsThanIdentsVarDecl, vec![
				(format!("more values than variables in variable declaration: {} values assigned to {} variables", values.len(), names.len()), names_start..self.tok().span.end),
				(format!("{} variables declared", names.len()), names_start..names_end),
				(format!("{} values assigned", values.len()), values_start..values_end),
			]));
			}
		}

		return Stmt::VarDecl(VarDecl::new(ty, names, Some(values)));
	}
}
