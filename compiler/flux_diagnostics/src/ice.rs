use std::process::exit;

use owo_colors::OwoColorize;

pub fn ice(msg: &str) -> ! {
    eprintln!("{}: {msg}", "internal compiler error".red());
    exit(1);
}
