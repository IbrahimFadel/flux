use flux_lexer::{Token, TokenKind};
use flux_syntax::SyntaxKind;

use crate::{
    event::Event, grammar::item::item, marker::Marker, source::Source, token_set::TokenSet,
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

    pub(crate) fn expect(&mut self, kind: TokenKind, parent: &str) -> bool {
        if self.eat(kind) {
            return true;
        }
        self.expected(&format!("`{kind}`"), parent);
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
        self.peek() == kind
    }

    pub(crate) fn at_set(&mut self, set: TokenSet) -> bool {
        set.contains(self.peek())
    }

    pub(crate) fn at_end(&mut self) -> bool {
        self.at(TokenKind::EOF)
    }

    pub(crate) fn error<T: Into<String>>(&mut self, msg: T) {
        self.events.push(Event::Error(msg.into()));
    }

    pub(crate) fn expected(&mut self, token: &str, parent: &str) {
        let msg = format!("expected {token} in {parent}");
        self.events.push(Event::Error(msg));
    }

    // pub(crate) fn update_last_error_expected(&mut self, token: &str, parent: &str) {
    //     let msg = format!("expected {token} in {parent}");
    //     let last_error = self
    //         .events
    //         .iter_mut()
    //         .rev()
    //         .find(|event| match event {
    //             Event::Error(_) => true,
    //             _ => false,
    //         })
    //         .unwrap_or_else(|| {
    //             flux_diagnostics::ice("tried updating last error but there are no errors")
    //         });
    //     *last_error = Event::Error(msg);
    // }

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
        !(self.at(kind) || self.at_end())
    }

    pub(crate) fn peek(&mut self) -> TokenKind {
        self.source.peek_kind().unwrap_or(TokenKind::EOF)
    }

    fn peek_token(&mut self) -> Option<&'a Token<'src>> {
        self.source.peek_token()
    }

    fn do_bump(&mut self) {
        self.source.next_token(); // Don't unwrap because we could have the EOF next
        self.events.push(Event::AddToken);
    }

    pub(crate) fn recover_for(&mut self, tokens: TokenSet) -> TokenKind {
        loop {
            let t = self.peek();

            if tokens.contains(t) {
                break t;
            }

            self.bump_any();
        }
    }
}

#[cfg(test)]
mod tests {
    use flux_lexer::{Token, TokenKind};
    use text_size::TextRange;

    use crate::{source::Source, token_set::TokenSet};

    use super::Parser;

    macro_rules! parser_with {
        ($toks:expr) => {
            Parser::new(Source::new($toks))
        };
    }

    macro_rules! tok {
        ($kind:expr) => {
            Token {
                kind: $kind,
                text: "<tok>",
                range: TextRange::new(0.into(), 0.into()),
            }
        };
        ($kind:expr, $text:expr) => {
            Token {
                kind: $kind,
                text: $text,
                range: TextRange::new(0.into(), 0.into()),
            }
        };
    }

    #[test]
    fn peek_skips_trivia() {
        let toks = &[tok!(TokenKind::Whitespace), tok!(TokenKind::Ampersand)];
        let mut p = parser_with!(toks);
        assert_eq!(p.peek(), TokenKind::Ampersand);
    }

    #[test]
    fn basic_functionality() {
        let toks = &[tok!(TokenKind::Ampersand), tok!(TokenKind::Apply)];
        let mut p = parser_with!(toks);
        assert_eq!(p.peek(), TokenKind::Ampersand);
        assert_eq!(p.peek(), TokenKind::Ampersand);
        p.bump(TokenKind::Ampersand);

        assert_eq!(p.eat(TokenKind::Arrow), false);
        assert_eq!(p.peek(), TokenKind::Apply);
        assert_eq!(p.eat(TokenKind::Apply), true);

        assert_eq!(p.peek(), TokenKind::EOF);
    }

    #[test]
    fn gives_eof_token() {
        let mut p = parser_with!(&[]);
        assert_eq!(p.peek(), TokenKind::EOF);
    }

    #[test]
    fn recovery() {
        let toks = &[tok!(TokenKind::LParen)];
        let mut p = parser_with!(toks);
        assert_eq!(p.expect(TokenKind::Ident, "function name"), false);
        let recovered_to = p.recover_for(TokenSet::new(&[TokenKind::LParen]));
        assert_eq!(recovered_to, TokenKind::LParen);

        let toks = &[
            tok!(TokenKind::Ampersand),
            tok!(TokenKind::Ampersand),
            tok!(TokenKind::LParen),
        ];
        let mut p = parser_with!(toks);
        assert_eq!(p.expect(TokenKind::Ident, "function name"), false);
        let recovered_to = p.recover_for(TokenSet::new(&[TokenKind::LParen]));
        assert_eq!(recovered_to, TokenKind::LParen);
    }
}
