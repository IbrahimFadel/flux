use std::vec;

use pi_ast::{
	BinOp, CallExpr, CharLit, Expr, Field, FloatLit, Ident, IntLit, Method, PrimitiveKind,
	PrimitiveType, StringLit, Unary,
};
use pi_error::PIErrorCode;
use pi_lexer::token::TokenKind;
use smol_str::SmolStr;

use super::Parser;

impl<'a> Parser<'a> {
	pub fn expr(&mut self) -> Expr {
		self.binop_expr(1)
	}

	fn binop_expr(&mut self, prec1: i8) -> Expr {
		let mut x = self.unary_expr();
		loop {
			let oprec = Parser::get_tokprec(&self.tok().kind);
			let op = self.tok().kind.clone();
			if oprec < prec1 {
				return x;
			}

			self.next();

			let y = self.binop_expr(oprec + 1);

			let binop = Expr::BinOp(BinOp::new(
				Box::from(x.clone()),
				self.token_kind_to_op_kind(&op),
				Box::from(y),
			));
			let post = self.postfix(binop);
			x = post;
		}
	}

	fn postfix(&mut self, x: Expr) -> Expr {
		match self.tok().kind {
			TokenKind::LParen => self.call(x),
			_ => x,
		}
	}

	fn call(&mut self, x: Expr) -> Expr {
		self.expect(
			TokenKind::LParen,
			self.error(
				"expected `(` at beginning of call expression".to_owned(),
				PIErrorCode::ParseExpectedLParenBeforeCallExpr,
				vec![],
			),
		);
		if self.tok().kind == TokenKind::LParen {
			self.next();
		}

		let mut args = vec![];
		while self.tok().kind != TokenKind::RParen {
			let arg = self.expr();
			args.push(Box::from(arg));

			if self.tok().kind != TokenKind::RParen {
				self.expect(
					TokenKind::Comma,
					self.error(
						"expected `,` in call args".to_owned(),
						PIErrorCode::ParseExpectedCommaInCallArgs,
						vec![],
					),
				);
				if self.tok().kind == TokenKind::Comma {
					self.next();
				} else {
					break;
				}
			}
		}

		self.expect(
			TokenKind::RParen,
			self.error(
				"expected `)` at beginning of call expression".to_owned(),
				PIErrorCode::ParseExpectedRParenAfterCallExpr,
				vec![],
			),
		);
		if self.tok().kind == TokenKind::RParen {
			self.next();
		}
		return Expr::CallExpr(CallExpr::new(Box::from(x), args));
	}

	fn unary_expr(&mut self) -> Expr {
		let kind = self.tok().kind.clone();
		match kind {
			TokenKind::Ampersand => Expr::Unary(Unary::new(kind, Box::from(self.expr()))),
			_ => self.primary_expr(),
		}
	}

	fn primary_expr(&mut self) -> Expr {
		let x = self.operand();
		return x;
		// loop {
		// match self.tok().kind {
		// TokenKind::Period =>

		// }
		// }
	}

	fn operand(&mut self) -> Expr {
		match self.tok().kind {
			TokenKind::Ident => Expr::Ident(self.ident()),
			TokenKind::Int | TokenKind::Float | TokenKind::CharLit | TokenKind::StringLit => {
				self.basic_lit()
			}
			// TokenKind::Nil => self.nil_expr(),
			_ => {
				self.errors.push(self.error(
					"unexpected expression operand".to_owned(),
					PIErrorCode::ParseUnexpectedExprOperand,
					vec![(
						format!(
							"unexpected expression operand `{}`",
							Parser::tok_val(self.program, self.tok())
						),
						self.tok().span.clone(),
					)],
				));
				Expr::Error
			}
		}
	}

