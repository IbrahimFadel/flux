use ast::Ident;

use super::*;

pub fn block(input: &str) -> IResult<&str, ast::BlockStmt> {
    let block_stmt = delimited(
        preceded(opt(ws), char('{')),
        many0(delimited(opt(ws), stmt_, opt(ws))),
        preceded(opt(ws), char('}')),
    )(input);
    return block_stmt;
}

fn stmt_(input: &str) -> IResult<&str, ast::Stmt> {
    let x = alt((
        map(var_decl_stmt, ast::Stmt::VarDecl),
        map(if_, ast::Stmt::If),
        map(expr::call, ast::Stmt::ExprStmt),
    ))(input);
    return x;
}

fn var_decl_stmt(input: &str) -> IResult<&str, ast::VarDecl> {
    let var_decl = tuple((
        preceded(opt(ws), expr::type_expr),
        delimited(
            ws,
            separated_list1(char(','), preceded(opt(ws), ident)),
            opt(ws),
        ),
        opt(preceded(
            char('='),
            delimited(
                ws,
                separated_list1(char(','), preceded(opt(ws), expr::expr)),
                opt(ws),
            ),
        )),
        preceded(char(';'), opt(ws)),
    ));
    let x = map(var_decl, |(ty, names, values, _)| {
        let n: Vec<SmolStr> = names.into_iter().map(|x| Ident::from(x)).collect();
        if (1..n.len()).any(|i| n[i..].contains(&n[i - 1])) {
            panic!("duplicate idents in var decl");
        }
        if let Some(ref v) = values {
            let values_len = v.len();
            if values_len != 1 && values_len != 0 {
                if values_len != n.len() {
                    panic!("invalid number of values in var decl");
                }
            }
        }
        return ast::VarDecl::new(ty, n, values);
    })(input);
    return x;
}

fn if_(input: &str) -> IResult<&str, ast::If> {
    let cond = preceded(tag("if"), preceded(ws, expr::expr));
    let if_stmt = pair(cond, preceded(opt(ws), block));
    map(if_stmt, |(cond, then)| ast::If::new(cond, then))(input)
}

// fn for_(input: &str) -> IResult<&str, ast::For> {
//     Ok((input, ast::For::new()))
// }
