use std::fs;

use pi_parser::*;

fn main() {
    let input = fs::read_to_string("examples/test.pi").unwrap();
    let _ = parse(input.as_str());
    // println!("{:?}", ast);
}