	fn basic_lit(&mut self) -> Expr {
		self.expect_range(
			TokenKind::BasicLitBegin,
			TokenKind::BasicLitEnd,
			self.error(
				"expected a basic literal expression".to_owned(),
				PIErrorCode::ParseExpectedBasicLit,
				vec![(
					format!(
						"expected a basic literal expression, instead got `{}`",
						Parser::tok_val(self.program, self.tok())
					),
					self.tok().span.clone(),
				)],
			),
		);

		match self.tok().kind {
			TokenKind::Int => {
				let mut str_val = Parser::tok_val(self.program, &self.tok());
				let base = if str_val.len() > 2 {
					match &str_val.as_str()[0..2] {
						"0x" => {
							str_val = str_val.as_str()[2..].to_string();
							16
						}
						"08" => {
							str_val = str_val.as_str()[2..].to_string();
							8
						}
						"0b" => {
							str_val = str_val.as_str()[2..].to_string();
							2
						}
						_ => 10,
					}
				} else {
					10
				};

				let x = IntLit::from_str_radix(str_val.as_str(), base);
				self.next();
				match x {
					Ok(val) => Expr::IntLit(IntLit::from(val)),
					Err(e) => {
						self.errors.push(self.error(
							format!("could not parse integer: {}", e.to_string()),
							PIErrorCode::ParseCouldNotParseInt,
							vec![
								("invalid integer".to_owned(), self.tok().span.clone()),
								(
									format!("(hint) this is a base {} integer", base).to_owned(),
									self.tok().span.clone(),
								),
							],
						));
						Expr::Error
					}
				}
			}
			TokenKind::Float => {
				let x = Parser::tok_val(self.program, &self.tok()).parse::<FloatLit>();
				self.next();
				match x {
					Ok(val) => Expr::FloatLit(FloatLit::from(val)),
					_ => Expr::Error,
				}
			}
			TokenKind::CharLit => {
				let x = Parser::tok_val(self.program, &self.tok());
				self.next();
				match x.chars().nth(0) {
					Some(val) => Expr::CharLit(CharLit::from(val)),
					_ => Expr::Error,
				}
			}
			TokenKind::StringLit => {
				let x = Parser::tok_val(self.program, &self.tok());
				self.next();
				Expr::StringLit(StringLit::from(x))
			}
			_ => {
				self.next();
				Expr::Error
			}
		}
	}

	pub fn ident(&mut self) -> Ident {
		let x = SmolStr::from(Parser::tok_val(
			self.program,
			self.expect(
				TokenKind::Ident,
				self.error(
					"expected identifier".to_owned(),
					PIErrorCode::ParseExpectedIdent,
					vec![("".to_owned(), self.tok().span.clone())],
				),
			),
		));
		self.next();
		return x;
	}

	fn struct_type_expr(&mut self) -> Expr {
		self.expect(
			TokenKind::Struct,
			self.error(
				"expected `struct` in struct type expression".to_owned(),
				PIErrorCode::ParseExpectedStructInStructTypeExpr,
				vec![],
			),
		);
		if self.tok().kind == TokenKind::Struct {
			self.next();
		}

		self.expect(
			TokenKind::LBrace,
			self.error(
				"expected `{` in struct type expression".to_owned(),
				PIErrorCode::ParseExpectedLBraceInStructTypeExpr,
				vec![],
			),
		);
		if self.tok().kind == TokenKind::LBrace {
			self.next();
		}

		let fields = self.field_list();

		self.expect(
			TokenKind::RBrace,
			self.error(
				"expected `{` in struct type expression".to_owned(),
				PIErrorCode::ParseExpectedRBraceInStructTypeExpr,
				vec![],
			),
		);
		if self.tok().kind == TokenKind::RBrace {
			self.next();
		}

		return Expr::StructType(fields);
	}

	fn field_list(&mut self) -> Vec<Field> {
		let mut fields = vec![];
		while self.tok().kind != TokenKind::RBrace {
			let field = self.field();
			fields.push(field);
		}
		fields
	}

	fn field(&mut self) -> Field {
		let mut pub_ = false;
		if self.tok().kind == TokenKind::Pub {
			pub_ = true;
			self.next();
		}
		let type_ = self.type_expr();

		self.expect(
			TokenKind::Ident,
			self.error(
				"expected identifier in field".to_owned(),
				PIErrorCode::ParseExpectedIdentInField,
				vec![(
					"(hint) give your field a name".to_owned(),
					self.tok().span.clone(),
				)],
			),
		);
		let mut name = String::new();
		if self.tok().kind == TokenKind::Ident {
			name = Parser::tok_val(self.program, &self.tok());
			self.next();
		}

		if self.tok().kind == TokenKind::Semicolon {
			self.next();
			return Field::new(pub_, type_, Ident::from(name), None);
		}

		self.expect(
			TokenKind::Eq,
			self.error(
				"expected `=` in field".to_owned(),
				PIErrorCode::ParseExpectedEqInField,
				vec![(
					"(hint) either terminate your field with a `;` or give it a default value".to_owned(),
					self.tok().span.clone(),
				)],
			),
		);
		if self.tok().kind == TokenKind::Eq {
			self.next();
		}

		let val = self.expr();

		self.expect(
			TokenKind::Semicolon,
			self.error(
				"expected `;` at end of field".to_owned(),
				PIErrorCode::ParseExpectedSemicolonInField,
				vec![(
					"(hint) insert a `;` here".to_owned(),
					self.tok().span.clone(),
				)],
			),
		);
		if self.tok().kind == TokenKind::Semicolon {
			self.next();
		}
		return Field::new(pub_, type_, Ident::from(name.as_str()), Some(val));
	}

