use flux_lexer::TokenKind;
use flux_syntax::SyntaxKind;

use crate::parser::Parser;

use super::r#type::type_;

pub(super) fn opt_generic_arg_list(p: &mut Parser) {
    if !p.at(TokenKind::CmpLt) {
        return;
    }
    let m = p.start();
    p.bump(TokenKind::CmpLt);
    while p.loop_safe_not_at(TokenKind::CmpGt) {
        generic_arg(p);
        if !p.at(TokenKind::CmpGt) && !p.expect(TokenKind::Comma) {
            break;
        }
    }
    p.expect(TokenKind::CmpGt);
    m.complete(p, SyntaxKind::GenericArgList);
}

fn generic_arg(p: &mut Parser) {
    type_(p);
}
