use self::{sink::Sink, source::Source};
use cstree::{interning::Resolver, GreenNode};
use errors::ParseError;
use flux_lexer::{Lexer, TokenKind};
use flux_span::FileId;
use flux_syntax::syntax_kind::SyntaxNode;
use parser::Parser;

mod errors;
mod event;
pub mod grammar;
pub mod parser;
pub mod sink;
pub mod source;

const GLOBAL_RECOVERY_SET: &[TokenKind] = &[
	TokenKind::PubKw,
	TokenKind::FnKw,
	TokenKind::TypeKw,
	TokenKind::TraitKw,
	TokenKind::ApplyKw,
];

const TYPE_RECOVERY_SET: &[TokenKind] = &[
	TokenKind::INKw,
	TokenKind::UNKw,
	TokenKind::F64Kw,
	TokenKind::F32Kw,
	TokenKind::Ident,
	TokenKind::LParen,
];

const EXPR_RECOVERY_SET: &[TokenKind] = &[
	TokenKind::IntLit,
	TokenKind::FloatLit,
	TokenKind::Ident,
	TokenKind::LParen,
];

fn recovery<'a>(set: &'a [TokenKind]) -> Vec<TokenKind> {
	[set, GLOBAL_RECOVERY_SET].concat()
}

pub fn parse(src: &str, file_id: FileId) -> Parse<impl Resolver> {
	let tokens: Vec<_> = Lexer::new(src).collect();
	let source = Source::new(&tokens, file_id);
	let parser = Parser::new(source);
	let events = parser.parse();
	let sink = Sink::new(&tokens, events);

	sink.finish()
}

#[derive(Debug, Clone)]
pub struct Parse<I> {
	green_node: GreenNode,
	pub resolver: I,
	pub errors: Vec<ParseError>,
}

impl<I> Parse<I> {
	pub fn syntax(&self) -> SyntaxNode {
		SyntaxNode::new_root(self.green_node.clone())
	}
}

impl<I> std::fmt::Display for Parse<I> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let mut s = String::new();
		let tree = format!("{:#?}", self.syntax());
		s.push_str(&tree[0..tree.len() - 1]);
		for error in &self.errors {
			s.push_str(&format!("\n{:?}", error));
		}
		write!(f, "{s}")
	}
}
