use std::ops::Range;

use pi_ast::{FnDecl, Mod, OpKind, TypeDecl, AST};
use pi_error::{filesystem::FileId, PIError, PIErrorCode};
use pi_lexer::token::{Token, TokenKind};

mod decl;
mod expr;
mod stmt;
mod tests;

use decl::{fn_decl, type_decl};
use stmt::mod_stmt;

pub struct ParseInput<'a> {
	program: String,
	toks: &'a [Token],
	errs: Vec<PIError>,
	offset: usize,
	file_id: FileId,
	typenames: Vec<String>,
}

impl<'a> ParseInput<'a> {
	pub fn tok(&self) -> &Token {
		&self.toks[self.offset]
	}

	pub fn next(&mut self) -> &Token {
		self.offset += 1;
		if self.offset == self.toks.len() {
			self.fatal_error(
				"unexpected EOF".to_owned(),
				PIErrorCode::ParseUnexpectedEOF,
				vec![],
			);
		}
		self.tok()
	}

	pub fn error(
		&self,
		msg: String,
		code: PIErrorCode,
		labels: Vec<(String, Range<usize>)>,
	) -> PIError {
		PIError::new(msg, code, labels, self.file_id)
	}

	pub fn fatal_error(
		&mut self,
		msg: String,
		code: PIErrorCode,
		labels: Vec<(String, Range<usize>)>,
	) {
		self.errs.push(self.error(msg, code, labels));
		self.offset = self.toks.len() - 1;
	}

	pub fn expect(&mut self, kind: TokenKind, error: PIError) -> &Token {
		if self.tok().kind != kind {
			self.errs.push(error);
		}
		self.tok()
	}

	pub fn expect_range(&mut self, begin: TokenKind, end: TokenKind, error: PIError) -> &Token {
		if self.tok().kind <= begin && self.tok().kind >= end {
			self.errs.push(error);
		}
		self.tok()
	}
}

fn tok_val(program: &String, tok: &Token) -> String {
	String::from(&program[tok.span.clone()])
}

fn get_tokprec(kind: &TokenKind) -> i8 {
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
		TokenKind::DoubleColon => 50,
		_ => -1,
	}
}

fn token_kind_to_op_kind(kind: &TokenKind) -> OpKind {
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
		TokenKind::DoubleColon => OpKind::Doublecolon,
		TokenKind::Period => OpKind::Period,
		TokenKind::Eq => OpKind::Eq,
		_ => OpKind::Illegal,
	}
}

pub fn top_level_decls(input: &mut ParseInput) -> (Vec<Mod>, Vec<FnDecl>, Vec<TypeDecl>) {
	let mut fn_decls = vec![];
	let mut type_decls = vec![];
	let mut mod_stmts = vec![];

	while input.tok().kind != TokenKind::EOF {
		match input.tok().kind {
			TokenKind::Pub => {
				input.next();
				match input.tok().kind {
					TokenKind::Fn => fn_decls.push(fn_decl(input, true)),
					TokenKind::Type => type_decls.push(type_decl(input, true)),
					TokenKind::Mod => mod_stmts.push(mod_stmt(input, true)),
					_ => {
						input.errs.push(input.error(
							"expected declaration following `pub`".to_owned(),
							PIErrorCode::ParseExpectedTopLevelDecl,
							vec![
								(
									format!(
										"expected declaration following `pub`, instead got `{}`",
										tok_val(&input.program, input.tok())
									),
									input.tok().span.clone(),
								),
								(
									"(hint) declare a function, type or global variable".to_owned(),
									input.tok().span.clone(),
								),
							],
						));
						input.next();
					}
				}
			}
			TokenKind::Fn => fn_decls.push(fn_decl(input, false)),
			TokenKind::Type => type_decls.push(type_decl(input, false)),
			TokenKind::Mod => mod_stmts.push(mod_stmt(input, false)),
			_ => {
				input.errs.push(input.error(
					"expected top level declaration".to_owned(),
					PIErrorCode::ParseExpectedTopLevelDecl,
					vec![
						(
							format!(
								"expected top level declaration, instead got `{}`",
								tok_val(&input.program, input.tok())
							),
							input.tok().span.clone(),
						),
						(
							"(hint) declare a function, type or global variable".to_owned(),
							input.tok().span.clone(),
						),
					],
				));
				input.next();
			}
		}
	}
	(mod_stmts, fn_decls, type_decls)
}

pub fn parse_tokens(
	name: String,
	program: String,
	tokens: Vec<Token>,
	file_id: FileId,
) -> (AST, Vec<PIError>) {
	let mut initial_input = ParseInput {
		program,
		file_id,
		toks: &tokens,
		errs: vec![],
		offset: 0,
		typenames: vec![],
	};
	let (mods, functions, types) = top_level_decls(&mut initial_input);

	return (AST::new(name, mods, functions, types), initial_input.errs);
}
