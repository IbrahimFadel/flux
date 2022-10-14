use flux_lexer::TokenKind;
use flux_syntax::SyntaxKind;

use crate::{parser::Parser, token_set::TokenSet};

use super::generic_args::opt_generic_arg_list;

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
    if !p.eat(TokenKind::Ident) {
        p.err_and_bump("expected type parameter");
    };
    m.complete(p, SyntaxKind::TypeParam);
}

/// Parse an optional [`SyntaxKind::WhereClause`]
///
/// Stops consuming tokens when it reaches a [`TokenKind`] included in `ending_tokens`
///
/// Does not consume ending token
///
/// ```
/// use lasso::ThreadedRodeo;
/// use flux_lexer::Lexer;
/// use flux_span::FileId;
/// use flux_parser::source::Source;

/// let rodeo = ThreadedRodeo::new();
/// let file_id = FileId(rodeo.get_or_intern("main.flx"));
/// let tokens: Vec<_> = Lexer::new(src).collect();
/// let source = Source::new(&tokens, file_id);
/// let parser = Parser::new(source);
/// let events = parser.parse();
/// ```
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
    p.bump(TokenKind::Ident);
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
    p.expect(TokenKind::Ident);
    opt_generic_arg_list(p);
    m.complete(p, SyntaxKind::TypeBound);
    true
}
