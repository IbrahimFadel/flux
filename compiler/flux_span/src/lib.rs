mod build;
mod input_file;
mod span;
mod word;

pub use build::*;
pub use input_file::*;
pub use span::*;
pub use word::*;

#[salsa::jar(db = Db)]
pub struct Jar(crate::word::Word, crate::InputFile);

pub trait Db: salsa::DbWithJar<Jar> {}

impl<DB> Db for DB where DB: ?Sized + salsa::DbWithJar<Jar> {}
