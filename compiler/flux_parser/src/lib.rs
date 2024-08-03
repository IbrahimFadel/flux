use cstree::green::GreenNode;
use flux_diagnostics::Diagnostic;
use flux_util::{FileId, Interner};
use lexer::Lexer;
use parser::Parser;
use sink::Sink;
use source::Source;
use syntax::SyntaxNode;

mod diagnostics;
mod event;
mod grammar;
mod lexer;
mod marker;
mod parser;
mod sink;
mod source;
pub mod syntax;
mod token_set;

pub use syntax::ast;

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
