use crate::{lexer::TokenKind, parser::Parser, syntax::SyntaxKind, token_set::TokenSet};

use super::{name, path};

pub(crate) fn opt_generic_param_list(p: &mut Parser) {
    if p.at(TokenKind::CmpLt) {
        generic_param_list(p);
    }
}

fn generic_param_list(p: &mut Parser) {
    let m = p.start();
    p.bump(TokenKind::CmpLt);

    while p.loop_safe_not_at(TokenKind::CmpGt) {
        generic_param(p);
        if !p.at(TokenKind::CmpGt) && !p.expect(TokenKind::Comma, "generic parameter list") {
            break;
        }
    }
    p.expect(TokenKind::CmpGt, "generic parameter list");
    m.complete(p, SyntaxKind::GenericParamList);
}

fn generic_param(p: &mut Parser) {
    let m = p.start();
    name(
        p,
        TokenSet::new(&[TokenKind::Comma, TokenKind::CmpGt]),
        "generic parameter",
    );
    m.complete(p, SyntaxKind::TypeParam);
}

/// Parse an optional [`SyntaxKind::WhereClause`]
///
/// Stops consuming tokens when it reaches a [`TokenKind`] included in `ending_tokens`
///
/// Does not consume ending token
pub(super) fn opt_where_clause(p: &mut Parser, ending_tokens: TokenSet) {
    if !p.at(TokenKind::Where) {
        return;
    }
    let m = p.start();
    p.bump(TokenKind::Where);
    while p.at(TokenKind::Ident) {
        where_predicate(p);
        let comma = p.eat(TokenKind::Comma);
        if ending_tokens.contains(p.peek()) {
            break;
        }
        if !comma {
            p.error("expected `,` separating where predicates");
        }
    }
    m.complete(p, SyntaxKind::WhereClause);
}

fn where_predicate(p: &mut Parser) {
    let m = p.start();
    name(p, TokenSet::new(&[TokenKind::Is]), "where predicate");
    if p.at(TokenKind::Is) {
        bounds(p);
    } else {
        p.error("expected `is`");
    }
    m.complete(p, SyntaxKind::WherePredicate);
}

pub(crate) fn bounds(p: &mut Parser) {
    let m = p.start();
    p.bump(TokenKind::Is);
    while type_bound(p) {
        if !p.eat(TokenKind::Plus) {
            break;
        }
    }
    m.complete(p, SyntaxKind::TypeBoundList);
}

fn type_bound(p: &mut Parser) -> bool {
    let m = p.start();
    path(p);
    m.complete(p, SyntaxKind::TypeBound);
    true
}
