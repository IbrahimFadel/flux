use colored::Colorize;
use std::fs;

use pi_parser::*;

fn main() {
    let input = fs::read_to_string("examples/src/main.pi").unwrap();
    let (_declarations, errors) = parse(input.as_str());
    for err in &errors {
        print_err(&input, &err);
        println!("{:?}", err);
    }
    println!(
        "Compiled with {} {}",
        errors.len().to_string().red(),
        "errors".red()
    );
    // let _ = fs::write("ast.txt", ast.to_string());
}

fn print_err(input: &String, err: &err::Error) {
    let err_range = err.0.clone();
    let line_num = (&input[0..err_range.clone().start].matches('\n').count() + 1) as i32;
    let lines: Vec<&str> = input.split('\n').collect();
    let line_padding = 2; // surround error line with 2 non-error lines

    let mut starting_line = line_num - line_padding;
    if starting_line <= 0 {
        starting_line = 0;
    }
    let mut ending_line = line_num + line_padding;
    if ending_line >= lines.len() as i32 {
        ending_line = lines.len() as i32;
    }
    println!("---------------------------------");
    for i in starting_line..ending_line {
        let mut line_num_str = (i + 1).to_string();
        if i + 1 == line_num {
            line_num_str = line_num_str.yellow().bold().to_string();
        }
        println!("{} | {}", line_num_str, lines[(i) as usize]);
        if i + 1 == line_num {
            println!("{}\n{}", "^".yellow(), err.1.red());
        }
    }
    println!("---------------------------------\n");
}
