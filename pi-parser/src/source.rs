use pi_error::filesystem::FileId;
use pi_lexer::{Token, TokenKind};
use text_size::TextRange;

pub(super) struct Source<'t, 'src> {
	tokens: &'t [Token<'src>],
	cursor: usize,
	file_id: FileId,
}

impl<'t, 'src> Source<'t, 'src> {
	pub(super) fn new(tokens: &'t [Token<'src>], file_id: FileId) -> Self {
		Self {
			tokens,
			cursor: 0,
			file_id,
		}
	}

	pub(super) fn next_token(&mut self) -> Option<&'t Token<'src>> {
		self.eat_trivia();

		let token = self.tokens.get(self.cursor)?;
		self.cursor += 1;

		Some(token)
	}

	pub(crate) fn peek_token(&mut self) -> Option<&Token> {
		self.eat_trivia();
		self.peek_token_raw()
	}

	pub(super) fn peek_kind(&mut self) -> Option<TokenKind> {
		self.eat_trivia();
		self.peek_kind_raw()
	}

	fn eat_trivia(&mut self) {
		while self.at_trivia() {
			self.cursor += 1;
		}
	}

	fn at_trivia(&self) -> bool {
		self.peek_kind_raw().map_or(false, TokenKind::is_trivia)
	}

	fn peek_token_raw(&self) -> Option<&Token> {
		self.tokens.get(self.cursor)
	}

	fn peek_kind_raw(&self) -> Option<TokenKind> {
		self
			.tokens
			.get(self.cursor)
			.map(|Token { kind, .. }| (*kind).into())
	}

	pub(crate) fn last_token_range(&self) -> Option<TextRange> {
		self.tokens.last().map(|Token { range, .. }| *range)
	}

	pub(crate) fn file_id(&self) -> FileId {
		self.file_id
	}
}
