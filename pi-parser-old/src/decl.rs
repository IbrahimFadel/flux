use super::*;

// pub fn type_(input: &str) -> PIResult<ast::TypeDecl> {
//     Ok((input, ast::TypeDecl {}))
// }

pub fn fn_(input: LocatedSpan) -> PIResult<Decl> {
    let fn_decl = tuple((
        preceded(
            preceded(preceded(opt(ws), tag("fn")), ws),
            expect(expr::ident, "expected identifier after 'fn'"),
        ),
        opt(generic_types),
        preceded(opt(ws), fn_params),
        opt(fn_ret),
        delimited(opt(ws), stmt::block, opt(ws)),
    ));
    let x = map(fn_decl, |(name, generics, params, ret_ty, block)| {
        let generic_types = match generics {
            Some(x) => x,
            _ => Vec::new(),
        };
        let name_val = match name {
            Some(x) => Expr::Ident(ast::Ident::from(x.fragment())),
            _ => Expr::Error,
        };
        println!("{:?}", ret_ty);
        let ret_val = match ret_ty {
            Some(Expr::Error) => {
                Expr::PrimitiveType(ast::PrimitiveType::new(ast::PrimitiveKind::Void))
            }
            Some(x) => x,
            _ => Expr::PrimitiveType(ast::PrimitiveType::new(ast::PrimitiveKind::Void)),
        };
        println!("{:?}", ret_val);
        Decl::FnDecl(ast::FnDecl::new(
            name_val,
            generic_types,
            params,
            ret_val,
            block,
        ))
    })(input);
    return x;
}

fn generic_types(input: LocatedSpan) -> PIResult<ast::GenericTypes> {
    let generics = delimited(
        preceded(opt(ws), char('<')),
        separated_list(
            char(','),
            delimited(
                opt(ws),
                expect(expr::ident, "expected identifier in generic type list"),
                opt(ws),
            ),
        ),
        preceded(opt(ws), char('>')),
    );
    let x = map(generics, |vals| {
        let result: Vec<Expr> = vals
            .into_iter()
            .map(|y| match y {
                Some(v) => Expr::Ident(ast::Ident::from(v.fragment())),
                _ => Expr::Error,
            })
            .collect();
        result
        // vec![]
    })(input);
    // let x = generics(input);
    return x;
}

fn fn_ret(input: LocatedSpan) -> PIResult<ast::Expr> {
    map(
        preceded(
            delimited(
                opt(ws),
                expect(tag("->"), "expected '->' before function return type"),
                opt(ws),
            ),
            expect(expr::type_expr, "expected return type following '->'"),
        ),
        |x| match x {
            Some(y) => y,
            _ => Expr::Error,
        },
    )(input)
}

fn fn_params(input: LocatedSpan) -> PIResult<Vec<ast::FnParam>> {
    delimited(
        expect(char('('), "expected '(' before function params"),
        separated_list(char(','), delimited(opt(ws), param, opt(ws))),
        expect(char(')'), "expected ')' after function params"),
    )(input)
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
