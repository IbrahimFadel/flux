use flux_lexer::TokenKind;
use flux_syntax::SyntaxKind;

use crate::{
    grammar::{
        generic_params::{opt_generic_param_list, opt_where_clause},
        name, path,
        r#type::type_,
    },
    marker::CompletedMarker,
    parser::Parser,
    token_set::TokenSet,
};

use super::fn_decl::fn_decl;

pub(crate) fn apply_decl(p: &mut Parser, visibility: CompletedMarker) {
    let m = visibility.precede(p);
    p.bump(TokenKind::Apply);
    opt_generic_param_list(p);

    if p.eat(TokenKind::To) {
        let m = p.start();
        type_(p);
        m.complete(p, SyntaxKind::ApplyDeclType);
    } else {
        let m = p.start();
        path(p);
        m.complete(p, SyntaxKind::ApplyDeclTrait);

        p.expect(TokenKind::To);

        let m = p.start();
        type_(p);
        m.complete(p, SyntaxKind::ApplyDeclType);
    }

    opt_where_clause(p, TokenSet::new(&[TokenKind::LBrace]));

    p.expect(TokenKind::LBrace);
    while p.loop_safe_not_at(TokenKind::RBrace) {
        apply_decl_assoc_type_or_method(p);
    }

    p.expect(TokenKind::RBrace);

    m.complete(p, SyntaxKind::ApplyDecl);
}

fn apply_decl_assoc_type_or_method(p: &mut Parser) {
    if p.at(TokenKind::Type) {
        apply_decl_assoc_type(p);
    } else if p.at(TokenKind::Fn) {
        let m = p.start();
        let m = m.complete(p, SyntaxKind::Visibility);
        fn_decl(p, m);
    }
}

fn apply_decl_assoc_type(p: &mut Parser) {
    let m = p.start();
    p.bump(TokenKind::Type);
    name(p, TokenSet::new(&[TokenKind::Eq]));
    p.expect(TokenKind::Eq);
    type_(p);
    p.expect(TokenKind::SemiColon);
    m.complete(p, SyntaxKind::ApplyDeclAssocType);
}
