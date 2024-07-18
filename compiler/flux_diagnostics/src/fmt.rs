use itertools::Itertools;
use num_traits::AsPrimitive;

pub trait Plural {
    fn plural(&self, suffix: &'static str) -> &str;
    fn singular(&self, suffix: &'static str) -> &str;
}

pub trait NthSuffix {
    fn nth_suffix(self) -> &'static str;
}

impl Plural for usize {
    fn plural(&self, suffix: &'static str) -> &str {
        if *self == 1 {
            ""
        } else {
            suffix
        }
    }

    fn singular(&self, suffix: &'static str) -> &str {
        if *self == 1 {
            suffix
        } else {
            ""
        }
    }
}

impl<T> Plural for Vec<T> {
    fn plural(&self, suffix: &'static str) -> &str {
        if self.len() == 1 {
            ""
        } else {
            suffix
        }
    }

    fn singular(&self, suffix: &'static str) -> &str {
        if self.len() == 1 {
            suffix
        } else {
            ""
        }
    }
}

impl<T> NthSuffix for T
where
    T: AsPrimitive<i64>,
{
    fn nth_suffix(self) -> &'static str {
        match self.as_() {
            1 => "st",
            2 => "nd",
            3 => "rd",
            _ => "th",
        }
    }
}

pub fn quote_and_listify<S>(iter: impl Iterator<Item = S>) -> String
where
    S: Into<String>,
{
    iter.map(|item| format!("`{}`", item.into())).join(", ")
}
