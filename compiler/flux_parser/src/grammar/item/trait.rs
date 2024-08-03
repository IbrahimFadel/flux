use crate::{
    grammar::{
        generic_params::{bounds, opt_generic_param_list, opt_where_clause},
        name, opt_return_type,
    },
    lexer::TokenKind,
    marker::CompletedMarker,
    parser::Parser,
    syntax::SyntaxKind,
    token_set::TokenSet,
};

use super::function;

pub(crate) fn trait_decl(p: &mut Parser, visibility: CompletedMarker) {
    let m = visibility.precede(p);
    p.bump(TokenKind::Trait);
    name(p, TokenSet::new(&[TokenKind::LBrace]), "trait declaration");
    opt_generic_param_list(p);
    opt_where_clause(p, TokenSet::new(&[TokenKind::LBrace]));
    if !p.eat(TokenKind::LBrace) {
        p.error("`{` in struct declaration");
    }
    while p.loop_safe_not_at(TokenKind::RBrace) {
        trait_method_or_assoc_type_decl(p);
    }
    p.expect(TokenKind::RBrace, "trait declaration");
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
    name(
        p,
        TokenSet::new(&[TokenKind::SemiColon]),
        "trait declaration",
    );

    if p.at(TokenKind::Is) {
        bounds(p);
    }

    p.expect(TokenKind::SemiColon, "trait declaration");
    m.complete(p, SyntaxKind::TraitAssocTypeDecl);
}

fn trait_method_decl(p: &mut Parser) {
    let m = p.start();
    p.bump(TokenKind::Fn);
    name(
        p,
        TokenSet::new(&[TokenKind::LParen]),
        "trait method declaration",
    );
    opt_generic_param_list(p);
    function::params(p);
    opt_return_type(p);
    opt_where_clause(p, TokenSet::new(&[TokenKind::SemiColon]));
    p.expect(TokenKind::SemiColon, "trait method declaration");
    m.complete(p, SyntaxKind::TraitMethodDecl);
}
