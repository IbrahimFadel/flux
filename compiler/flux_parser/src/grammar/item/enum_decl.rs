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

use super::ITEM_RECOVERY_SET;

pub(crate) fn enum_decl(p: &mut Parser, visibility: CompletedMarker) {
    let m = visibility.precede(p);
    p.bump(TokenKind::Enum);
    name(p, ITEM_RECOVERY_SET);
    opt_generic_param_list(p);
    opt_where_clause(p, TokenSet::new(&[TokenKind::LBrace]));
    if !p.eat(TokenKind::LBrace) {
        p.error("`{` in struct declaration");
    }
    while p.loop_safe_not_at(TokenKind::RBrace) {
        enum_decl_variant(p);
        if !p.eat(TokenKind::Comma) {
            break;
        }
    }
    p.expect(TokenKind::RBrace);
    m.complete(p, SyntaxKind::EnumDecl);
}

fn enum_decl_variant(p: &mut Parser) {
    let m = p.start();
    name(p, TokenSet::new(&[TokenKind::Arrow, TokenKind::Comma]));
    if !p.at(TokenKind::Comma) {
        p.expect(TokenKind::Arrow);
        type_(p);
    }
    m.complete(p, SyntaxKind::EnumDeclVariant);
}
