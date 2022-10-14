use flux_lexer::TokenKind;
use flux_syntax::SyntaxKind;

use crate::{
    grammar::{
        generic_params::{opt_generic_param_list, opt_where_clause},
        name, opt_return_type,
    },
    marker::CompletedMarker,
    parser::Parser,
    token_set::TokenSet,
};

use super::{fn_decl::fn_params, ITEM_RECOVERY_SET};

pub(crate) fn trait_decl(p: &mut Parser, visibility: CompletedMarker) {
    let m = visibility.precede(p);
    p.bump(TokenKind::Trait);
    name(p, ITEM_RECOVERY_SET);
    opt_generic_param_list(p);
    opt_where_clause(p, TokenSet::new(&[TokenKind::LBrace]));
    if !p.eat(TokenKind::LBrace) {
        p.error("`{` in struct declaration");
    }
    while p.loop_safe_not_at(TokenKind::RBrace) {
        trait_method_or_assoc_type_decl(p);
    }
    p.expect(TokenKind::RBrace);
    m.complete(p, SyntaxKind::TraitDecl);
}

fn trait_method_or_assoc_type_decl(p: &mut Parser) {
    if p.at(TokenKind::Type) {
        assoc_type_decl(p);
    } else if p.at(TokenKind::Fn) {
        trait_method_decl(p);
    } else {
        p.error("trait method or associated type declaration");
    }
}

fn assoc_type_decl(p: &mut Parser) {
    let m = p.start();
    p.bump(TokenKind::Type);
    p.expect(TokenKind::Ident);
    p.expect(TokenKind::SemiColon);
    m.complete(p, SyntaxKind::TraitAssocTypeDecl);
}

fn trait_method_decl(p: &mut Parser) {
    let m = p.start();
    p.bump(TokenKind::Fn);
    name(p, ITEM_RECOVERY_SET);
    opt_generic_param_list(p);
    fn_params(p);
    opt_return_type(p);
    opt_where_clause(p, TokenSet::new(&[TokenKind::SemiColon]));
    p.expect(TokenKind::SemiColon);
    m.complete(p, SyntaxKind::TraitMethodDecl);
}
