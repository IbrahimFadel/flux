mod diagnostic;
pub mod fmt;
mod io;
mod reporting;

use std::{
    fmt::Display,
    iter::{once, repeat},
    process::exit,
};

use colored::Colorize;
pub use diagnostic::*;
pub use io::IOError;
pub use reporting::*;

const TAB_WIDTH: usize = 8;
const ICE_MSG: &'static str = "internal compiler error:";

pub fn ice(msg: impl Display) -> ! {
    let formatted_msg = format!("{}", msg);
    let box_width = formatted_msg.len() + TAB_WIDTH;
    let box_top = once("+")
        .chain(repeat("-").take(box_width))
        .chain(once("+"))
        .collect::<String>();
    let ice = format!(
        "{ICE_MSG}{}",
        repeat(" ")
            .take(box_width - ICE_MSG.len() - 2)
            .collect::<String>()
    );

    eprintln!(
        "{}\n| {} |\n|\t{} |\n{}\n{}",
        box_top.clone(),
        ice.red(),
        formatted_msg.blue(),
        box_top,
        "Exiting...".black()
    );
    exit(0)
}
