use super::*;

pub fn expr(input: LocatedSpan) -> PIResult<Expr> {
    // alt((type_expr, paren, binop, atom, error))(input)
    alt((type_expr, paren, binop, atom))(input)
}

pub fn type_expr(input: LocatedSpan) -> PIResult<Expr> {
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
        map(ident, |v| {
            Expr::Ident(ast::Ident::from(v.fragment().to_string()))
        }),
    ))(input);
    return res;
}

fn term(input: LocatedSpan) -> PIResult<Expr> {
    let (rest, curr) = atom(input)?;
    let x = many0(pair(
        delimited(opt(ws), alt((char('*'), char('/'))), opt(ws)),
        delimited(opt(ws), atom, opt(ws)),
    ));
    let y = map(x, |vals| {
        let mut final_e = curr.clone();
        for e in vals {
            let kind = match e.0 {
                '*' => ast::OpKind::Asterisk,
                '/' => ast::OpKind::Slash,
                _ => ast::OpKind::Illegal,
            };
            final_e = Expr::BinOp(ast::BinOp::new(
                Box::from(final_e.clone()),
                kind,
                Box::from(e.1.clone()),
            ));
        }
        final_e
    })(rest);
    return y;
}

fn binop(input: LocatedSpan) -> PIResult<Expr> {
    let (rest, curr) = term(input)?;
    let x = many0(pair(
        delimited(opt(ws), alt((char('+'), char('-'))), opt(ws)),
        preceded(opt(ws), term),
    ));
    let binop = map(x, |vals| {
        let mut final_e = curr.clone();
        for e in vals {
            let kind = match e.0 {
                '+' => ast::OpKind::Plus,
                '-' => ast::OpKind::Minus,
                _ => ast::OpKind::Illegal,
            };
            final_e = Expr::BinOp(ast::BinOp::new(
                Box::from(final_e.clone()),
                kind,
                Box::from(e.1.clone()),
            ));
        }
        final_e
    })(rest);
    return binop;
}

pub fn atom(input: LocatedSpan) -> PIResult<Expr> {
    alt((
        map(ident, |v| {
            Expr::Ident(ast::Ident::from(v.fragment().to_string()))
        }),
        float_lit,
        int_lit,
    ))(input)
}

fn float_lit(input: LocatedSpan) -> PIResult<Expr> {
    let float = tuple((
        opt(char('-')),
        many1(digit1),
        preceded(char('.'), many1(digit1)),
    ));
    map(
        float,
        |(sign, left, right): (Option<char>, Vec<LocatedSpan>, Vec<LocatedSpan>)| {
            let sign_correction_val: f64 = match sign {
                Some('-') => -1.0,
                _ => 1.0,
            };
            let full_float_str = left[0].fragment().to_string() + "." + right[0].fragment();
            let full_float_val: f64 = full_float_str.parse::<f64>().unwrap() * sign_correction_val;
            Expr::FloatLit(ast::FloatLit::from(full_float_val))
        },
    )(input)
}

fn int_lit(input: LocatedSpan) -> PIResult<Expr> {
    let hex_num = map(preceded(tag("0x"), hex_digit1), |v: LocatedSpan| {
        ast::IntLit::from_str_radix(&v.fragment().to_string(), 16).unwrap()
    });
    let bin_num = map(preceded(tag("0b"), is_a("01")), |v: LocatedSpan| {
        ast::IntLit::from_str_radix(&v.fragment().to_string(), 16).unwrap()
    });
    let dec_num = map(digit1, |v: LocatedSpan| {
        ast::IntLit::from_str_radix(&v.fragment().to_string(), 10).unwrap()
    });
    let num = alt((hex_num, bin_num, dec_num));
    map(num, |v| Expr::IntLit(v))(input)
}

pub fn ident(input: LocatedSpan) -> PIResult<LocatedSpan> {
    let first = verify(anychar, |c| c.is_ascii_alphabetic() || *c == '_');
    let rest = take_while(|c: char| c.is_ascii_alphanumeric() || "_-'".contains(c));
    let ident = recognize(preceded(first, rest))(input);
    return ident;
}

fn paren(input: LocatedSpan) -> PIResult<Expr> {
    let paren = delimited(
        char('('),
        expect(expr, "expected expression after `(`"),
        expect(char(')'), "expected `)`"),
    );

    map(paren, |inner| {
        Expr::Paren(Box::new(inner.unwrap_or(Expr::Error)))
    })(input)
}

pub fn call(input: LocatedSpan) -> PIResult<ast::Expr> {
    let call_args = separated_list(char(','), delimited(opt(ws), expr, opt(ws)));
    let call = pair(
        expr,
        delimited(
            expect(char('('), "expected '(' before function call args"),
            call_args,
            expect(char(')'), "expected ')' after function call args"),
        ),
    );
    let x = map(call, |(callee, args)| {
        let y: Vec<Box<ast::Expr>> = args.into_iter().map(|x| Box::from(x)).collect();
        ast::Expr::CallExpr(ast::CallExpr::new(Box::from(callee), y))
    })(input);
    return x;
}

