// use std::fs;

// pub mod nodes;
// pub mod tokens;
// use crate::builder::Builder;

// use self::tokens::IntExpr;
// pub use nodes as ast;

// pub fn generate_ast() {
// 	let s = fs::read_to_string("pi.ungram").unwrap();
// 	let grammar: ungrammar::Grammar = s.parse().unwrap();
// 	let mut b = Builder::new(&grammar);
// 	b.generate_ast();
// 	let res = b.write_ast_to_file();
// 	if res.is_err() {
// 		println!("could not write to file: {}", res.err().unwrap());
// 	}
// }

// Some things are easier to write by hand
// impl IntExpr {
// 	pub fn parse(&self) -> u64 {
// 		self.syntax.first_token().unwrap().text().parse().unwrap()
// 	}
// }
