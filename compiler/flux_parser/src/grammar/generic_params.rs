use flux_lexer::TokenKind;
use flux_syntax::SyntaxKind;

use crate::{parser::Parser, token_set::TokenSet};

use super::{generic_args::opt_generic_arg_list, name, path};

pub(super) fn opt_generic_param_list(p: &mut Parser) {
    if p.at(TokenKind::CmpLt) {
        generic_param_list(p);
    }
}

fn generic_param_list(p: &mut Parser) {
    let m = p.start();
    p.bump(TokenKind::CmpLt);

    while p.loop_safe_not_at(TokenKind::CmpGt) {
        generic_param(p);
        if !p.at(TokenKind::CmpGt) && !p.expect(TokenKind::Comma) {
            break;
        }
    }
    p.expect(TokenKind::CmpGt);
    m.complete(p, SyntaxKind::GenericParamList);
}

fn generic_param(p: &mut Parser) {
    let m = p.start();
    name(p, TokenSet::new(&[TokenKind::CmpGt]));
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
        if ending_tokens.contains(p.current()) {
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
    name(p, TokenSet::new(&[TokenKind::Is]));
    if p.at(TokenKind::Is) {
        bounds(p);
    } else {
        p.error("expected `is`");
    }
    m.complete(p, SyntaxKind::WherePredicate);
}

fn bounds(p: &mut Parser) {
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
    opt_generic_arg_list(p);
    m.complete(p, SyntaxKind::TypeBound);
    true
}
