use crate::{
    grammar::{
        generic_params::{opt_generic_param_list, opt_where_clause},
        name, path,
        r#type::type_,
    },
    lexer::TokenKind,
    marker::CompletedMarker,
    parser::Parser,
    syntax::SyntaxKind,
    token_set::TokenSet,
};

use super::function;

pub(crate) fn decl(p: &mut Parser, visibility: CompletedMarker) {
    let m = visibility.precede(p);
    p.bump(TokenKind::Apply);
    opt_generic_param_list(p);

    if p.eat(TokenKind::To) {
        let m = p.start();
        type_(p, "apply declaration");
        m.complete(p, SyntaxKind::ApplyDeclType);
    } else {
        let m = p.start();
        path(p);
        m.complete(p, SyntaxKind::ApplyDeclTrait);

        p.expect(TokenKind::To, "apply declaration");

        let m = p.start();
        type_(p, "apply declaration");
        m.complete(p, SyntaxKind::ApplyDeclType);
    }

    opt_where_clause(p, TokenSet::new(&[TokenKind::LBrace]));

    p.expect(TokenKind::LBrace, "apply declaration");
    while p.loop_safe_not_at(TokenKind::RBrace) {
        apply_decl_assoc_type_or_method(p);
    }

    p.expect(TokenKind::RBrace, "apply declaration");

    m.complete(p, SyntaxKind::ApplyDecl);
}

fn apply_decl_assoc_type_or_method(p: &mut Parser) {
    if p.at(TokenKind::Type) {
        apply_decl_assoc_type(p);
    } else if p.at(TokenKind::Fn) {
        let m = p.start();
        let m = m.complete(p, SyntaxKind::Visibility);
        function::decl(p, m);
    }
}

fn apply_decl_assoc_type(p: &mut Parser) {
    let m = p.start();
    p.bump(TokenKind::Type);
    name(
        p,
        TokenSet::new(&[TokenKind::Eq]),
        "associated type declaration",
    );
    p.expect(TokenKind::Eq, "associated type declaration");
    type_(p, "associated type declaration");
    p.expect(TokenKind::SemiColon, "associated type declaration");
    m.complete(p, SyntaxKind::ApplyDeclAssocType);
}
