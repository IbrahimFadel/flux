use flux_error::{PIError, PIErrorCode, Span};
use flux_lexer::{Token, TokenKind};
use flux_syntax::syntax_kind::SyntaxKind;

use crate::{event::Event, grammar::top_level_decl, source::Source};

use self::marker::Marker;

pub(crate) mod marker;

const RECOVERY_SET: [TokenKind; 7] = [
	TokenKind::INKw,
	TokenKind::FnKw,
	TokenKind::LBrace,
	TokenKind::RBrace,
	TokenKind::LParen,
	TokenKind::RParen,
	TokenKind::SemiColon,
];

pub(crate) struct Parser<'t, 'src> {
	source: Source<'t, 'src>,
	pub(crate) events: Vec<Event>,
	expected_kinds: Vec<TokenKind>,
}

impl<'t, 'src> Parser<'t, 'src> {
	pub(crate) fn new(source: Source<'t, 'src>) -> Self {
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

	pub(crate) fn expect(&mut self, kind: TokenKind, msg: String) {
		if self.at(kind) {
			self.bump();
		} else {
			self.error(msg);
		}
	}

	pub(crate) fn error(&mut self, msg: String) {
		let current_token = self.source.peek_token();

		let range = if let Some(Token { range, .. }) = current_token {
			*range
		} else {
			self.source.last_token_range().unwrap()
		};

		self.events.push(Event::Error(
			PIError::default()
				.with_msg(msg.clone())
				.with_code(PIErrorCode::UnexpectedToken)
				.with_label(msg, Some(Span::new(range, self.source.file_id()))),
		));

		if !self.at_set(&RECOVERY_SET) && !self.at_end() {
			let m = self.start();
			self.bump();
			m.complete(self, SyntaxKind::Error);
		}
	}

	pub(crate) fn at(&mut self, kind: TokenKind) -> bool {
		self.expected_kinds.push(kind);
		self.peek() == Some(kind)
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
