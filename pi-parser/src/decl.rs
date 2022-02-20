use super::*;

// pub fn type_(input: &str) -> PIResult<ast::TypeDecl> {
//     Ok((input, ast::TypeDecl {}))
// }

pub fn fn_(input: LocatedSpan) -> PIResult<Decl> {
    let fn_decl = tuple((
        preceded(preceded(preceded(opt(ws), tag("fn")), ws), expr::ident),
        opt(generic_types),
        fn_params,
        fn_ret,
        delimited(opt(ws), stmt::block, opt(ws)),
    ));
    let x = map(fn_decl, |(name, generics, params, ret_ty, block)| {
        let generic_types = match generics {
            Some(x) => x,
            _ => Vec::new(),
        };
        Decl::FnDecl(ast::FnDecl::new(
            ast::Ident::from(name.fragment()),
            generic_types,
            params,
            ret_ty,
            block,
        ))
    })(input);
    return x;
}

fn generic_types(input: LocatedSpan) -> PIResult<ast::GenericTypes> {
    let generics = delimited(
        preceded(opt(ws), char('<')),
        separated_list(char(','), delimited(opt(ws), expr::ident, opt(ws))),
        preceded(opt(ws), char('>')),
    );
    let x = map(generics, |vals| {
        return vals
            .into_iter()
            .map(|x| SmolStr::from(x.fragment()))
            .collect();
    })(input);
    // let x = generics(input);
    return x;
}

fn fn_ret(input: LocatedSpan) -> PIResult<ast::Expr> {
    let (rest, _) = delimited(opt(ws), tag("->"), opt(ws))(input).unwrap();
    expr::type_expr(rest)
}

fn fn_params(input: LocatedSpan) -> PIResult<Vec<ast::FnParam>> {
    let res = delimited(
        preceded(opt(ws), char('(')),
        separated_list(char(','), delimited(opt(ws), param, opt(ws))),
        char(')'),
    )(input)
    .unwrap();
    Ok(res)
}

fn param(input: LocatedSpan) -> PIResult<ast::FnParam> {
    let p = tuple((
        pair(opt(pair(tag("mut"), ws)), expr::type_expr),
        preceded(ws, expr::ident),
    ));
    let x = map(p, |((mut_, type_), name)| {
        let is_mut = match mut_ {
            Some(_) => true,
            _ => false,
        };
        ast::FnParam::new(is_mut, type_, SmolStr::from(name.fragment()))
    });
    return x(input);
}
