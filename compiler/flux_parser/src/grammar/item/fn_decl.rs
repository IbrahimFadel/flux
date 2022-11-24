use crate::{
    grammar::{
        expr::{atom::block_expr, expr_no_blocks},
        generic_params::{opt_generic_param_list, opt_where_clause},
        name, opt_return_type,
        r#type::type_,
    },
    marker::CompletedMarker,
    token_set::TokenSet,
};

use super::*;

pub(crate) fn fn_decl(p: &mut Parser, visibility: CompletedMarker) {
    let m = visibility.precede(p);
    p.bump(TokenKind::Fn);
    name(p, ITEM_RECOVERY_SET);
    opt_generic_param_list(p);
    if p.at(TokenKind::LParen) {
        fn_params(p);
    } else {
        let m = p.start();
        p.error("expected function parameters");
        m.complete(p, SyntaxKind::ParamList);
    }
    opt_return_type(p);
    opt_where_clause(p, TokenSet::new(&[TokenKind::LBrace, TokenKind::FatArrow]));

    if p.at(TokenKind::FatArrow) {
        p.bump(TokenKind::FatArrow);
        expr_no_blocks(p);
    } else {
        block_expr(p);
    }

    m.complete(p, SyntaxKind::FnDecl);
}

pub(crate) fn fn_params(p: &mut Parser) {
    let m = p.start();
    p.expect(TokenKind::LParen);
    if p.at(TokenKind::LBrace) {
        p.error("expected `)` in parameter list");
        m.complete(p, SyntaxKind::ParamList);
        return;
    }
    if !p.at(TokenKind::RParen) {
        fn_param(p);
    }
    while p.loop_safe_not_at(TokenKind::RParen) {
        p.expect(TokenKind::Comma);
        if p.at(TokenKind::Arrow) {
            break;
        }
        fn_param(p);
    }
    p.expect(TokenKind::RParen);
    m.complete(p, SyntaxKind::ParamList);
}

fn fn_param(p: &mut Parser) {
    let m = p.start();
    p.expect(TokenKind::Ident);
    type_(p);
    m.complete(p, SyntaxKind::Param);
}
