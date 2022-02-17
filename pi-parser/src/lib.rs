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
mod expr;
mod stmt;
mod token;
mod utils;

pub fn parse(input: &str) -> Box<ast::AST> {
    let fns = Vec::new();
    let ast = ast::AST::new(fns);

    let res = top_level_decl(input);

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
