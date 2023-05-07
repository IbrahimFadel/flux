use crate::{grammar::name, marker::CompletedMarker, token_set::TokenSet};

use super::*;

pub(crate) fn decl(p: &mut Parser, visibility: CompletedMarker) {
    let m = visibility.precede(p);
    p.bump(TokenKind::Mod);
    name(p, TokenSet::new(&[TokenKind::SemiColon]), "mod declaration");
    p.expect(TokenKind::SemiColon, "mod declaration");
    m.complete(p, SyntaxKind::ModDecl);
}
