use std::process::exit;

use owo_colors::OwoColorize;

pub fn ice(msg: &str) -> ! {
    let b = std::backtrace::Backtrace::capture();
    eprintln!("{}: {msg}", "internal compiler error".red());
    eprintln!("backtrace: {}", b);
    exit(1);
}
