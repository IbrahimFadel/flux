use cstree::green::GreenNode;
use flux_diagnostics::Diagnostic;
use flux_lexer::Lexer;
use flux_span::{FileId, Interner};
use flux_syntax::SyntaxNode;
use parser::Parser;
use sink::Sink;
use source::Source;

mod diagnostics;
mod event;
mod grammar;
mod marker;
mod parser;
mod sink;
mod source;
mod token_set;

pub fn parse(src: &str, file: FileId, interner: &'static Interner) -> Parse {
    let tokens: Vec<_> = Lexer::new(src).collect();
    let source = Source::new(&tokens);
    let parser = Parser::new(source);
    let events = parser.parse();
    let sink = Sink::new(interner, &tokens, events);
    sink.finish(file)
}

#[derive(Debug)]
pub struct Parse {
    pub green_node: GreenNode,
    pub diagnostics: Vec<Diagnostic>,
}

impl Parse {
    pub fn syntax(&self) -> SyntaxNode {
        SyntaxNode::new_root(self.green_node.clone())
    }
}
