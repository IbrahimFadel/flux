use cstree::TextRange;
use flux_lexer::{Token, TokenKind};
use flux_span::{FileSpanned, Span, Spanned};
use flux_syntax::SyntaxKind;

use crate::{
    diagnostics::ParserDiagnostic, event::Event, grammar::item::item, marker::Marker,
    source::Source, token_set::TokenSet,
};

pub(super) struct Parser<'a, 'src> {
    source: Source<'a, 'src>,
    pub(crate) events: Vec<Event>,
}

impl<'a, 'src> Parser<'a, 'src> {
    pub(crate) fn new(source: Source<'a, 'src>) -> Self {
        Self {
            source,
            events: vec![],
        }
    }

    pub(crate) fn parse(mut self) -> Vec<Event> {
        let m = self.start();

        while !self.at_end() {
            item(&mut self);
        }

        m.complete(&mut self, SyntaxKind::Root);
        self.events
    }

    pub(crate) fn start(&mut self) -> Marker {
        let pos = self.events.len();
        self.events.push(Event::Placeholder);
        Marker::new(pos)
    }

    pub(crate) fn expect(&mut self, kind: TokenKind) -> bool {
        if self.eat(kind) {
            return true;
        }
        self.error(format!("expected {kind}"));
        false
    }

    pub(crate) fn eat(&mut self, kind: TokenKind) -> bool {
        if !self.at(kind) {
            return false;
        }
        self.do_bump();
        true
    }

    pub(crate) fn at(&mut self, kind: TokenKind) -> bool {
        self.peek() == Some(kind)
    }

    pub(crate) fn at_set(&mut self, set: TokenSet) -> bool {
        self.peek().map_or(false, |k| set.contains(k))
    }

    pub(crate) fn at_end(&mut self) -> bool {
        self.peek().is_none()
    }

    pub(crate) fn error<T: Into<String>>(&mut self, msg: T) {
        let range = self.peek_range().unwrap_or_default();
        let msg = msg.into();
        self.events.push(Event::Error(ParserDiagnostic::Unxpected {
            expected: FileSpanned::new(Spanned::new(msg, Span::new(range)), self.source.file_id),
        }));
    }

    pub(crate) fn err_recover(&mut self, msg: &str, recovery_set: TokenSet) {
        if self.at_set(recovery_set) {
            self.error(msg);
            return;
        }

        let m = self.start();
        self.error(msg);
        self.bump_any();
        m.complete(self, SyntaxKind::Error);
    }

    pub(crate) fn err_and_bump(&mut self, message: &str) {
        self.err_recover(message, TokenSet::EMPTY);
    }

    pub(crate) fn bump(&mut self, kind: TokenKind) {
        assert!(self.eat(kind));
    }

    pub(crate) fn bump_any(&mut self) {
        if self.peek_token().is_some() {
            self.do_bump();
        }
    }

    pub(crate) fn loop_safe_not_at(&mut self, kind: TokenKind) -> bool {
        !(self.at(kind) || self.at_end() || self.at(TokenKind::Error))
    }

    pub(crate) fn current(&mut self) -> TokenKind {
        self.source.peek_kind().unwrap_or(TokenKind::Error)
    }

    fn peek(&mut self) -> Option<TokenKind> {
        self.source.peek_kind()
    }

    fn peek_token(&mut self) -> Option<&'a Token<'src>> {
        self.source.peek_token()
    }

    fn peek_range(&mut self) -> Option<TextRange> {
        self.source.peek_range()
    }

    fn do_bump(&mut self) {
        self.source.next_token(); // Don't unwrap because we could have the EOF next
        self.events.push(Event::AddToken);
    }
}
