#![feature(trait_upcasting)]

mod diagnostic;
mod reporting;

use std::fmt::Display;

pub use diagnostic::*;
pub use reporting::*;

pub fn ice(msg: impl Display) -> ! {
    panic!("{msg}")
}

#[salsa::jar(db = Db)]
pub struct Jar(crate::Diagnostics);

pub trait Db: salsa::DbWithJar<Jar> + flux_span::Db {}

impl<DB> Db for DB where DB: ?Sized + salsa::DbWithJar<Jar> + flux_span::Db {}
