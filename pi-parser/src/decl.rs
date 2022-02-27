use super::Parser;
use pi_ast::{Expr, FnDecl, FnParam, GenericTypes, PrimitiveKind, PrimitiveType};
use pi_error::PIErrorCode;
use pi_lexer::token::TokenKind;
use smol_str::SmolStr;

impl<'a> Parser<'a> {
	pub fn fn_decl(&mut self) -> FnDecl {
		self.next();
		let name = Parser::tok_val(
			self.program,
			self.expect(
				TokenKind::Ident,
				self.error(
					"expected identifier in function declaration".to_owned(),
					PIErrorCode::ParseExpectedIdentFnDecl,
					vec![
						(
							"expected identifier following `fn` keyword".to_owned(),
							self.tok().span.clone(),
						),
						(
							"(hint) name the function".to_owned(),
							self.tok().span.clone(),
						),
					],
				),
			),
		);
		if self.tok().kind != TokenKind::LParen && self.tok().kind != TokenKind::CmpLT {
			// if someone forgets an indentifier, then we shouldn't advance so that params / generics can be parsed
			self.next();
		}

		let mut generics = vec![];
		if self.tok().kind == TokenKind::CmpLT {
			generics = self.generic_types();
		}
		let params = self.params();
		let mut ret_ty = Expr::PrimitiveType(PrimitiveType::new(PrimitiveKind::Void));
		if self.tok().kind == TokenKind::Arrow {
			ret_ty = self.return_type();
		}
		let block = self.block();

		FnDecl::new(SmolStr::from(name), generics, params, ret_ty, block)
	}

	fn generic_types(&mut self) -> GenericTypes {
		self.next();
		let mut names = vec![];
		if self.tok().kind == TokenKind::CmpGT {
			self.errors.push(self.error(
				"expected indetifier in function generic type list".to_owned(),
				PIErrorCode::ParseExpectedIdentGenericTypeList,
				vec![(
					format!(
						"expected identifier, instead got `{}`",
						Parser::tok_val(self.program, self.tok())
					),
					self.tok().span.clone(),
				)],
			))
		}
		while self.tok().kind != TokenKind::CmpGT {
			let name = self.ident();
			names.push(name);
			if self.tok().kind != TokenKind::CmpGT {
				self.expect(
					TokenKind::Comma,
					self.error(
						"expected `,` between identifiers in generic type list".to_owned(),
						PIErrorCode::ParseExpectedCommaInGenericTypeList,
						vec![],
					),
				);
				self.next();
			}
		}
		self.expect(
			TokenKind::CmpGT,
			self.error(
				"expected `>` after identifiers in generic type list".to_owned(),
				PIErrorCode::ParseExpectedGTAfterGenericTypeList,
				vec![],
			),
		);
		self.next();

		return names;
	}

	fn params(&mut self) -> Vec<FnParam> {
		self.expect(
			TokenKind::LParen,
			self.error(
				"expected `(` before function parameter list".to_owned(),
				PIErrorCode::ParseExpectedLParenBeforeParamList,
				vec![(
					format!(
						"expected `(` before function parameter list, instead got `{}`",
						Parser::tok_val(self.program, self.tok())
					),
					self.tok().span.clone(),
				)],
			),
		);
		if self.tok().kind == TokenKind::LParen {
			self.next();
		}
		let mut params = vec![];
		while self.tok().kind != TokenKind::RParen {
			let param = self.param();
			params.push(param);
			if self.tok().kind != TokenKind::RParen {
				self.expect(
					TokenKind::Comma,
					self.error(
						"expected `,` between parameters in function parameter list".to_owned(),
						PIErrorCode::ParseExpectedCommaInParamList,
						vec![],
					),
				);
				self.next();
			}
		}
		self.expect(
			TokenKind::RParen,
			self.error(
				"expected `)` after function parameter list".to_owned(),
				PIErrorCode::ParseExpectedRParenAfterParamList,
				vec![],
			),
		);
		self.next();

		return params;
	}

	fn param(&mut self) -> FnParam {
		let mut mut_ = false;
		if self.tok().kind == TokenKind::Mut {
			mut_ = true;
			self.next();
		}

		let type_ = self.type_expr();
		let name = self.ident();

		FnParam::new(mut_, type_, name)
	}

	fn return_type(&mut self) -> Expr {
		self.next();
		let ty = self.type_expr();
		if ty == Expr::Error {
			if self.tok().kind != TokenKind::LBrace {
				self.next();
			}
		}
		return ty;
	}
}
