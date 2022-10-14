use flux_lexer::TokenKind;
use flux_syntax::SyntaxKind;

use crate::{parser::Parser, token_set::TokenSet};

use self::{
    apply_decl::apply_decl, enum_decl::enum_decl, fn_decl::fn_decl, struct_decl::struct_decl,
    trait_decl::trait_decl,
};

mod apply_decl;
mod enum_decl;
mod fn_decl;
mod struct_decl;
mod trait_decl;

const ITEM_RECOVERY_SET: TokenSet = TokenSet::new(&[
    TokenKind::Fn,
    TokenKind::Struct,
    TokenKind::Enum,
    TokenKind::Trait,
    TokenKind::Let,
    TokenKind::Mod,
    TokenKind::Pub,
    TokenKind::Use,
    TokenKind::SemiColon,
]);

pub(crate) fn item(p: &mut Parser) {
    let m = p.start();
    p.eat(TokenKind::Pub);
    let m = m.complete(p, SyntaxKind::Visibility);
    if p.at(TokenKind::Fn) {
        fn_decl(p, m);
    } else if p.at(TokenKind::Struct) {
        struct_decl(p, m);
    } else if p.at(TokenKind::Enum) {
        enum_decl(p, m);
    } else if p.at(TokenKind::Trait) {
        trait_decl(p, m);
    } else if p.at(TokenKind::Apply) {
        apply_decl(p, m);
    } else {
        // TokenKind::Type => {
        //     type_decl_list(p, m);
        // }
        // TokenKind::Trait => {
        //     trait_decl(p, m);
        // }
        // TokenKind::Apply => {
        //     apply_decl(p, m);
        // }
        p.err_and_bump("expected item");
    }
}
