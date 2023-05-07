use flux_lexer::TokenKind;
use flux_syntax::SyntaxKind;

use crate::parser::Parser;

mod apply;
mod r#enum;
mod function;
mod mod_decl;
mod r#struct;
mod r#trait;
mod r#use;

pub(crate) fn item(p: &mut Parser) {
    let m = p.start();
    p.eat(TokenKind::Pub);
    let m = m.complete(p, SyntaxKind::Visibility);
    match p.peek() {
        TokenKind::Apply => apply::decl(p, m),
        TokenKind::Enum => r#enum::decl(p, m),
        TokenKind::Fn => function::decl(p, m),
        TokenKind::Mod => mod_decl::decl(p, m),
        TokenKind::Struct => r#struct::decl(p, m),
        TokenKind::Trait => r#trait::trait_decl(p, m),
        TokenKind::Use => r#use::decl(p, m),
        _ => {
            p.err_and_bump(
                "expected one of `apply`, `enum`, `fn`, `mod`, `struct`, `trait`, `use`",
            );
        }
    }
}
