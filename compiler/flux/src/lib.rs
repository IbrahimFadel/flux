use std::ffi::OsString;

use clap::{Parser, Subcommand};
use commands::build;

mod commands;
mod diagnostics;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExitStatus {
    Success,
    Failure,
}

#[derive(Parser, Debug)]
#[command(name = "flux")]
#[command(author = "Ibrahim F. <ibrahim.m.fadel@gmail.com>")]
#[command(version = "1.0")]
#[command(about = "The Flux Compiler", long_about = None)]
#[command(propagate_version = true)]
struct Args {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Build
    ///
    /// Build a flux project without running it
    Build(build::Args),
}

pub fn run_with_args<T, I>(args: I) -> ExitStatus
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    let args = Args::parse_from(args);
    match args.command {
        Command::Build(args) => build::build(args),
    }
}
