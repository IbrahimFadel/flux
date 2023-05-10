use flux_lexer::TokenKind;
use flux_syntax::SyntaxKind;

use crate::{marker::CompletedMarker, parser::Parser};

use super::{expr, path, PathType};

pub(crate) fn poisoned_type(p: &mut Parser, parent: &str) {
    let m = p.start();
    p.expected("type", parent);
    m.complete(p, SyntaxKind::TupleType);
}

pub(crate) fn type_(p: &mut Parser, parent: &str) {
    let m = match p.peek() {
        TokenKind::LParen => tuple_type(p),
        TokenKind::Ident | TokenKind::This => path_type(p),
        TokenKind::LSquare => array_type(p),
        _ => return p.expected("type", parent),
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
        type_(p, "tuple type");
        p.eat(TokenKind::Comma);
    }
    p.expect(TokenKind::RParen, "tuple type");
    m.complete(p, SyntaxKind::TupleType)
}

fn path_type(p: &mut Parser) -> CompletedMarker {
    let m = p.start();
    let path_type = path(p);
    match path_type {
        PathType::Regular => m.complete(p, SyntaxKind::PathType),
        PathType::This => m.complete(p, SyntaxKind::ThisPathType),
    }
}

fn array_type(p: &mut Parser) -> CompletedMarker {
    let m = p.start();
    p.bump(TokenKind::LSquare);
    type_(p, "array type");
    if p.eat(TokenKind::SemiColon) {
        expr::atom::int_expr(p);
    }
    p.expect(TokenKind::RSquare, "array type");
    m.complete(p, SyntaxKind::ArrayType)
}
