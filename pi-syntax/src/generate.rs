use std::fs;

mod builder;
pub mod generated_ast;
use builder::Builder;
pub use generated_ast as ast;
use smol_str::SmolStr;

use self::generated_ast::{AstNode, IntExpr};

pub fn generate_ast() {
	let s = fs::read_to_string("pi.ungram").unwrap();
	let grammar: ungrammar::Grammar = s.parse().unwrap();
	let mut b = Builder::new(&grammar);
	b.generate_ast();
	let _ = b.write_ast_to_file();
}

// Some things are easier to write by hand
impl IntExpr {
	pub fn parse(&self) -> u64 {
		self.syntax().first_token().unwrap().text().parse().unwrap()
	}
}
