use pi_error::{filesystem, PIError, PIErrorCode, Span};
use std::ops::Range;
use token::{lookup_keyword, Token, TokenKind};

mod tests;
pub mod token;

struct Lexer<'a> {
	offset: usize,
	program: &'a str,
	file_id: filesystem::FileId,
	errors: &'a mut Vec<PIError>,
}

impl<'a> Lexer<'a> {
	pub fn new(
		program: &'a str,
		file_id: filesystem::FileId,
		errors: &'a mut Vec<PIError>,
	) -> Lexer<'a> {
		Lexer {
			program,
			offset: 0,
			file_id,
			errors,
		}
	}

	fn ch(&self) -> char {
		match self.program.chars().nth(self.offset) {
			Some(x) => x,
			_ => '\0',
		}
	}

	fn peek(&self) -> char {
		match self.program.chars().nth(self.offset + 1) {
			Some(x) => x,
			_ => '\0',
		}
	}

	fn next(&mut self) {
		self.offset += 1;
	}

	pub fn token(&mut self) -> Token {
		let mut tok = Token::new();
		self.whitespace();

		let ch = self.ch();
		if ch.is_alphabetic() {
			tok.span = self.ident();
			let s = &self.program[tok.span.clone()];
			if s.len() > 1 {
				tok.kind = lookup_keyword(s);
			} else {
				tok.kind = TokenKind::Ident;
			}
		} else if ch.is_digit(10) || (ch == '.' && self.peek().is_digit(10)) {
			tok = self.number();
		} else {
			self.next();
			let span = self.offset - 1..self.offset;
			match ch {
				' ' | '\n' | '\t' | '\r' => self.next(),
				';' => {
					tok.kind = TokenKind::Semicolon;
					tok.span = span;
				}
				'"' => {
					tok.kind = TokenKind::StringLit;
					tok.span = self.string();
				}
				'\'' => {
					tok.kind = TokenKind::CharLit;
					tok.span = self.char();
				}
				'(' => {
					tok.kind = TokenKind::LParen;
					tok.span = span;
				}
				')' => {
					tok.kind = TokenKind::RParen;
					tok.span = span;
				}
				'{' => {
					tok.kind = TokenKind::LBrace;
					tok.span = span;
				}
				'}' => {
					tok.kind = TokenKind::RBrace;
					tok.span = span;
				}
				'[' => {
					tok.kind = TokenKind::LBracket;
					tok.span = span;
				}
				']' => {
					tok.kind = TokenKind::RBracket;
					tok.span = span;
				}
				'+' => {
					tok.kind = TokenKind::Plus;
					tok.span = span;
				}
				'*' => {
					tok.kind = TokenKind::Asterisk;
					tok.span = span;
				}
				':' => {
					tok.kind = TokenKind::Colon;
					tok.span = span;
					if self.ch() == ':' {
						self.next();
						tok.kind = TokenKind::DoubleColon;
						tok.span = self.offset - 2..self.offset;
					}
				}
				'/' => {
					tok.kind = TokenKind::Slash;
					tok.span = span;
					if self.ch() == '/' {
						self.next();
						tok.kind = TokenKind::LineComment;
						tok.span = self.line_comment();
					} else if self.ch() == '*' {
						self.next();
						tok.kind = TokenKind::BlockComment;
						tok.span = self.block_comment();
					}
				}
				',' => {
					tok.kind = TokenKind::Comma;
					tok.span = span;
				}
				'.' => {
					tok.kind = TokenKind::Period;
					tok.span = span;
					if self.ch() == '.' {
						self.next();
						tok.kind = TokenKind::DoublePeriod;
						tok.span = self.offset - 2..self.offset;
					}
				}
				'!' => {
					if self.ch() == '=' {
						self.next();
						tok.kind = TokenKind::CmpNE;
						tok.span = self.offset - 2..self.offset;
					}
				}
				'=' => {
					tok.kind = TokenKind::Eq;
					tok.span = span;
					if self.ch() == '=' {
						self.next();
						tok.kind = TokenKind::CmpEQ;
						tok.span = self.offset - 2..self.offset;
					}
				}
				'&' => {
					tok.kind = TokenKind::Ampersand;
					tok.span = span;
					if self.ch() == '&' {
						self.next();
						tok.kind = TokenKind::CmpAnd;
						tok.span = self.offset - 2..self.offset;
					}
				}
				'|' => {
					if self.ch() == '|' {
						self.next();
						tok.kind = TokenKind::CmpOr;
						tok.span = self.offset - 2..self.offset;
					}
				}
				'<' => {
					tok.kind = TokenKind::CmpLT;
					tok.span = span;
					if self.ch() == '=' {
						self.next();
						tok.kind = TokenKind::CmpLTE;
						tok.span = self.offset - 2..self.offset;
					}
				}
				'>' => {
					tok.kind = TokenKind::CmpGT;
					tok.span = span;
					if self.ch() == '=' {
						self.next();
						tok.kind = TokenKind::CmpGTE;
						tok.span = self.offset - 2..self.offset;
					}
				}
				'-' => {
					tok.kind = TokenKind::Minus;
					tok.span = span;
					if self.ch() == '>' {
						self.next();
						tok.kind = TokenKind::Arrow;
						tok.span = self.offset - 2..self.offset;
					}
				}
				_ => (),
				// _ => self.errors.push(PIError::new(
				// 	"unknown character".to_owned(),
				// 	PIErrorCode::LexUnknownChar,
				// 	vec![(
				// 		format!("unknown character `{}`", ch),
				// 		self.offset - 1..self.offset - 1,
				// 	)],
				// 	self.file_id,
				// )),
			}
		}

		return tok;
	}

	fn block_comment(&mut self) -> Range<usize> {
		let initial_offset = self.offset - 2;
		loop {
			if self.ch() == '\0' {
				self.errors.push(PIError::new(
					"missing end of block comment".to_owned(),
					PIErrorCode::LexMissingEndOfBlockComment,
					vec![
						(
							"missing end of block comment".to_owned(),
							Span::new(initial_offset..self.offset, self.file_id),
						),
						(
							"(hint) insert `*/`".to_owned(),
							Span::new(initial_offset..self.offset, self.file_id),
						),
					],
				));
				break;
			}
			if self.ch() == '*' && self.peek() == '/' {
				self.next();
				self.next();
				break;
			}
			self.next();
		}
		initial_offset..self.offset
	}

	fn line_comment(&mut self) -> Range<usize> {
		let initial_offset = self.offset - 2;
		while self.ch() != '\n' && self.ch() != '\0' {
			self.next();
		}
		initial_offset..self.offset
	}

	fn char(&mut self) -> Range<usize> {
		let initial_offset = self.offset - 1;
		let mut n = 0;
		loop {
			match self.ch() {
				'\n' | '\0' => {
					self.errors.push(PIError::new(
						"char literal not terminated".to_owned(),
						PIErrorCode::LexCharLitUnterminated,
						vec![
							(
								"".to_owned(),
								Span::new(initial_offset..self.offset, self.file_id),
							),
							(
								"(hint) insert missing `\'`".to_owned(),
								Span::new(initial_offset..self.offset, self.file_id),
							),
						],
					));
					break;
				}
				'\'' => {
					self.next();
					break;
				}
				'\\' => {
					self.escape('\'');
					n += 1;
					continue;
				}
				_ => {
					n += 1;
				}
			}
			self.next();
		}
		if n != 1 {
			self.errors.push(PIError::new(
				format!(
					"invalid char literal `{}`",
					&self.program[initial_offset..self.offset]
				),
				PIErrorCode::LexInvalidCharLit,
				vec![
					(
						format!(
							"invalid char literal `{}`",
							&self.program[initial_offset..self.offset]
						),
						Span::new(initial_offset..self.offset, self.file_id),
					),
					(
						"char literals can only be one character long".to_owned(),
						Span::new(initial_offset..self.offset, self.file_id),
					),
				],
			));
		}

		initial_offset..self.offset
	}

	fn string(&mut self) -> Range<usize> {
		let initial_offset = self.offset - 1;

		loop {
			if self.ch() == '\n' || self.ch() == '\0' {
				self.errors.push(PIError::new(
					"string literal not terminated".to_owned(),
					PIErrorCode::LexStringLitUnterminated,
					vec![(
						"(hint) insert missing '\"'".to_owned(),
						Span::new(initial_offset..self.offset, self.file_id),
					)],
				));
				break;
			}
			if self.ch() == '"' {
				self.next();
				break;
			}
			if self.ch() == '\\' {
				self.escape('"');
				continue;
			}
			self.next();
		}

		initial_offset..self.offset
	}

	fn escape(&mut self, quote: char) -> char {
		let initial_offset = self.offset;
		self.next();
		if self.ch() == quote {
			self.next();
			return self.ch();
		}
		let mut recognized_escapes_str =
			String::from("recognized escape sequences are '\\r', '\\t', '\\n'");
		if quote == '\'' {
			recognized_escapes_str += " and '\\''";
		} else {
			recognized_escapes_str += " and '\"'";
		}
		match self.ch() {
			'r' | 't' | 'n' => {
				self.next();
				return self.ch();
			}
			_ => {
				self.errors.push(PIError::new(
					format!(
						"unknown escape sequence {}",
						&self.program[initial_offset..self.offset + 1]
					),
					PIErrorCode::LexUnknownEscapeSequence,
					vec![(
						recognized_escapes_str,
						Span::new(initial_offset..self.offset + 1, self.file_id),
					)],
				));
				return self.ch();
			}
		}
	}

	fn whitespace(&mut self) {
		loop {
			match self.ch() {
				' ' | '\t' | '\n' | '\r' => self.next(),
				_ => break,
			}
		}
	}

	fn ident(&mut self) -> Range<usize> {
		let initial_offset = self.offset;
		self.next();
		while self.ch().is_alphabetic() || self.ch().is_digit(10) || self.ch() == '_' {
			self.next();
		}
		initial_offset..self.offset
	}

	fn number(&mut self) -> Token {
		let initial_offset = self.offset;
		let mut tok = Token::new();

		let mut base: u8 = 10;

		if self.ch() != '.' {
			tok.kind = TokenKind::Int;
			let prefix_initial_offset = self.offset;
			if self.ch() == '0' {
				self.next();
				match self.ch() {
					'x' => {
						self.next();
						base = 16;
					}
					'8' => {
						self.next();
						base = 8;
					}
					'b' => {
						self.next();
						base = 2;
					}
					_ => base = 10,
				}
			}

			self.digits(base);
			if base != 10 && self.offset - prefix_initial_offset == 2 {
				let prefix = if base == 16 {
					"0x"
				} else if base == 8 {
					"08"
				} else if base == 2 {
					"0b"
				} else {
					""
				};
				let mut got_str = "EOF";
				if self.offset != self.program.len() {
					got_str = &self.program[prefix_initial_offset + 2..self.offset + 1];
				}
				self.errors.push(PIError::new(
					format!("expected base {} digit(s) following {}", base, prefix),
					PIErrorCode::LexExpectedDigitsFollowingIntPrefix,
					vec![(
						format!(
							"expected base {} digit(s) following {}, instead got `{}`",
							base, prefix, got_str
						),
						Span::new(initial_offset..self.offset, self.file_id),
					)],
				))
			}
		}

		if self.ch() == '.' {
			tok.kind = TokenKind::Float;
			if base != 10 {
				self.errors.push(PIError::new(
					"floating point numbers are only permitted in base 10".to_owned(),
					PIErrorCode::LexFloatInWrongBase,
					vec![(
						format!("base {} literal", base),
						Span::new(initial_offset..self.offset, self.file_id),
					)],
				))
			}
			self.next();
			self.digits(base);
		}

		tok.span = initial_offset..self.offset;
		return tok;
	}

	fn digits(&mut self, base: u8) {
		while self.ch().is_digit(base as u32) {
			self.next();
		}
	}
}

pub fn tokenize(program: &str, file_id: filesystem::FileId) -> (Vec<Token>, Vec<PIError>) {
	let mut errors = vec![];
	let mut lex = Lexer::new(program, file_id, &mut errors);
	let mut arr = vec![];
	while lex.ch() != '\0' {
		let res = lex.token();
		arr.push(res);
	}
	arr.push(Token::from(TokenKind::EOF, 0..0));
	(arr, errors)
}
