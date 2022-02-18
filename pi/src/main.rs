use std::fs;

use pi_parser::*;

fn main() {
    let input = fs::read_to_string("examples/test.pi").unwrap();
    let ast = parse(input.as_str());
    println!("{}", ast);
    let _ = fs::write("ast.txt", ast.to_string());
}
