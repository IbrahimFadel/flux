use super::*;

pub fn block(input: LocatedSpan) -> PIResult<ast::BlockStmt> {
    let block_stmt = delimited(
        preceded(
            opt(ws),
            expect(char('{'), "expected '{' at beginning of block"),
        ),
        many0(delimited(opt(ws), stmt_, opt(ws))),
        preceded(opt(ws), char('}')),
    )(input);
    return block_stmt;
}

fn stmt_(input: LocatedSpan) -> PIResult<Stmt> {
    let x = alt((
        map(var_decl_stmt, ast::Stmt::VarDecl),
        map(if_, ast::Stmt::If),
        // map(expr::expr, ast::Stmt::ExprStmt),
        // map(expr::call, ast::Stmt::ExprStmt),
    ))(input);
    // let x = alt(map(var_decl_stmt, Stmt::VarDecl))(input);
    return x;
}

fn var_decl_stmt(input: LocatedSpan) -> PIResult<ast::VarDecl> {
    let var_decl = tuple((
        preceded(opt(ws), expr::type_expr),
        delimited(
            ws,
            separated_list(char(','), preceded(opt(ws), expr::ident)),
            opt(ws),
        ),
        opt(preceded(
            char('='),
            delimited(
                ws,
                separated_list(char(','), preceded(opt(ws), expr::expr)),
                opt(ws),
            ),
        )),
        preceded(char(';'), opt(ws)),
    ));
    let x = map(var_decl, |(ty, names, values, _)| {
        let n: Vec<SmolStr> = names
            .into_iter()
            .map(|x| ast::Ident::from(x.fragment()))
            .collect();
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

fn if_(input: LocatedSpan) -> PIResult<ast::If> {
    let cond = preceded(tag("if"), preceded(ws, expr::expr));
    let if_stmt = pair(cond, preceded(opt(ws), block));
    map(if_stmt, |(cond, then)| ast::If::new(cond, then))(input)
}

// fn for_(input: &str) -> PIResult<ast::For> {
//     Ok((input, ast::For::new()))
// }
