use std::ops::Range;

use pi_ast::{FnDecl, OpKind};
use pi_error::{filesystem::FileId, PIError, PIErrorCode};
use pi_lexer::token::{Token, TokenKind};

mod decl;
mod expr;
mod stmt;

struct Parser<'a> {
	program: &'a str,
	tokens: Vec<Token>,
	offset: usize,
	file_id: FileId,
	errors: &'a mut Vec<PIError>,
}

impl<'a> Parser<'a> {
	pub fn new(
		program: &'a str,
		tokens: Vec<Token>,
		file_id: FileId,
		errors: &'a mut Vec<PIError>,
	) -> Parser<'a> {
		Parser {
			program,
			tokens,
			offset: 0,
			file_id,
			errors,
		}
	}

	pub fn error(
		&self,
		msg: String,
		code: PIErrorCode,
		labels: Vec<(String, Range<usize>)>,
	) -> PIError {
		PIError::new(msg, code, labels, self.file_id)
	}

	pub fn fatal_error(&mut self, error: PIError) {
		self.errors.push(error);
		self.offset = self.tokens.len() - 1;
	}

	pub fn tok_val(program: &str, tok: &Token) -> String {
		String::from(&program[tok.span.clone()])
	}

	pub fn get_tokprec(kind: &TokenKind) -> i8 {
		match kind {
			TokenKind::Eq => 2,
			TokenKind::CmpAnd => 3,
			TokenKind::CmpOr => 5,
			TokenKind::CmpLT => 10,
			TokenKind::CmpGT => 10,
			TokenKind::CmpLTE => 10,
			TokenKind::CmpGTE => 10,
			TokenKind::CmpEQ => 10,
			TokenKind::CmpNE => 10,
			TokenKind::Plus => 20,
			TokenKind::Minus => 20,
			TokenKind::Asterisk => 40,
			TokenKind::Slash => 40,
			TokenKind::Period => 50,
			TokenKind::Arrow => 50,
			_ => -1,
		}
	}

	pub fn next(&mut self) -> &Token {
		self.offset += 1;
		if self.offset == self.tokens.len() {
			self.fatal_error(self.error(
				"unexpected EOF".to_owned(),
				PIErrorCode::ParseUnexpectedEOF,
				vec![],
			))
		}
		self.tok()
	}

	pub fn tok(&self) -> &Token {
		&self.tokens[self.offset]
	}

	pub fn expect(&mut self, kind: TokenKind, error: PIError) -> &Token {
		if self.tok().kind != kind {
			self.errors.push(error);
		}
		self.tok()
	}

	pub fn expect_range(&mut self, begin: TokenKind, end: TokenKind, error: PIError) -> &Token {
		if self.tok().kind <= begin && self.tok().kind >= end {
			self.errors.push(error);
		}
		self.tok()
	}

	fn token_kind_to_op_kind(&self, kind: &TokenKind) -> OpKind {
		match kind {
			TokenKind::Plus => OpKind::Plus,
			TokenKind::Minus => OpKind::Minus,
			TokenKind::Asterisk => OpKind::Asterisk,
			TokenKind::Slash => OpKind::Slash,
			TokenKind::CmpEQ => OpKind::CmpEQ,
			TokenKind::CmpNE => OpKind::CmpNE,
			TokenKind::CmpLT => OpKind::CmpLT,
			TokenKind::CmpLTE => OpKind::CmpLTE,
			TokenKind::CmpGT => OpKind::CmpGT,
			TokenKind::CmpGTE => OpKind::CmpGTE,
			_ => OpKind::Illegal,
		}
	}

	pub fn top_level_decls(&mut self) -> Vec<FnDecl> {
		let mut fn_decls = vec![];

		while self.tok().kind != TokenKind::EOF {
			match self.tok().kind {
				TokenKind::Fn => fn_decls.push(self.fn_decl()),
				_ => {
					self.errors.push(self.error(
						"expected top level declaration".to_owned(),
						PIErrorCode::ParseExpectedTopLevelDecl,
						vec![
							(
								format!(
									"expected top level declaration, instead got `{}`",
									Parser::tok_val(self.program, self.tok())
								),
								self.tok().span.clone(),
							),
							(
								"(hint) declare a function, type or global variable".to_owned(),
								self.tok().span.clone(),
							),
						],
					));
					self.next();
				}
			}
		}
		fn_decls
	}
}

pub fn parse_tokens(
	program: &str,
	tokens: Vec<Token>,
	file_id: FileId,
) -> (Vec<FnDecl>, Vec<PIError>) {
	let mut errors = vec![];
	let mut parse = Parser::new(program, tokens, file_id, &mut errors);
	let fns = parse.top_level_decls();

	(fns, errors)
}
