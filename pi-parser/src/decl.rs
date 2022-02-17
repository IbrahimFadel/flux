use super::*;

pub fn type_(input: &str) -> IResult<&str, ast::TypeDecl> {
    Ok((input, ast::TypeDecl {}))
}

pub fn fn_(input: &str) -> IResult<&str, ast::FnDecl> {
    let fn_decl = tuple((
        preceded(preceded(tag("fn"), ws), ident),
        opt(generic_types),
        fn_params,
        fn_ret,
        stmt::block,
    ));
    let x = map(fn_decl, |(name, generics, params, ret_ty, block)| {
        let generic_types = match generics {
            Some(x) => x,
            _ => Vec::new(),
        };
        ast::FnDecl::new(SmolStr::from(name), generic_types, params, ret_ty, block)
    })(input);
    return x;
}

fn fn_ret(input: &str) -> IResult<&str, ast::Expr> {
    let (rest, _) = delimited(opt(ws), tag("->"), opt(ws))(input).unwrap();
    expr::type_expr(rest)
}

fn fn_params(input: &str) -> IResult<&str, Vec<ast::FnParam>> {
    let res = delimited(
        preceded(opt(ws), char('(')),
        separated_list0(char(','), delimited(opt(ws), param, opt(ws))),
        char(')'),
    )(input)
    .unwrap();
    Ok(res)
}

fn param(input: &str) -> IResult<&str, ast::FnParam> {
    let p = tuple((
        pair(opt(pair(tag("mut"), ws)), expr::type_expr),
        preceded(ws, ident),
    ));
    let mut x = map(p, |((mut_, type_), name)| {
        let is_mut = match mut_ {
            Some(("mut", _)) => true,
            _ => false,
        };
        FnParam::new(is_mut, type_, SmolStr::from(name))
    });
    return x(input);
}
