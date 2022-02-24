use nom::branch::alt;
use nom::bytes::complete::{is_a, tag, take_till1, take_while};
use nom::character::complete::{anychar, char, digit1, hex_digit1, multispace1, not_line_ending};
use nom::combinator::{all_consuming, map, not, opt, recognize, rest, verify};
use nom::multi::{many0, many1, many1_count, separated_list};
use nom::sequence::{delimited, pair, preceded, terminated, tuple};
use pi_ast as ast;
use smol_str::SmolStr;
use std::cell::RefCell;

mod decl;
pub mod err;
mod expr;
mod stmt;
mod utils;

use ast::{Decl, Expr, Stmt};
use err::{expect, Error, LocatedSpan, PIResult, State, ToRange};
use utils::ws;

fn top_level_declarations(input: LocatedSpan) -> PIResult<Vec<Decl>> {
    let decls = many0(decl::fn_);
    return decls(input);
    // let decls = map(decl::fn_, |x| vec![x]);
    // terminated(
    //     decls,
    //     expect(not(anychar), "expected top level declaration or EOF"),
    // )(input)
}

pub fn parse(source: &str) -> (Vec<Decl>, Vec<Error>) {
    let errors = RefCell::new(Vec::new());
    let input = LocatedSpan::new_extra(source, State(&errors));
    let (_, decls) = top_level_declarations(input).expect(
        "internal compiler error: parser should not fail. A bug report would be appreciated",
    );
    // println!("{:?}", errors);
    (decls, errors.into_inner())
}

#[cfg(test)]
mod test {
    use super::*;
    use std::ops::Range;

    #[test]
    fn expr_parsing() {
        let ident = Expr::Ident(ast::Ident::from("foo"));
        let programs = [
            ("foo", ident.clone()),
            ("(foo)", Expr::Paren(Box::from(ident.clone()))),
            ("(foo))", Expr::Paren(Box::from(ident.clone()))),
        ];
        let expected_errors: Vec<Vec<Error>> = vec![
            vec![],
            vec![],
            vec![Error(
                Range { start: 5, end: 6 },
                String::from("expected EOF"),
            )],
        ];

        let mut i = 0;
        for (src, target) in programs {
            let (output, errs) = parse(src);
            // assert_eq!(output, target);
            assert!(errors_eq(&expected_errors[i], &errs));
            i += 1;
        }
    }

    fn errors_eq(x: &[Error], y: &[Error]) -> bool {
        if x.len() != y.len() {
            return false;
        };
        for xerr in x {
            for yerr in y {
                if xerr.0 != yerr.0 {
                    return false;
                }
                if xerr.1 != yerr.1 {
                    return false;
                }
            }
        }
        true
    }
}

// use std::cell::RefCell;
// use std::ops::Range;
// // use std::cell::RefCell;

// use pi_ast as ast;

// use nom::branch::alt;
// use nom::bytes::complete::{take, take_till1, take_while};
// use nom::character::complete::{anychar, char};
// use nom::combinator::{all_consuming, map, not, recognize, rest, verify};
// use nom::sequence::{delimited, preceded, terminated};
// // use utils::{ident, ws};

// // use smol_str::SmolStr;

// // use nom::branch::*;
// // use nom::bytes::complete::*;
// // use nom::character::complete::*;
// // use nom::combinator::all_consuming;
// // use nom::combinator::*;
// // use nom::multi::*;
// // use nom::sequence::*;
// // use nom::IResult;

// // mod decl;
// mod err;
// // pub mod expr;
// // mod stmt;
// // mod token;
// // mod utils;

// use ast::{Expr, Ident};

// use err::{expect, Error, LocatedSpan, PIResult, State};

// pub fn parse(input: &str) -> Box<ast::AST> {
//     let errors = RefCell::new(Vec::new());
//     let input = LocatedSpan::new_extra(input, State(&errors));
//     let (_, expr) = all_consuming(source_file)(input).expect("parser cannot fail");
//     // (expr, errors.into_inner())

//     let ast = ast::AST::new(vec![]);
//     return Box::new(ast);
// }

// fn ident(input: LocatedSpan) -> PIResult<Expr> {
//     let first = verify(anychar, |c| c.is_ascii_alphabetic() || *c == '_');
//     let rest = take_while(|c: char| c.is_ascii_alphanumeric() || "_-'".contains(c));
//     let ident = recognize(preceded(first, rest));
//     map(ident, |span: LocatedSpan| {
//         Expr::Ident(Ident::from(span.fragment().to_string()))
//     })(input)
// }

// fn error(input: LocatedSpan) -> PIResult<Expr> {
//     map(take_till1(|c| c == ')'), |span: LocatedSpan| {
//         let err = Error(span.to_range(), format!("unexpected `{}`", span.fragment()));
//         span.extra.report_error(err);
//         Expr::Error
//     })(input)
// }

// fn expr(input: LocatedSpan) -> PIResult<Expr> {
//     alt((ident, error))(input)
// }

// fn source_file(input: LocatedSpan) -> PIResult<Expr> {
//     let expr = alt((expr, map(take(0usize), |_| Expr::Error)));
//     terminated(expr, preceded(expect(not(anychar), "expected EOF"), rest))(input)
// }

// pub fn parse(input: &str) -> Box<ast::AST> {
//     let errors: RefCell<Vec<Error>> = RefCell::new(Vec::new());
//     let input = LocatedSpan::new_extra(input, State(&errors));
//     let (_, expr) = all_consuming(parse_source)(input).expect("parser cannot fail");
//     // let (_, expr) = all_consuming(source_file)(input).expect("parser cannot fail");
//     // (expr, errors.into_inner())
//     // let mut top_level_declarations = Vec::new();
//     // let (_, decl) = top_level_decl(input).unwrap();
//     // top_level_declarations.push(decl);
//     // let ast = ast::AST::new(top_level_declarations);
//     let ast = ast::AST::new(vec![]);
//     return Box::new(ast);
// }

// fn parse_source(input: &str) -> PIResult<ast::Expr> {
//     let expr = alt((expr::expr, map(take(0usize), |_| ast::Expr::Error)));
//     terminated(expr, preceded(expect(not(anychar), "expected EOF"), rest))(input)
// }

// fn top_level_decl(input: &str) -> PIResult<ast::Decl> {
//     alt((
//         map(decl::fn_, ast::Decl::FnDecl),
//         map(decl::type_, ast::Decl::TypeDecl),
//     ))(input)
// }

// fn generic_types(input: &str) -> PIResult<ast::GenericTypes> {
//     let generics = delimited(
//         preceded(opt(ws), char('<')),
//         separated_list1(char(','), delimited(opt(ws), ident, opt(ws))),
//         preceded(opt(ws), char('>')),
//     );
//     let x = map(generics, |vals| {
//         return vals.into_iter().map(|x| SmolStr::from(x)).collect();
//     })(input);
//     return x;
// }
