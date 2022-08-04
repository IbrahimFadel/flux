use cstree::TextRange;
use flux_lexer::{Token, TokenKind};
use flux_span::{Span, Spanned};
use flux_syntax::syntax_kind::SyntaxKind;

use crate::{errors::ParseError, event::Event, grammar::top_level_decl, source::Source};

use self::marker::Marker;

pub(crate) mod marker;

pub struct Parser<'t, 'src> {
	pub(crate) source: Source<'t, 'src>,
	pub events: Vec<Event>,
	expected_kinds: Vec<TokenKind>,
}

impl<'t, 'src> Parser<'t, 'src> {
	pub fn new(source: Source<'t, 'src>) -> Self {
		Self {
			source,
			events: vec![],
			expected_kinds: vec![],
		}
	}

	pub(crate) fn parse(mut self) -> Vec<Event> {
		let m = self.start();
		while !self.at_end() {
			top_level_decl(&mut self);
		}
		m.complete(&mut self, SyntaxKind::Root);
		self.events
	}

	pub(crate) fn start(&mut self) -> Marker {
		let pos = self.events.len();
		self.events.push(Event::Placeholder);
		Marker::new(pos)
	}

	pub(crate) fn expect(&mut self, kind: TokenKind, recovery_set: &[TokenKind]) {
		if self.at(kind) {
			self.bump();
		} else {
			let range = self.peek_range();
			let got = if let Some(tok) = self.peek() {
				tok
			} else {
				TokenKind::Error
			};
			self.error(ParseError::UnexpectedToken {
				expected: self.expected_kinds.clone(),
				got: Spanned::new(got, Span::new(range, self.source.file_id.clone())),
			});

			while !self.at_set(recovery_set) && !self.at_end() && !self.at(TokenKind::Error) {
				self.bump();
			}
		}
	}

	pub(crate) fn peek_range(&mut self) -> TextRange {
		let current_token = self.source.peek_token();
		if let Some(Token { range, .. }) = current_token {
			*range
		} else {
			self.source.last_token_range().unwrap()
		}
	}

	pub(crate) fn cur_span(&mut self) -> Span {
		Span::new(self.peek_range(), self.source.file_id.clone())
	}

	pub(crate) fn error(&mut self, error: ParseError) {
		self.events.push(Event::Error(error));
		// if !self.at_end() {
		// 	let m = self.start();
		// 	self.bump();
		// 	m.complete(self, SyntaxKind::Error);
		// }
	}

	pub(crate) fn expected(&mut self, expected: String) {
		let span = self.cur_span();
		self.error(ParseError::Unxpected {
			expected: Spanned::new(expected, span),
		});
	}

	pub(crate) fn at(&mut self, kind: TokenKind) -> bool {
		self.expected_kinds.push(kind);
		self.peek() == Some(kind)
	}

	pub(crate) fn loop_safe_not_at(&mut self, kind: TokenKind) -> bool {
		self.expected_kinds.push(kind);
		!self.at(kind) && !self.at(TokenKind::Error) && !self.at_end()
	}

	pub(crate) fn next_at(&mut self, kind: TokenKind) -> bool {
		self.expected_kinds.push(kind);
		self.peek_next() == Some(kind)
	}

	pub(crate) fn at_set(&mut self, set: &[TokenKind]) -> bool {
		self.peek().map_or(false, |k| set.contains(&k))
	}

	pub(crate) fn at_end(&mut self) -> bool {
		self.peek().is_none()
	}

	pub(crate) fn bump(&mut self) {
		self.expected_kinds.clear();
		self.source.next_token().unwrap();
		self.events.push(Event::AddToken);
	}

	pub(crate) fn peek(&mut self) -> Option<TokenKind> {
		self.source.peek_kind()
	}

	pub(crate) fn peek_next(&mut self) -> Option<TokenKind> {
		self.source.peek_next_kind()
	}
}
