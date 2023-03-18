mod diagnostic;
pub mod reporting;
pub use diagnostic::*;
mod ice;
pub use ice::ice;
use itertools::Itertools;

pub trait Plural {
    fn plural(&self, suffix: &'static str) -> &str;
    fn singular(&self, suffix: &'static str) -> &str;
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

pub fn quote_and_listify<S>(iter: impl Iterator<Item = S>) -> String
where
    S: Into<String>,
{
    iter.map(|item| format!("`{}`", item.into())).join(", ")
}
