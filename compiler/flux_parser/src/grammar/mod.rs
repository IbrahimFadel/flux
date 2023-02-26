use flux_lexer::TokenKind;
use flux_syntax::SyntaxKind;

use crate::{parser::Parser, token_set::TokenSet};

use self::{generic_args::opt_generic_arg_list, r#type::type_};

mod expr;
mod generic_args;
mod generic_params;
pub mod item;
mod r#type;

fn name(p: &mut Parser, _recovery: TokenSet) {
    let m = p.start();
    if !p.eat(TokenKind::Ident) {
        p.error("expected a name");
        // p.err_recover("expected a name", recovery);
    }
    m.complete(p, SyntaxKind::Name);
}

fn opt_return_type(p: &mut Parser) {
    let m = p.start();
    if p.eat(TokenKind::Arrow) {
        if p.at_set(TokenSet::TYPE_BEGIN) {
            type_(p);
        } else {
            p.error("expected return type following `->`");
        }
    }
    m.complete(p, SyntaxKind::FnReturnType);
}

fn path(p: &mut Parser) {
    let m = p.start();
    p.expect(TokenKind::Ident);
    while p.at(TokenKind::DoubleColon) {
        p.bump(TokenKind::DoubleColon);
        p.expect(TokenKind::Ident);
    }
    opt_generic_arg_list(p);
    m.complete(p, SyntaxKind::Path);
}
