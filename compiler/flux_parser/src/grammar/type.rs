use flux_lexer::TokenKind;
use flux_syntax::SyntaxKind;

use crate::{marker::CompletedMarker, parser::Parser, token_set::TokenSet};

use super::{expr, generic_args::opt_generic_arg_list, path};

const TYPE_RECOVERY_SET: TokenSet = TokenSet::new(&[TokenKind::RParen, TokenKind::Comma]);

pub(crate) fn type_(p: &mut Parser) {
    let m = if p.at(TokenKind::LParen) {
        tuple_type(p)
    } else if p.at(TokenKind::Ident) {
        path_type(p)
    } else if p.at(TokenKind::LSquare) {
        array_type(p)
    } else {
        return p.err_recover("expected type", TYPE_RECOVERY_SET);
    };
    while p.at(TokenKind::Star) {
        let m = m.clone().precede(p);
        p.bump(TokenKind::Star);
        m.complete(p, SyntaxKind::PtrType);
    }
}

fn tuple_type(p: &mut Parser) -> CompletedMarker {
    let m = p.start();
    p.bump(TokenKind::LParen);
    while p.loop_safe_not_at(TokenKind::RParen) {
        type_(p);
        p.eat(TokenKind::Comma);
    }
    p.expect(TokenKind::RParen);
    m.complete(p, SyntaxKind::TupleType)
}

fn path_type(p: &mut Parser) -> CompletedMarker {
    let m = p.start();
    path(p);
    m.complete(p, SyntaxKind::PathType)
}

fn array_type(p: &mut Parser) -> CompletedMarker {
    let m = p.start();
    p.bump(TokenKind::LSquare);
    type_(p);
    if p.eat(TokenKind::SemiColon) {
        expr::atom::int_expr(p);
    }
    p.expect(TokenKind::RSquare);
    m.complete(p, SyntaxKind::ArrayType)
}
