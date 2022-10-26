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

pub fn parse(src: &str, file_id: FileId, interner: &'static ThreadedRodeo) -> Parse {
    let tokens: Vec<_> = Lexer::new(src).collect();
    let source = Source::new(&tokens, file_id);
    let parser = Parser::new(source);
    let events = parser.parse();
    let sink = Sink::new(&tokens, events, interner);
    sink.finish()
}

#[derive(Debug, Clone)]
pub struct Parse {
    green_node: GreenNode,
    pub resolver: &'static ThreadedRodeo,
    pub diagnostics: Vec<Diagnostic>,
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
        // for error in &self.errors {
        //     s.push_str(&format!("\n{:?}", error));
        // }
        write!(f, "{s}")
    }
}
