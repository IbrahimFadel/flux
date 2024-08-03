use crate::{
    grammar::{
        generic_params::{opt_generic_param_list, opt_where_clause},
        name,
        r#type::type_,
    },
    lexer::TokenKind,
    marker::CompletedMarker,
    parser::Parser,
    syntax::SyntaxKind,
    token_set::TokenSet,
};

pub(super) fn decl(p: &mut Parser, visibility: CompletedMarker) {
    let m = visibility.precede(p);
    p.bump(TokenKind::Struct);
    name(
        p,
        TokenSet::new(&[TokenKind::LBrace, TokenKind::CmpLt]),
        "struct declaration",
    );
    opt_generic_param_list(p);
    opt_where_clause(p, TokenSet::new(&[TokenKind::LBrace]));
    if !p.eat(TokenKind::LBrace) {
        p.error("`{` in struct declaration");
    }
    struct_decl_field_list(p);
    m.complete(p, SyntaxKind::StructDecl);
}

fn struct_decl_field_list(p: &mut Parser) {
    let m = p.start();
    while p.loop_safe_not_at(TokenKind::RBrace) {
        struct_decl_field(p);
        if !p.eat(TokenKind::Comma) {
            break;
        }
    }
    p.expect(TokenKind::RBrace, "struct declaration field list");
    m.complete(p, SyntaxKind::StructDeclFieldList);
}

fn struct_decl_field(p: &mut Parser) {
    let m = p.start();
    name(
        p,
        TokenSet::new(&[TokenKind::Colon]),
        "struct field declaration",
    );
    type_(p, "struct field declaration");
    m.complete(p, SyntaxKind::StructDeclField);
}
