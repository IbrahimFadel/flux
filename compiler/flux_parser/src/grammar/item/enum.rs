use flux_lexer::TokenKind;
use flux_syntax::SyntaxKind;

use crate::{
    grammar::{
        generic_params::{opt_generic_param_list, opt_where_clause},
        name,
        r#type::type_,
    },
    marker::CompletedMarker,
    parser::Parser,
    token_set::TokenSet,
};

pub(crate) fn decl(p: &mut Parser, visibility: CompletedMarker) {
    let m = visibility.precede(p);
    p.bump(TokenKind::Enum);
    name(
        p,
        TokenSet::new(&[TokenKind::LBrace, TokenKind::CmpLt]),
        "enum declaration",
    );
    opt_generic_param_list(p);
    opt_where_clause(p, TokenSet::new(&[TokenKind::LBrace]));
    if !p.eat(TokenKind::LBrace) {
        p.error("`{` in struct declaration");
    }
    while p.loop_safe_not_at(TokenKind::RBrace) {
        if !enum_decl_variant(p) {
            break;
        }
    }
    p.expect(TokenKind::RBrace, "enum declaration");
    m.complete(p, SyntaxKind::EnumDecl);
}

fn enum_decl_variant(p: &mut Parser) -> bool {
    let m = p.start();
    name(
        p,
        TokenSet::new(&[TokenKind::Arrow, TokenKind::Comma]),
        "enum variant declaration",
    );
    if !p.at_set(TokenSet::new(&[TokenKind::Arrow, TokenKind::RBrace])) {
        p.expect(TokenKind::Comma, "enum variant declaration");
        m.complete(p, SyntaxKind::EnumDeclVariant);
        return true;
    } else if !p.at_set(TokenSet::new(&[TokenKind::Comma, TokenKind::RBrace])) {
        p.expect(TokenKind::Arrow, "enum variant declaration");
        type_(p, "enum variant declaration");
    }
    m.complete(p, SyntaxKind::EnumDeclVariant);

    return p.eat(TokenKind::Comma);
}
