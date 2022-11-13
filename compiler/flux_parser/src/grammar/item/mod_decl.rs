use crate::{grammar::name, marker::CompletedMarker};

use super::*;

pub(crate) fn mod_decl(p: &mut Parser, visibility: CompletedMarker) {
    let m = visibility.precede(p);
    p.bump(TokenKind::Mod);
    name(p, TokenSet::new(&[TokenKind::SemiColon]));
    p.expect(TokenKind::SemiColon);
    m.complete(p, SyntaxKind::ModDecl);
}
