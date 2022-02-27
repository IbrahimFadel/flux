use std::cell::RefCell;
use std::ops::Range;

// https://eyalkalderon.com/blog/nom-error-recovery/

pub type LocatedSpan<'a> = nom_locate::LocatedSpan<&'a str, State<'a>>;

// impl LocatedSpan<'a> {
//     pub fn new(src: &'a str) -> LocatedSpan<'a> {
//         LocatedSpan::new_extra(src, &RefCell::new(vec![]))
//     }
// }

pub type PIResult<'a, T> = nom::IResult<LocatedSpan<'a>, T>;

pub trait ToRange {
    fn to_range(&self) -> Range<usize>;
}

impl<'a> ToRange for LocatedSpan<'a> {
    fn to_range(&self) -> Range<usize> {
        let start = self.location_offset();
        let end = start + self.fragment().len();
        start..end
    }
}

#[derive(Debug)]
pub struct Error(pub Range<usize>, pub String);

#[derive(Clone, Debug, Copy)]
pub struct State<'a>(pub &'a RefCell<Vec<Error>>);

impl<'a> State<'a> {
    pub fn report_error(&self, error: Error) {
        self.0.borrow_mut().push(error);
    }
}

pub fn expect<'a, F, E, T>(
    parser: F,
    error_msg: E,
) -> impl Fn(LocatedSpan<'a>) -> PIResult<Option<T>>
where
    F: Fn(LocatedSpan<'a>) -> PIResult<T>,
    E: ToString,
{
    move |input| match parser(input) {
        Ok((remaining, out)) => Ok((remaining, Some(out))),
        Err(nom::Err::Error((input, _))) | Err(nom::Err::Failure((input, _))) => {
            let err = Error(input.to_range(), error_msg.to_string());
            input.extra.report_error(err);
            Ok((input, None))
        }
        Err(err) => Err(err),
    }
}
