use logos::Logos;
use text_size::{TextRange, TextSize};
pub use token_kind::TokenKind;

mod token_kind;

pub struct Lexer<'a> {
    inner: logos::Lexer<'a, TokenKind>,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            inner: TokenKind::lexer(input),
        }
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Token<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let kind = self.inner.next()?;
        let kind = match kind {
            Ok(kind) => kind,
            Err(_) => return None,
        };
        let text = self.inner.slice();

        let range = {
            let std::ops::Range { start, end } = self.inner.span();
            let start = TextSize::try_from(start).unwrap();
            let end = TextSize::try_from(end).unwrap();

            TextRange::new(start, end)
        };

        Some(Self::Item { kind, text, range })
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Token<'a> {
    pub kind: TokenKind,
    pub text: &'a str,
    pub range: TextRange,
}
