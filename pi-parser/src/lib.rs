use ast::FnParam;
use pi_ast as ast;
use utils::{ident, ws};

use smol_str::SmolStr;

use nom::branch::*;
use nom::bytes::complete::*;
use nom::character::complete::*;
use nom::combinator::*;
use nom::multi::*;
use nom::sequence::*;
use nom::IResult;

mod decl;
pub mod expr;
mod stmt;
mod token;
mod utils;

pub fn parse(input: &str) -> Box<ast::AST> {
    let mut top_level_declarations = Vec::new();

    let (_, decl) = top_level_decl(input).unwrap();
    top_level_declarations.push(decl);
    // let top_level_declarations = many0(top_level_decl)(input);
    // println!("{:?}", top_level_declarations);
    // let res = match top_level_declarations {
    //     Ok((_, decls)) => decls,
    //     Err(_) => vec![],
    // };
    // println!("{:?}", res);
    // let ast = ast::AST::new(vec![]);
    let ast = ast::AST::new(top_level_declarations);
    return Box::new(ast);
}

fn top_level_decl(input: &str) -> IResult<&str, ast::Decl> {
    alt((
        map(decl::fn_, ast::Decl::FnDecl),
        map(decl::type_, ast::Decl::TypeDecl),
    ))(input)
}

fn generic_types(input: &str) -> IResult<&str, ast::GenericTypes> {
    let generics = delimited(
        preceded(opt(ws), char('<')),
        separated_list1(char(','), delimited(opt(ws), ident, opt(ws))),
        preceded(opt(ws), char('>')),
    );
    let x = map(generics, |vals| {
        return vals.into_iter().map(|x| SmolStr::from(x)).collect();
    })(input);
    return x;
}
