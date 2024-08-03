//! Shamelessly yanked from rust-analyzer: <https://github.com/rust-lang/rust-analyzer/blob/master/crates/parser/src/token_set.rs>

use crate::lexer::TokenKind;

#[derive(Debug, Clone, Copy)]
pub(crate) struct TokenSet(u128);

impl TokenSet {
    pub(crate) const EMPTY: TokenSet = TokenSet(0);
    pub(crate) const TYPE_BEGIN: TokenSet = TokenSet::new(&[
        TokenKind::LParen,
        TokenKind::Ident,
        TokenKind::This,
        TokenKind::LSquare,
    ]);

    pub(crate) const fn new(kinds: &[TokenKind]) -> TokenSet {
        let mut res = 0u128;
        let mut i = 0;
        while i < kinds.len() {
            res |= mask(kinds[i]);
            i += 1;
        }
        TokenSet(res)
    }

    // pub(crate) const fn union(self, other: TokenSet) -> TokenSet {
    //     TokenSet(self.0 | other.0)
    // }

    pub(crate) const fn contains(&self, kind: TokenKind) -> bool {
        self.0 & mask(kind) != 0
    }
}

const fn mask(kind: TokenKind) -> u128 {
    1u128 << (kind as usize)
}

#[cfg(test)]
mod tests {
    use crate::{lexer::TokenKind, token_set::TokenSet};

    #[test]
    fn token_set_works_for_tokens() {
        let ts = TokenSet::new(&[TokenKind::Fn, TokenKind::Minus]);
        assert!(ts.contains(TokenKind::Fn));
        assert!(ts.contains(TokenKind::Minus));
        assert!(!ts.contains(TokenKind::Whitespace));
    }

    // #[test]
    // fn token_set_works_for_unions() {
    //     let ts_a = TokenSet::new(&[TokenKind::Fn, TokenKind::Minus]);
    //     let ts_b = TokenSet::new(&[TokenKind::Period, TokenKind::Mod]);
    //     let ts = ts_a.union(ts_b);
    //     assert!(ts.contains(TokenKind::Fn));
    //     assert!(ts.contains(TokenKind::Minus));
    //     assert!(ts.contains(TokenKind::Period));
    //     assert!(ts.contains(TokenKind::Mod));
    //     assert!(!ts.contains(TokenKind::Whitespace));
    // }
}
