#![feature(result_option_inspect)]

use cstree::GreenNode;
use flux_diagnostics::Diagnostic;
use flux_lexer::Lexer;
use flux_span::FileId;
use flux_syntax::SyntaxNode;
use lasso::ThreadedRodeo;
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

pub fn parse(src: &str, file_id: FileId, string_interner: &'static ThreadedRodeo) -> Parse {
    let tokens: Vec<_> = Lexer::new(src).collect();
    let source = Source::new(&tokens);
    let parser = Parser::new(source);
    let events = parser.parse();
    let sink = Sink::new(&tokens, events, string_interner);
    sink.finish(file_id)
}

#[derive(Debug)]
pub struct Parse {
    green_node: GreenNode,
    pub diagnostics: Vec<Diagnostic>,
}

impl Parse {
    pub fn syntax(&self) -> SyntaxNode {
        SyntaxNode::new_root(self.green_node.clone())
    }
}
