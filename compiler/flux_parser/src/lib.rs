#![feature(trait_upcasting)]

use cstree::{green::GreenNode, interning::TokenInterner};
use flux_diagnostics::Diagnostic;
use flux_lexer::Lexer;
use flux_span::InputFile;
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

pub fn parse(db: &dyn Db, file: InputFile) -> Parse {
    let tokens: Vec<_> = Lexer::new(file.source_text(db)).collect();
    let source = Source::new(&tokens);
    let parser = Parser::new(source);
    let events = parser.parse();
    let sink = Sink::new(&tokens, events);
    sink.finish(file)
}

#[derive(Debug)]
pub struct Parse {
    green_node: GreenNode,
    pub diagnostics: Vec<Diagnostic>,
    pub interner: TokenInterner,
}

impl Parse {
    pub fn syntax(&self) -> SyntaxNode {
        SyntaxNode::new_root(self.green_node.clone())
    }
}

#[extension_trait::extension_trait]
pub impl FluxHirInputFileExt for InputFile {
    fn cst(self, db: &dyn crate::Db) -> Parse {
        parse(db, self)
    }
}

#[salsa::jar(db = Db)]
pub struct Jar();

pub trait Db: salsa::DbWithJar<Jar> + flux_span::Db {}

impl<DB> Db for DB where DB: ?Sized + salsa::DbWithJar<Jar> + flux_span::Db {}
