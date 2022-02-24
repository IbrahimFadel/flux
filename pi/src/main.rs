use std::fs;

use pi_lexer::*;

fn main() {
    let input = fs::read_to_string("examples/src/main.pi").unwrap();

    tokenize(input.as_str());
}

// use colored::Colorize;
// use std::fs;

// use pi_parser::*;

// fn main() {
//     let input = fs::read_to_string("examples/src/main.pi").unwrap();
//     let (_declarations, _errors) = parse(input.as_str());
//     // println!("{:?}", _declarations);
//     for err in &_errors {
//         // print_err(&input, &err);
//         println!("{:?}", err);
//     }
//     let len = _errors.len();
//     let mut len_str = (len.to_string() + " error").green();
//     if len > 0 {
//         len_str = len_str.red();
//     }
//     println!("Compiled with {}", len_str);
//     // let _ = fs::write("ast.txt", ast.to_string());
// }

// fn print_err(input: &String, err: &err::Error) {
//     let err_range = err.0.clone();
//     let line_num = (&input[0..err_range.clone().start].matches('\n').count() + 1) as i32;
//     let lines: Vec<&str> = input.split('\n').collect();
//     let line_padding = 2; // surround error line with 2 non-error lines

//     let chars_in_each_line: Vec<usize> = (&lines).into_iter().map(|x| x.len()).collect();
//     let mut pos_to_right = 0;
//     for num in chars_in_each_line {
//         if pos_to_right + num < err_range.start {
//             pos_to_right += num;
//         } else {
//             pos_to_right += err_range.start;
//         }
//     }

//     let mut starting_line = line_num - line_padding;
//     if starting_line <= 0 {
//         starting_line = 0;
//     }
//     let mut ending_line = line_num + line_padding;
//     if ending_line >= lines.len() as i32 {
//         ending_line = lines.len() as i32;
//     }
//     println!("---------------------------------");
//     for i in starting_line..ending_line {
//         let mut line_num_str = (i + 1).to_string();
//         if i + 1 == line_num {
//             line_num_str = line_num_str.yellow().bold().to_string();
//         }
//         println!("{} | {}", line_num_str, lines[(i) as usize]);
//         if i + 1 == line_num {
//             println!(
//                 "{}{}\n{}",
//                 " ".repeat(pos_to_right + i.to_string().len()),
//                 "^".yellow(),
//                 err.1.red()
//             );
//         }
//     }
//     println!("---------------------------------\n");
// }
