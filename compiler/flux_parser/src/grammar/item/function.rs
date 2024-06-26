use crate::{
    grammar::{
        expr::{atom::block_expr, expr_no_blocks},
        generic_params::{opt_generic_param_list, opt_where_clause},
        name, opt_return_type,
        r#type::{poisoned_type, type_},
    },
    marker::CompletedMarker,
    token_set::TokenSet,
};

use super::*;

pub(crate) fn decl(p: &mut Parser, visibility: CompletedMarker) {
    let m = visibility.precede(p);
    p.bump(TokenKind::Fn);

    let recovered_to = name(
        p,
        TokenSet::new(&[
            TokenKind::CmpLt,
            TokenKind::LParen,
            TokenKind::Arrow,
            TokenKind::LBrace,
        ]),
        "function declaration",
    );

    let skip_param_list = recovered_to
        .map(|token| match token {
            TokenKind::Arrow | TokenKind::LBrace => true,
            _ => false,
        })
        .unwrap_or(false);

    opt_generic_param_list(p);
    if skip_param_list {
        poisoned_params(p);
    } else {
        params(p);
    }

    opt_return_type(p);

    // End at a block expression or `=>` arbitrary expression
    opt_where_clause(p, TokenSet::new(&[TokenKind::LBrace, TokenKind::FatArrow]));

    if p.at(TokenKind::FatArrow) {
        p.bump(TokenKind::FatArrow);
        expr_no_blocks(p);
    } else {
        block_expr(p);
    }

    m.complete(p, SyntaxKind::FnDecl);
}

fn poisoned_params(p: &mut Parser) {
    let m = p.start();
    p.expected("parameter list", "function declaration");
    m.complete(p, SyntaxKind::ParamList);
}

pub(crate) fn params(p: &mut Parser) {
    let m = p.start();
    p.expect(TokenKind::LParen, "function parameter list");
    if p.at(TokenKind::LBrace) {
        p.error("expected `)` in parameter list");
        m.complete(p, SyntaxKind::ParamList);
        return;
    }
    if !p.at(TokenKind::RParen) {
        fn_param(p);
    }
    while p.loop_safe_not_at(TokenKind::RParen) {
        p.expect(TokenKind::Comma, "function parameter list");
        if p.at(TokenKind::Arrow) {
            break;
        }
        fn_param(p);
    }
    p.expect(TokenKind::RParen, "function parameter list");
    m.complete(p, SyntaxKind::ParamList);
}

fn fn_param(p: &mut Parser) {
    let m = p.start();
    let recovered_to = name(
        p,
        TokenSet::new(&[TokenKind::Comma, TokenKind::RParen]),
        "function parameter",
    );
    let skip_type = recovered_to
        .map(|token| match token {
            TokenKind::Comma | TokenKind::RParen => true,
            _ => false,
        })
        .unwrap_or(false);

    if skip_type {
        poisoned_type(p, "function parameter");
    } else {
        type_(p, "function parameter");
    }
    m.complete(p, SyntaxKind::Param);
}

// #[cfg(test)]
// mod tests {
//     use crate::test_str;

//     test_str!(no_name, "fn (){}");
//     test_str!(no_name_or_params, "fn {}");
//     test_str!(missing_params, "fn foo{}");
//     test_str!(missing_rparen, "fn foo({}");
//     test_str!(missing_lparen, "fn foo){}");
//     test_str!(comma_in_paramlist_but_no_params, "fn foo(,){}");
//     test_str!(missing_param_after_comma, "fn foo(x X,){}");
// }
