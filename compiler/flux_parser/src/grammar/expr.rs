use flux_lexer::TokenKind;
use flux_syntax::SyntaxKind;

use crate::{marker::CompletedMarker, parser::Parser, token_set::TokenSet};

use super::{name, r#type::type_};

pub(crate) mod atom;

#[derive(Debug, Clone, Copy)]
pub(crate) struct ExprRestrictions {
    pub allow_struct_expressions: bool,
    pub allow_block_expressions: bool,
}

pub(super) fn expr(p: &mut Parser) -> bool {
    expr_binding_power(
        p,
        1,
        ExprRestrictions {
            allow_struct_expressions: true,
            allow_block_expressions: true,
        },
    )
    .is_some()
}

// pub(crate) fn expr_no_structs(p: &mut Parser) {
//     expr_binding_power(
//         p,
//         1,
//         ExprRestrictions {
//             allow_struct_expressions: false,
//             allow_block_expressions: true,
//         },
//     );
// }

pub(crate) fn expr_no_blocks(p: &mut Parser) {
    expr_binding_power(
        p,
        1,
        ExprRestrictions {
            allow_struct_expressions: true,
            allow_block_expressions: false,
        },
    );
}

fn current_op_prec(p: &mut Parser) -> u8 {
    match p.current() {
        TokenKind::Eq => 2,
        TokenKind::CmpEq
        | TokenKind::CmpNeq
        | TokenKind::CmpLt
        | TokenKind::CmpGt
        | TokenKind::CmpLte
        | TokenKind::CmpGte => 3,
        TokenKind::Plus | TokenKind::Minus => 4,
        TokenKind::Star | TokenKind::Slash => 5,
        _ => 0,
    }
}

fn expr_binding_power(
    p: &mut Parser,
    minimum_binding_power: u8,
    restrictions: ExprRestrictions,
) -> Option<CompletedMarker> {
    let mut lhs = lhs(p, restrictions)?;
    loop {
        let op = p.current();
        let op_bp = current_op_prec(p);
        if op_bp < minimum_binding_power {
            break;
        }

        if p.at(TokenKind::As) {
            lhs = cast_expr(p, lhs);
            continue;
        }

        let m = lhs.precede(p);
        p.bump(op);

        expr_binding_power(p, op_bp + 1, restrictions);
        lhs = m.complete(p, SyntaxKind::BinExpr);
    }
    Some(lhs)
}

fn lhs(p: &mut Parser, restrictions: ExprRestrictions) -> Option<CompletedMarker> {
    let m;
    let kind = match p.current() {
        TokenKind::Ampersand => {
            m = p.start();
            p.bump(TokenKind::Ampersand);
            SyntaxKind::AddressExpr
        }
        TokenKind::Star => {
            m = p.start();
            p.bump(TokenKind::Ampersand);
            SyntaxKind::DerefExpr
        }
        // TokenKind::LBrace => {1
        //     if restrictions.allow_block_expressions {
        //         SyntaxKind::BlockExpr
        //     } else {
        //         p.err_recover(
        //             "block expressions are not allowed here",
        //             TokenSet::new(&[TokenKind::RBrace]),
        //         );
        //         return None;
        //     }
        // }
        _ => {
            let lhs = atom::atom(p, restrictions)?;
            let m = postfix_expr(p, lhs);
            return Some(m);
        }
    };
    expr_binding_power(p, 255, restrictions);
    let m = m.complete(p, kind);
    Some(m)
}

fn postfix_expr(p: &mut Parser, mut lhs: CompletedMarker) -> CompletedMarker {
    loop {
        lhs = match p.current() {
            TokenKind::LParen => call_expr(p, lhs),
            TokenKind::LSquare => idx_expr(p, lhs),
            TokenKind::Period => todo!(),
            _ => break,
        };
    }
    lhs
}

fn cast_expr(p: &mut Parser, lhs: CompletedMarker) -> CompletedMarker {
    let m = lhs.precede(p);
    p.bump(TokenKind::As);
    type_(p);
    m.complete(p, SyntaxKind::CastExpr)
}

fn call_expr(p: &mut Parser, callee: CompletedMarker) -> CompletedMarker {
    let m = callee.precede(p);
    arg_list(p);
    m.complete(p, SyntaxKind::CallExpr)
}

fn idx_expr(p: &mut Parser, callee: CompletedMarker) -> CompletedMarker {
    let m = callee.precede(p);
    p.bump(TokenKind::LSquare);
    expr(p);
    m.complete(p, SyntaxKind::IdxExpr)
}

fn arg_list(p: &mut Parser) {
    let m = p.start();
    p.bump(TokenKind::LParen);
    while p.loop_safe_not_at(TokenKind::RParen) {
        if !expr(p) {
            break;
        }
        if !p.at(TokenKind::RParen) && !p.expect(TokenKind::Comma) {
            break;
        }
    }
    p.expect(TokenKind::RParen);
    m.complete(p, SyntaxKind::ArgList);
}

fn stmt(p: &mut Parser) {
    if p.at(TokenKind::Let) {
        let_stmt(p);
    } else {
        let m = p.start();
        expr(p);
        p.eat(TokenKind::SemiColon);
        m.complete(p, SyntaxKind::ExprStmt);
    }
}

fn let_stmt(p: &mut Parser) -> CompletedMarker {
    let m = p.start();
    p.bump(TokenKind::Let);
    name(p, TokenSet::new(&[TokenKind::Eq, TokenKind::SemiColon]));
    if !p.at(TokenKind::Eq) {
        type_(p);
    }
    if p.eat(TokenKind::Eq) {
        expr(p);
    } else {
        p.err_recover(
            "expected `=` in let statement",
            TokenSet::new(&[TokenKind::SemiColon, TokenKind::RBrace]),
        );
    }
    p.expect(TokenKind::SemiColon);
    m.complete(p, SyntaxKind::LetStmt)
}
