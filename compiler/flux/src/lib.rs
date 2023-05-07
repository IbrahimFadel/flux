use std::ffi::OsString;

use clap::{Parser, Subcommand};
use commands::build;
use lasso::ThreadedRodeo;
use once_cell::sync::Lazy;

mod cfg;
mod commands;
mod diagnostics;
mod driver;

static INTERNER: Lazy<ThreadedRodeo> = Lazy::new(ThreadedRodeo::new);

#[derive(Debug)]
pub enum ExitStatus {
    Success,
    Failure,
}

#[derive(Parser, Debug)]
#[command(name = "Flux")]
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
