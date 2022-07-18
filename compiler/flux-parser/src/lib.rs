use self::{sink::Sink, source::Source};
use errors::ParseError;
use flux_lexer::Lexer;
use flux_span::FileId;
use flux_syntax::syntax_kind::SyntaxNode;
use parser::Parser;
use rowan::GreenNode;

mod errors;
mod event;
pub mod grammar;
pub mod parser;
pub mod sink;
pub mod source;

pub fn parse(src: &str, file_id: FileId) -> Parse {
	let tokens: Vec<_> = Lexer::new(src).collect();
	let source = Source::new(&tokens, file_id);
	let parser = Parser::new(source);
	let events = parser.parse();
	let sink = Sink::new(&tokens, events);

	sink.finish()
}

#[derive(Debug, Clone)]
pub struct Parse {
	green_node: GreenNode,
	pub errors: Vec<ParseError>,
}

impl Parse {
	pub fn syntax(&self) -> SyntaxNode {
		SyntaxNode::new_root(self.green_node.clone())
	}
}

impl std::fmt::Display for Parse {
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