fn error(input: LocatedSpan) -> PIResult<Expr> {
    map(take_till1(|c| c == ')'), |span: LocatedSpan| {
        let err = Error(
            span.to_range(),
            format!("expected expression, instead got '{}'", span.fragment()),
        );
        span.extra.report_error(err);
        Expr::Error
    })(input)
}

// use super::*;

// pub fn type_expr(input: &str) -> PIResult<ast::Expr> {
//     let res = alt((
//         map(tag("i64"), |_| {
//             ast::Expr::PrimitiveType(ast::PrimitiveType::new(ast::PrimitiveKind::I64))
//         }),
//         map(tag("u64"), |_| {
//             ast::Expr::PrimitiveType(ast::PrimitiveType::new(ast::PrimitiveKind::U64))
//         }),
//         map(tag("i32"), |_| {
//             ast::Expr::PrimitiveType(ast::PrimitiveType::new(ast::PrimitiveKind::I32))
//         }),
//         map(tag("u32"), |_| {
//             ast::Expr::PrimitiveType(ast::PrimitiveType::new(ast::PrimitiveKind::U32))
//         }),
//         map(ident, |x| ast::Expr::Ident(ast::Ident::from(x))),
//     ))(input);
//     return res;
// }

// pub fn expr(input: &str) -> PIResult<ast::Expr> {
//     let (mut rest, mut curr) = atom(input)?;

//     match pair(
//         delimited(
//             opt(ws),
//             alt((
//                 tag("=="),
//                 tag("!="),
//                 tag("+"),
//                 tag("-"),
//                 tag("*"),
//                 tag("/"),
//                 tag("::"),
//             )),
//             opt(ws),
//         ),
//         atom,
//     )(rest)
//     {
//         Ok((r, (op, result))) => {
//             let binop = match op {
//                 "==" => ast::OpKind::CmpEq,
//                 "!=" => ast::OpKind::CmpNEq,
//                 "+" => ast::OpKind::Plus,
//                 "-" => ast::OpKind::Minus,
//                 "*" => ast::OpKind::Asterisk,
//                 "/" => ast::OpKind::Slash,
//                 "::" => ast::OpKind::Doublecolon,
//                 _ => ast::OpKind::Illegal,
//             };
//             rest = r;
//             curr = ast::Expr::BinOp(ast::BinOp::new(
//                 Box::from(curr.clone()),
//                 binop,
//                 Box::from(result),
//             ));
//         }
//         Err(_) => (),
//     };

//     loop {
//         match pair(
//             delimited(opt(ws), alt((tag("&&"), tag("||"))), opt(ws)),
//             expr,
//         )(rest)
//         {
//             Ok((r, (op, result))) => {
//                 let binop = match op {
//                     "&&" => ast::OpKind::And,
//                     "||" => ast::OpKind::Or,
//                     _ => ast::OpKind::Illegal,
//                 };
//                 rest = r;
//                 curr = ast::Expr::BinOp(ast::BinOp::new(
//                     Box::from(curr.clone()),
//                     binop,
//                     Box::from(result),
//                 ));
//             }
//             Err(_) => break,
//         }
//     }

//     return Ok((rest, curr));
// }

// fn atom(input: &str) -> PIResult<ast::Expr> {
//     let x = alt((
//         map(float_lit, ast::Expr::FloatLit),
//         map(int_lit, ast::Expr::IntLit),
//         map(utils::ident, |v| ast::Expr::Ident(ast::Ident::from(v))),
//     ))(input);
//     return x;
// }

// fn int_lit(input: &str) -> PIResult<ast::IntLit> {
//     let x = alt((
//         map(preceded(tag("0x"), hex_digit1), |v| {
//             ast::IntLit::from_str_radix(v, 16).unwrap()
//         }),
//         map(preceded(tag("0b"), is_a("01")), |v| {
//             ast::IntLit::from_str_radix(v, 2).unwrap()
//         }),
//         map(digit1, |v| ast::IntLit::from_str_radix(v, 10).unwrap()),
//     ))(input);
//     return x;
// }

// fn float_lit(input: &str) -> PIResult<ast::FloatLit> {
//     map(
//         tuple((
//             opt(char('-')),
//             many1(digit1),
//             preceded(char('.'), many1(digit1)),
//         )),
//         |(sign, left, right): (Option<char>, Vec<&str>, Vec<&str>)| {
//             let sign_correction_val: f64 = match sign {
//                 Some('-') => -1.0,
//                 _ => 1.0,
//             };
//             let full_float_str = left[0].to_owned() + "." + right[0];
//             let full_float_val: f64 = full_float_str.parse::<f64>().unwrap();
//             return ast::FloatLit::from(full_float_val * sign_correction_val);
//         },
//     )(input)
// }
