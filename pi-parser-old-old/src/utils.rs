use super::*;

pub fn ws(input: LocatedSpan) -> PIResult<LocatedSpan> {
    recognize(many1_count(alt((
        multispace1,
        line_comment,
        // multiline_comment,
    ))))(input)
}

pub fn line_comment(input: LocatedSpan) -> PIResult<LocatedSpan> {
    preceded(tag("//"), not_line_ending)(input)
}

// pub fn multiline_comment(input: LocatedSpan) -> PIResult<LocatedSpan> {
//     fn multiline_inner(input: LocatedSpan) -> PIResult<()> {
//         let mut chars = input.chars();
//         let mut rest = input;
//         while let Some(c) = chars.next() {
//             if c == '*' {
//                 if let Some('/') = chars.next() {
//                     break;
//                 }
//             }
//             if c == '/' {
//                 if let Some('*') = chars.next() {
//                     let (after_comment, _) = multiline_comment(rest)?;
//                     chars = after_comment.chars();
//                 }
//             }
//             // rest = chars;
//             rest = LocatedSpan::new_extra(chars.as_str(), rest.extra);
//             // rest.program = chars.as_str();
//         }
//         // Ok((rest, ()))
//         Ok((rest, ()))
//     }
//     delimited(tag("/*"), recognize(multiline_inner), tag("*/"))(input)
// }

// pub fn ident(input: &str) -> PIResult<&str> {
//     verify(is_not(token::SPECIAL_CHARACTERS), |ident: &str| {
//         !token::is_keyword(ident)
//     })(input)
// }
