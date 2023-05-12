use flux_lexer::TokenKind;
use flux_syntax::SyntaxKind;

use crate::{
    grammar::{expr::arg_list, generic_args::opt_generic_arg_list, name},
    marker::CompletedMarker,
    parser::Parser,
    token_set::TokenSet,
};

use super::{expr, expr_no_blocks, expr_no_structs, stmt, ExprRestrictions};

pub(super) fn atom(p: &mut Parser, restrictions: ExprRestrictions) -> Option<CompletedMarker> {
    let m = match p.peek() {
        TokenKind::IntLit => int_expr(p),
        TokenKind::FloatLit => float_expr(p),
        TokenKind::StringLit => string_expr(p),
        TokenKind::LParen => paren_or_tuple_expr(p),
        TokenKind::LBrace if restrictions.allow_block_expressions => block_expr(p),
        TokenKind::Ident => path_or_complex_type_expr(p, restrictions),
        TokenKind::If => if_expr(p),
        TokenKind::Intrinsic => intrinsic_expr(p),
        _ => {
            p.err_and_bump("expected expression atom");
            return None;
        }
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
            p.expect(TokenKind::Comma, "tuple expression");
        }
    }
    p.expect(
        TokenKind::RParen,
        match comma {
            true => "tuple expression",
            false => "parentheses expression",
        },
    );
    let kind = match comma {
        true => SyntaxKind::TupleExpr,
        false => SyntaxKind::ParenExpr,
    };
    m.complete(p, kind)
}

pub(crate) fn block_expr(p: &mut Parser) -> CompletedMarker {
    let m = p.start();
    /* We can enter this parser from the function declaration parser and it's not guaranteed to be at a LBrace, so do not bump.
    Example:
    ```flux
    fn main() test {

    }
    ```
    the `->` before return type was omitted, so now we are at `test`, not `->`
    */
    p.expect(TokenKind::LBrace, "block expression");
    while p.loop_safe_not_at(TokenKind::RBrace) {
        stmt(p);
    }
    p.expect(TokenKind::RBrace, "block expression");
    m.complete(p, SyntaxKind::BlockExpr)
}

fn path_or_complex_type_expr(p: &mut Parser, restrictions: ExprRestrictions) -> CompletedMarker {
    let m = p.start();
    p.bump(TokenKind::Ident);
    while p.at(TokenKind::DoubleColon) {
        p.bump(TokenKind::DoubleColon);
        if !p.expect(TokenKind::Ident, "path") {
            break;
        }
    }
    opt_generic_arg_list(p);
    if p.at(TokenKind::LBrace) && restrictions.allow_struct_expressions {
        let path = m.complete(p, SyntaxKind::Path);
        let m = path.precede(p);
        struct_expr_field_list(p);
        m.complete(p, SyntaxKind::StructExpr)
    } else {
        m.complete(p, SyntaxKind::PathExpr)
    }
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
    p.expect(TokenKind::RBrace, "struct expression field list");
    m.complete(p, SyntaxKind::StructExprFieldList);
}

fn struct_expr_field(p: &mut Parser) {
    let m = p.start();
    name(
        p,
        TokenSet::new(&[TokenKind::Colon]),
        "struct expression field",
    );
    p.expect(TokenKind::Colon, "struct expression field");
    expr(p);
    m.complete(p, SyntaxKind::StructExprField);
}

fn if_expr(p: &mut Parser) -> CompletedMarker {
    let m = p.start();
    p.bump(TokenKind::If);
    expr_no_structs(p);
    block_expr(p);

    while p.eat(TokenKind::Else) {
        let m = p.start();
        if p.eat(TokenKind::If) {
            expr_no_structs(p);
            block_expr(p);
            m.complete(p, SyntaxKind::ElseIfBlock);
        } else {
            block_expr(p);
            m.complete(p, SyntaxKind::ElseBlock);
        }
    }
    m.complete(p, SyntaxKind::IfExpr)
}

fn intrinsic_expr(p: &mut Parser) -> CompletedMarker {
    let m = p.start();
    p.bump(TokenKind::Intrinsic);
    arg_list(p);
    m.complete(p, SyntaxKind::IntrinsicExpr)
}
