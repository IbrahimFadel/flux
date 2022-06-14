mod decl;
mod expr;
mod stmt;
use crate::parser::{marker::CompletedMarker, Parser};
pub(crate) use decl::top_level_decl;
use flux_lexer::TokenKind;
use flux_syntax::syntax_kind::SyntaxKind;

#[macro_export]
#[cfg(test)]
macro_rules! test_decl_str {
	($name:ident, $src:literal) => {
		paste::paste! {
				#[test]
				fn [<test_parse_ $name>]() {
					let file_id = flux_error::filesystem::FileId(0);
					let tokens: Vec<_> = flux_lexer::Lexer::new($src).collect();
					let source = crate::Source::new(&tokens, file_id);
					let mut parser = crate::Parser::new(source);
					crate::grammar::decl::top_level_decl(&mut parser);
					let sink = crate::Sink::new(&tokens, parser.events);
					let parse = sink.finish();
					let mut settings = insta::Settings::clone_current();
					settings.set_snapshot_path("./snapshots");
					settings.bind(|| {
						insta::assert_snapshot!(&parse.debug_tree());
					});
				}
		}
	};
}

#[macro_export]
#[cfg(test)]
macro_rules! test_stmt_str {
	($name:ident, $src:literal) => {
		paste::paste! {
				#[test]
				fn [<test_parse_ $name>]() {
					let file_id = flux_error::filesystem::FileId(0);
					let tokens: Vec<_> = flux_lexer::Lexer::new($src).collect();
					let source = crate::Source::new(&tokens, file_id);
					let mut parser = crate::Parser::new(source);
					crate::grammar::stmt::stmt(&mut parser);
					let sink = crate::Sink::new(&tokens, parser.events);
					let parse = sink.finish();
					let mut settings = insta::Settings::clone_current();
					settings.set_snapshot_path("./snapshots");
					settings.bind(|| {
						insta::assert_snapshot!(&parse.debug_tree());
					});
				}
		}
	};
}

#[macro_export]
#[cfg(test)]
macro_rules! test_expr_str {
	($name:ident, $src:literal) => {
		paste::paste! {
				#[test]
				fn [<test_parse_ $name>]() {
					let file_id = flux_error::filesystem::FileId(0);
					let tokens: Vec<_> = flux_lexer::Lexer::new($src).collect();
					let source = crate::Source::new(&tokens, file_id);
					let mut parser = crate::Parser::new(source);
					crate::grammar::expr::expr(&mut parser, true);
					let sink = crate::Sink::new(&tokens, parser.events);
					let parse = sink.finish();
					let mut settings = insta::Settings::clone_current();
					settings.set_snapshot_path("./snapshots");
					settings.bind(|| {
						insta::assert_snapshot!(&parse.debug_tree());
					});
				}
		}
	};
}
