use crate::{lexer::TokenKind, parser::Parser, syntax::SyntaxKind};

use super::r#type::type_;

// use super::r#type::type_;

pub(super) fn opt_generic_arg_list(p: &mut Parser) {
    if !p.at(TokenKind::CmpLt) {
        return;
    }
    let m = p.start();
    p.bump(TokenKind::CmpLt);
    while p.loop_safe_not_at(TokenKind::CmpGt) {
        generic_arg(p);
        if !p.at(TokenKind::CmpGt) && !p.expect(TokenKind::Comma, "generic argument list") {
            break;
        }
    }
    p.expect(TokenKind::CmpGt, "generic argument list");
    m.complete(p, SyntaxKind::GenericArgList);
}

fn generic_arg(p: &mut Parser) {
    type_(p, "generic argument");
}
