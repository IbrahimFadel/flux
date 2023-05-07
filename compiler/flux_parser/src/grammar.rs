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

fn path(p: &mut Parser) {
    let m = p.start();
    p.expect(TokenKind::Ident, "path");
    while p.at(TokenKind::DoubleColon) {
        p.bump(TokenKind::DoubleColon);
        p.expect(TokenKind::Ident, "path");
    }
    opt_generic_arg_list(p);
    m.complete(p, SyntaxKind::Path);
}

#[cfg(test)]
mod tests {
    use lasso::ThreadedRodeo;
    use once_cell::sync::Lazy;

    static INTERNER: Lazy<ThreadedRodeo> = Lazy::new(ThreadedRodeo::new);

    pub fn parse_and_fmt(src: &str) -> String {
        let tokens: Vec<_> = flux_lexer::Lexer::new(src).collect();
        let source = crate::source::Source::new(&tokens);
        let parser = crate::parser::Parser::new(source);
        let events = parser.parse();
        let sink = crate::sink::Sink::new(&tokens, events, &INTERNER);

        // let file_id = flux_span::FileId::new(&db, "test.flx".to_string());
        let file_id = flux_span::FileId::poisoned();
        let parse = sink.finish(file_id);
        let (root, diagnostics) = (parse.syntax(), parse.diagnostics);

        let mut file_cache = flux_diagnostics::reporting::FileCache::new(&INTERNER);
        file_cache.add_file_with_file_id(file_id, src);

        let mut buf: std::io::BufWriter<Vec<u8>> = std::io::BufWriter::new(Vec::new());
        file_cache.write_diagnostics_to_buffer(&diagnostics, &mut buf);
        let bytes: Vec<u8> = buf.into_inner().unwrap();
        let diagnostics_bytes_without_ansi = strip_ansi_escapes::strip(&bytes).unwrap();
        let diagnostics_s = String::from_utf8(diagnostics_bytes_without_ansi).unwrap();
        let cst = root.debug(&*INTERNER, true);

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
