use std::{error::Error, fs};

use pi_cfg::*;
use pi_error::*;
use pi_lexer::*;
// use pi_mir::*;
use pi_parser::*;

fn main() {
	let project_dir = "examples/crate-1/";
	let parse_cfg = parse_cfg(project_dir);
	let input =
		fs::read_to_string(project_dir.to_owned() + "src/main.pi").expect("could not read file");
	let mut err_reporting = PIErrorReporting::new();
	let file_id = err_reporting
		.add_file(project_dir.to_owned() + "src/main.pi", input.clone())
		.expect("could not add file");
	let (toks, errs) = tokenize(&input, file_id);
	err_reporting.report(errs);
	let (fns, errs) = parse_tokens(&input, toks, file_id);
	err_reporting.report(errs);

	for f in &fns {
		let _ = fs::write("ast.txt", f.to_string());
	}

	// generate_mir(fns);
}
