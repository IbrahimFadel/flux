use crate::lexer::{Token, TokenKind};

pub(crate) struct Source<'a, 'src> {
    tokens: &'a [Token<'src>],
    idx: usize,
}

impl<'a, 'src> Source<'a, 'src> {
    pub fn new(tokens: &'a [Token<'src>]) -> Self {
        Self { tokens, idx: 0 }
    }

    pub(super) fn next_token(&mut self) -> Option<&'a Token<'src>> {
        self.eat_trivia();
        self.idx += 1;
        self.tokens.get(self.idx)
    }

    pub(super) fn peek_kind(&mut self) -> Option<TokenKind> {
        self.eat_trivia();
        self.peek_kind_raw()
    }

    pub(super) fn peek_token(&mut self) -> Option<&'a Token<'src>> {
        self.eat_trivia();
        self.peek_token_raw()
    }

    fn eat_trivia(&mut self) {
        while self.at_trivia() {
            self.idx += 1;
        }
    }

    fn at_trivia(&self) -> bool {
        self.peek_kind_raw().map_or(false, TokenKind::is_trivia)
    }

    fn peek_kind_raw(&self) -> Option<TokenKind> {
        self.tokens.get(self.idx).map(|tok| tok.kind)
    }

    fn peek_token_raw(&self) -> Option<&'a Token<'src>> {
        self.tokens.get(self.idx)
    }
}
