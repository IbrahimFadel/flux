use super::*;

pub fn ws(input: &str) -> IResult<&str, &str> {
    recognize(many1_count(alt((
        multispace1,
        line_comment,
        multiline_comment,
    ))))(input)
}

pub fn line_comment(input: &str) -> IResult<&str, &str> {
    preceded(tag("//"), not_line_ending)(input)
}

pub fn multiline_comment(input: &str) -> IResult<&str, &str> {
    fn multiline_inner(input: &str) -> IResult<&str, ()> {
        let mut chars = input.chars();
        let mut rest = input;
        while let Some(c) = chars.next() {
            if c == '*' {
                if let Some('/') = chars.next() {
                    break;
                }
            }
            if c == '/' {
                if let Some('*') = chars.next() {
                    let (after_comment, _) = multiline_comment(rest)?;
                    chars = after_comment.chars();
                }
            }
            rest = chars.as_str();
        }
        Ok((rest, ()))
    }
    delimited(tag("/*"), recognize(multiline_inner), tag("*/"))(input)
}

pub fn ident(input: &str) -> IResult<&str, &str> {
    verify(is_not(token::SPECIAL_CHARACTERS), |ident: &str| {
        !token::is_keyword(ident)
    })(input)
}
