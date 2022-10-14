use flux_lexer::TokenKind;
use flux_syntax::SyntaxKind;

use crate::{grammar::generic_args::opt_generic_arg_list, marker::CompletedMarker, parser::Parser};

use super::{expr, stmt, ExprRestrictions};

pub(super) fn atom(p: &mut Parser, restrictions: ExprRestrictions) -> Option<CompletedMarker> {
    let m = if p.at(TokenKind::IntLit) {
        int_expr(p)
    } else if p.at(TokenKind::FloatLit) {
        float_expr(p)
    } else if p.at(TokenKind::StringLit) {
        string_expr(p)
    } else if p.at(TokenKind::LParen) {
        paren_or_tuple_expr(p)
    } else if p.at(TokenKind::LBrace) && restrictions.allow_block_expressions {
        block_expr(p)
    } else if p.at(TokenKind::Ident) {
        path_or_complex_type_expr(p, restrictions)
    } else {
        p.err_and_bump("expected expression atom");
        return None;
    };
    Some(m)
}

pub(crate) fn int_expr(p: &mut Parser) -> CompletedMarker {
    let m = p.start();
    p.bump(TokenKind::IntLit);
    m.complete(p, SyntaxKind::IntExpr)
}

fn float_expr(p: &mut Parser) -> CompletedMarker {
    let m = p.start();
    p.bump(TokenKind::FloatLit);
    m.complete(p, SyntaxKind::FloatExpr)
}

fn string_expr(p: &mut Parser) -> CompletedMarker {
    let m = p.start();
    p.bump(TokenKind::StringLit);
    m.complete(p, SyntaxKind::StringExpr)
}

fn paren_or_tuple_expr(p: &mut Parser) -> CompletedMarker {
    let m = p.start();
    p.bump(TokenKind::LParen);

    let mut comma = false;
    while p.loop_safe_not_at(TokenKind::RParen) {
        if !expr(p) {
            break;
        }
        if !p.at(TokenKind::RParen) {
            comma = true;
            p.expect(TokenKind::Comma);
        }
    }
    p.expect(TokenKind::RParen);
    let kind = match comma {
        true => SyntaxKind::TupleExpr,
        false => SyntaxKind::ParenExpr,
    };
    m.complete(p, kind)
}

pub(crate) fn block_expr(p: &mut Parser) -> CompletedMarker {
    let m = p.start();
    p.bump(TokenKind::LBrace);
    while p.loop_safe_not_at(TokenKind::RBrace) {
        stmt(p);
    }
    p.bump(TokenKind::RBrace);
    m.complete(p, SyntaxKind::BlockExpr)
}

fn path_or_complex_type_expr(p: &mut Parser, restrictions: ExprRestrictions) -> CompletedMarker {
    let m = p.start();
    p.bump(TokenKind::Ident);
    while p.at(TokenKind::DoubleColon) {
        p.bump(TokenKind::DoubleColon);
        if !p.expect(TokenKind::Ident) {
            break;
        }
    }
    opt_generic_arg_list(p);
    let kind = if p.at(TokenKind::LBrace) && restrictions.allow_struct_expressions {
        struct_expr_field_list(p);
        SyntaxKind::StructExpr
    } else {
        SyntaxKind::PathExpr
    };

    m.complete(p, kind)
}

fn struct_expr_field_list(p: &mut Parser) {
    let m = p.start();
    p.bump(TokenKind::LBrace);
    while p.loop_safe_not_at(TokenKind::RBrace) {
        struct_expr_field(p);
        let comma = p.eat(TokenKind::Comma);
        if p.at(TokenKind::RBrace) {
            break;
        }
        if !comma {
            p.error("expected `,` separating struct expression fields");
        }
    }
    p.expect(TokenKind::RBrace);
    m.complete(p, SyntaxKind::StructExprFieldList);
}

fn struct_expr_field(p: &mut Parser) {
    let m = p.start();
    p.expect(TokenKind::Ident);
    p.expect(TokenKind::Colon);
    expr(p);
    m.complete(p, SyntaxKind::StructExprField);
}