	fn interface_type_expr(&mut self) -> Expr {
		self.expect(
			TokenKind::Interface,
			self.error(
				"expected `interface` in interface type expression".to_owned(),
				PIErrorCode::ParseExpectedInterfaceInInterfaceTypeExpr,
				vec![],
			),
		);
		if self.tok().kind == TokenKind::Interface {
			self.next();
		}

		self.expect(
			TokenKind::LBrace,
			self.error(
				"expected `{` in interface type expression".to_owned(),
				PIErrorCode::ParseExpectedLBraceInInterfaceTypeExpr,
				vec![],
			),
		);
		if self.tok().kind == TokenKind::LBrace {
			self.next();
		}

		let methods = self.method_list();

		self.expect(
			TokenKind::RBrace,
			self.error(
				"expected `{` in interface type expression".to_owned(),
				PIErrorCode::ParseExpectedRBraceInInterfaceTypeExpr,
				vec![],
			),
		);
		if self.tok().kind == TokenKind::RBrace {
			self.next();
		}

		return Expr::InterfaceType(methods);
	}

	fn method_list(&mut self) -> Vec<Method> {
		let mut methods = vec![];
		while self.tok().kind != TokenKind::RBrace {
			let method = self.method();
			methods.push(method);
		}
		methods
	}

	fn method(&mut self) -> Method {
		let mut pub_ = false;
		if self.tok().kind == TokenKind::Pub {
			pub_ = true;
			self.next();
		}

		let name = self.ident();
		let params = self.params();
		let mut ret_ty = Expr::PrimitiveType(PrimitiveType::new(PrimitiveKind::Void));
		if self.tok().kind == TokenKind::Arrow {
			self.next();
			ret_ty = self.type_expr();
		}

		self.expect(
			TokenKind::Semicolon,
			self.error(
				"expected `;` after method in interface type expression method list".to_owned(),
				PIErrorCode::ParseExpectedSemicolonAfterMethodInInterfaceTypeMethodList,
				vec![],
			),
		);
		if self.tok().kind == TokenKind::Semicolon {
			self.next();
		}

		return Method::new(pub_, name, params, ret_ty);
	}

	pub fn type_expr(&mut self) -> Expr {
		match self.tok().kind {
			TokenKind::I64
			| TokenKind::U64
			| TokenKind::I32
			| TokenKind::U32
			| TokenKind::I16
			| TokenKind::U16
			| TokenKind::I8
			| TokenKind::U8
			| TokenKind::F64
			| TokenKind::F32
			| TokenKind::Bool => {
				let y = self.token_kind_to_primitive_kind(self.tok().kind.clone());
				self.next();
				let x = Expr::PrimitiveType(PrimitiveType::new(y));
				return x;
			}
			TokenKind::Struct => self.struct_type_expr(),
			TokenKind::Interface => self.interface_type_expr(),
			TokenKind::Ident => Expr::Ident(self.ident()),
			_ => {
				self.errors.push(self.error(
					"expected type expression".to_owned(),
					PIErrorCode::ParseExpectedTypeExpr,
					vec![(
						format!(
							"expected type expression, instead got `{}`",
							Parser::tok_val(self.program, self.tok())
						),
						self.tok().span.clone(),
					)],
				));
				Expr::Error
			}
		}
	}

	fn token_kind_to_primitive_kind(&mut self, tok_kind: TokenKind) -> PrimitiveKind {
		match tok_kind {
			TokenKind::I64 => PrimitiveKind::I64,
			TokenKind::U64 => PrimitiveKind::U64,
			TokenKind::I32 => PrimitiveKind::I32,
			TokenKind::U32 => PrimitiveKind::U32,
			TokenKind::I16 => PrimitiveKind::I16,
			TokenKind::U16 => PrimitiveKind::U16,
			TokenKind::I8 => PrimitiveKind::I8,
			TokenKind::U8 => PrimitiveKind::U8,
			TokenKind::F64 => PrimitiveKind::F64,
			TokenKind::F32 => PrimitiveKind::F32,
			TokenKind::Bool => PrimitiveKind::Bool,
			_ => {
				self.fatal_error(self.error(
					format!(
						"internal compiler error: could not convert token kind `{}` to a primitive type kind",
						tok_kind
					),
					PIErrorCode::ParseCouldNotConvertTokKindToPrimitiveKind,
					vec![],
				));
				return PrimitiveKind::Void;
			}
		}
	}
}
