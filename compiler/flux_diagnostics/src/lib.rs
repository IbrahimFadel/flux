mod diagnostic;
pub mod fmt;
mod io;
mod reporting;

use std::fmt::Display;

use colored::Colorize;
pub use diagnostic::*;
pub use io::IOError;
pub use reporting::*;

pub fn ice(msg: impl Display) -> ! {
    panic!(
        "{}\n\t{}",
        "internal compiler error:".red(),
        format!("{}", msg).blue()
    )
}
