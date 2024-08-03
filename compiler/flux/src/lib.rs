use std::{ffi::OsString, fs, path::Path, sync::OnceLock};

use cfg::{Config, CFG_FILE_NAME};
use clap::{Parser, Subcommand};
use commands::build;
use diagnostics::DriverError;
use flux_diagnostics::IOError;
use flux_util::Interner;

mod cfg;
mod commands;
mod diagnostics;
mod driver;

static INTERNER: OnceLock<Interner> = OnceLock::new();

pub const PRE_INTERNED_VALUES: [&'static str; 14] = [
    "s64", "s32", "s16", "s8", "u64", "u32", "u16", "u8", "f64", "f32", "str", "bool", "true",
    "false",
];

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

pub fn get_config(project_root: &Path) -> Result<Config, IOError> {
    let cfg_path = project_root.join(CFG_FILE_NAME);
    let content = fs::read_to_string(&cfg_path).map_err(|_error| {
        DriverError::ReadConfigFile {
            candidate: cfg_path.to_str().unwrap().to_string(),
        }
        .to_io_error()
    })?;
    Ok(cfg::parse_cfg(&content))
}

pub fn get_package_entry_file_path(
    package_root: &Path,
    package_name: &str,
) -> Result<(String, String), IOError> {
    let file_path = package_root.join("src/main.flx");
    std::fs::read_to_string(&file_path)
        .map_err(|_error| {
            DriverError::ReadEntryFile {
                package: package_name.to_string(),
                candidate: file_path.to_str().unwrap().to_string(),
            }
            .to_io_error()
        })
        .and_then(|content| Ok((file_path.to_str().unwrap().to_string(), content)))
}
