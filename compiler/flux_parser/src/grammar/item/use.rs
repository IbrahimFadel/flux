use crate::{grammar::name, marker::CompletedMarker, token_set::TokenSet};

use super::*;

pub(crate) fn decl(p: &mut Parser, visibility: CompletedMarker) {
    let m = visibility.precede(p);
    p.bump(TokenKind::Use);

    let path_m = p.start();
    p.expect(TokenKind::Ident, "use declaration");
    while p.at(TokenKind::DoubleColon) {
        p.bump(TokenKind::DoubleColon);
        if !p.expect(TokenKind::Ident, "use path") {
            break;
        }
    }
    path_m.complete(p, SyntaxKind::Path);

    if p.eat(TokenKind::As) {
        name(
            p,
            TokenSet::new(&[TokenKind::SemiColon]),
            "use declaration alias",
        );
    }
    p.expect(TokenKind::SemiColon, "use declaration");
    m.complete(p, SyntaxKind::UseDecl);
}
