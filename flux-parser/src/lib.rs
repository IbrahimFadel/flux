use self::{sink::Sink, source::Source};
use flux_error::{filesystem::FileId, FluxError};
use flux_lexer::Lexer;
use flux_syntax::syntax_kind::SyntaxNode;
use parser::Parser;
use rowan::GreenNode;

mod event;
mod grammar;
mod parser;
mod sink;
mod source;

pub fn parse(src: &str, file_id: FileId) -> Parse {
	let tokens: Vec<_> = Lexer::new(src).collect();
	let source = Source::new(&tokens, file_id);
	let parser = Parser::new(source);
	let events = parser.parse();
	let sink = Sink::new(&tokens, events);

	sink.finish()
}

#[derive(Debug)]
pub struct Parse {
	green_node: GreenNode,
	pub errors: Vec<FluxError>,
}

impl Parse {
	pub fn debug_tree(&self) -> String {
		let mut s = String::new();
		let tree = format!("{:#?}", self.syntax());
		s.push_str(&tree[0..tree.len() - 1]);
		for error in &self.errors {
			s.push_str(&format!("\n{:?}", error));
		}
		s
	}

	pub fn syntax(&self) -> SyntaxNode {
		SyntaxNode::new_root(self.green_node.clone())
	}
}
