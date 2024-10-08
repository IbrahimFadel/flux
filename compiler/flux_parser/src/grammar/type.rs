use crate::{lexer::TokenKind, marker::CompletedMarker, parser::Parser, syntax::SyntaxKind};

use super::{expr, path, PathType};

pub(crate) fn poisoned_type(p: &mut Parser, parent: &str) {
    let m = p.start();
    p.expected("type", parent);
    m.complete(p, SyntaxKind::TupleType);
}

pub(crate) fn type_(p: &mut Parser, parent: &str) {
    let mut m = match p.peek() {
        TokenKind::LParen => tuple_type(p),
        TokenKind::Ident | TokenKind::This => path_type(p),
        TokenKind::LSquare => array_type(p),
        _ => return p.expected("type", parent),
    };
    while p.at(TokenKind::Ampersand) || p.at(TokenKind::CmpAnd) {
        if p.at(TokenKind::Ampersand) {
            let ref_m = m.clone().precede(p);
            p.bump(TokenKind::Ampersand);
            m = ref_m.complete(p, SyntaxKind::RefType);
        } else if p.at(TokenKind::CmpAnd) {
            let ref_m = m.clone().precede(p);
            p.bump(TokenKind::CmpAnd);
            let inner_ref_m = ref_m.complete(p, SyntaxKind::RefType).clone().precede(p);
            m = inner_ref_m.complete(p, SyntaxKind::RefType);
        }
    }
    while p.at(TokenKind::Star) {
        let ptr_m = m.clone().precede(p);
        p.bump(TokenKind::Star);
        m = ptr_m.complete(p, SyntaxKind::PtrType);
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
