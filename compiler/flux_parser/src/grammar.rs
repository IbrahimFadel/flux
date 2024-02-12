use flux_lexer::TokenKind;
use flux_syntax::SyntaxKind;

use crate::{parser::Parser, token_set::TokenSet};

use self::{generic_args::opt_generic_arg_list, r#type::type_};

mod expr;
mod generic_args;
mod generic_params;
pub(crate) mod item;
mod r#type;

fn name(p: &mut Parser, recovery_set: TokenSet, parent: &str) -> Option<TokenKind> {
    let m = p.start();
    let recovered_to = if !p.eat(TokenKind::Ident) {
        p.expected("name", parent);
        Some(p.recover_for(recovery_set))
    } else {
        None
    };
    m.complete(p, SyntaxKind::Name);
    recovered_to
}

fn opt_return_type(p: &mut Parser) {
    let m = p.start();
    if p.eat(TokenKind::Arrow) {
        if p.at_set(TokenSet::TYPE_BEGIN) {
            type_(p, "function return type");
        } else {
            p.error("expected return type following `->`");
        }
    }
    m.complete(p, SyntaxKind::FnReturnType);
}

pub enum PathType {
    Regular,
    This,
}

fn path(p: &mut Parser) -> PathType {
    let m = p.start();
    let is_this_path = p.eat(TokenKind::This);
    if !is_this_path {
        p.expect(TokenKind::Ident, "path");
    }
    while p.at(TokenKind::DoubleColon) {
        p.bump(TokenKind::DoubleColon);
        p.expect(TokenKind::Ident, "path");
    }
    opt_generic_arg_list(p);
    m.complete(p, SyntaxKind::Path);
    if is_this_path {
        PathType::This
    } else {
        PathType::Regular
    }
}

#[cfg(test)]
mod tests {
    use flux_diagnostics::SourceCache;

    pub fn parse_and_fmt(src: &str) -> String {
        let mut db = flux_db::Db::default();
        let file = db.new_input_file("test.flx", src.to_string());
        let parse = db.cst(file);
        let (root, diagnostics, interner) = (parse.syntax(), parse.diagnostics, parse.interner);

        let mut file_cache = SourceCache::new(&mut db);
        file_cache.add_input_file(file);

        let mut buf: std::io::BufWriter<Vec<u8>> = std::io::BufWriter::new(Vec::new());
        file_cache.write_diagnostics_to_buffer(&diagnostics, &mut buf);
        let bytes: Vec<u8> = buf.into_inner().unwrap();
        let diagnostics_s = String::from_utf8(bytes).unwrap();
        let cst = root.debug(&interner, true);
        format!("{cst}\n\nErrors:\n{diagnostics_s}")
    }

    #[macro_export]
    macro_rules! test_str {
        ($name:ident, $src:literal) => {
            paste::paste! {
                #[test]
                fn [<$name>]() {
                    let s = crate::grammar::tests::parse_and_fmt($src);
                    insta::assert_snapshot!(s);
                }
            }
        };
    }
}
