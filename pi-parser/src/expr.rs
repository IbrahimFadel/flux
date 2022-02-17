use super::*;

pub fn type_expr(input: &str) -> IResult<&str, ast::Expr> {
    let res = alt((
        map(tag("i64"), |_| {
            ast::Expr::PrimitiveType(ast::PrimitiveType::new(ast::PrimitiveKind::I64))
        }),
        map(tag("u64"), |_| {
            ast::Expr::PrimitiveType(ast::PrimitiveType::new(ast::PrimitiveKind::U64))
        }),
        map(tag("i32"), |_| {
            ast::Expr::PrimitiveType(ast::PrimitiveType::new(ast::PrimitiveKind::I32))
        }),
        map(tag("u32"), |_| {
            ast::Expr::PrimitiveType(ast::PrimitiveType::new(ast::PrimitiveKind::U32))
        }),
    ))(input);
    return res;
}

pub fn expr(input: &str) -> IResult<&str, ast::Expr> {
    let (mut rest, mut curr) = atom(input).unwrap();

    match pair(
        delimited(opt(ws), alt((tag("=="), tag("!="))), opt(ws)),
        atom,
    )(rest)
    {
        Ok((r, (op, result))) => {
            let binop = match op {
                "==" => ast::OpKind::CmpEq,
                "!=" => ast::OpKind::CmpNEq,
                _ => ast::OpKind::Illegal,
            };
            rest = r;
            curr = ast::Expr::BinOp(ast::BinOp::new(
                Box::from(curr.clone()),
                binop,
                Box::from(result),
            ));
        }
        Err(_) => (),
    };

    loop {
        match pair(
            delimited(opt(ws), alt((tag("&&"), tag("||"))), opt(ws)),
            expr,
        )(rest)
        {
            Ok((r, (op, result))) => {
                let binop = match op {
                    "&&" => ast::OpKind::And,
                    "||" => ast::OpKind::Or,
                    _ => ast::OpKind::Illegal,
                };
                rest = r;
                curr = ast::Expr::BinOp(ast::BinOp::new(
                    Box::from(curr.clone()),
                    binop,
                    Box::from(result),
                ));
            }
            Err(_) => break,
        }
    }

    return Ok((rest, curr));
}

fn atom(input: &str) -> IResult<&str, ast::Expr> {
    alt((
        map(float_lit, ast::Expr::FloatLit),
        map(int_lit, ast::Expr::IntLit),
    ))(input)
}

fn int_lit(input: &str) -> IResult<&str, ast::IntLit> {
    let x = alt((
        map(preceded(tag("0x"), hex_digit1), |v| {
            ast::IntLit::from_str_radix(v, 16).unwrap()
        }),
        map(preceded(tag("0b"), is_a("01")), |v| {
            ast::IntLit::from_str_radix(v, 2).unwrap()
        }),
        map(digit1, |v| ast::IntLit::from_str_radix(v, 10).unwrap()),
    ))(input);
    return x;
}

fn float_lit(input: &str) -> IResult<&str, ast::FloatLit> {
    map(
        tuple((
            opt(char('-')),
            many1(digit1),
            preceded(char('.'), many1(digit1)),
        )),
        |(sign, left, right): (Option<char>, Vec<&str>, Vec<&str>)| {
            let sign_correction_val: f64 = match sign {
                Some('-') => -1.0,
                _ => 1.0,
            };
            let full_float_str = left[0].to_owned() + "." + right[0];
            let full_float_val: f64 = full_float_str.parse::<f64>().unwrap();
            return ast::FloatLit::from(full_float_val * sign_correction_val);
        },
    )(input)
}
