mod diagnostic;
pub mod fmt;
mod io;
mod reporting;

use std::{
    backtrace::Backtrace,
    env,
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
    let print_stack_trace = env::var("FLUX_BACKTRACE").map_or(false, |val| {
        val.parse::<u8>().map_or(false, |v| match v {
            0 => false,
            1 => true,
            _ => false,
        })
    });

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
        "{}\n| {} |\n|\t{} |\n{}\n{}\n{}",
        box_top.clone(),
        ice.red(),
        formatted_msg.blue(),
        box_top,
        "Exiting...".black(),
        if print_stack_trace {
            format!("Stack Trace:\n{}", Backtrace::force_capture())
        } else {
            String::new()
        }
    );
    exit(0)
}
