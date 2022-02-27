use std::fs;

use pi_error::*;
use pi_lexer::*;
use pi_parser::*;

fn main() {
	let input = fs::read_to_string("examples/src/main.pi").unwrap();
	let mut err_reporting = PIErrorReporting::new();
	let file_id = err_reporting
		.add_file("examples/src/main.pi".to_owned(), input.clone())
		.expect("could not add file");
	let (toks, errs) = tokenize(&input, file_id);
	err_reporting.report(errs);
	let (_, errs) = parse_tokens(&input, toks, file_id);
	err_reporting.report(errs);
}
