use colored::Colorize;
use itertools::Itertools;

use crate::DiagnosticCode;

/// IO Errors have no files and therefore we can't use ariadne for them
#[derive(Debug)]
pub struct IOError {
    code: DiagnosticCode,
    msg: String,
    help: Vec<String>,
}

impl IOError {
    pub fn new(code: DiagnosticCode, msg: String, help: Vec<String>) -> Self {
        Self { code, msg, help }
    }

    pub fn to_string(self) -> String {
        format!(
            "{} {}\n\t{}",
            format!("[{}]", self.code).red(),
            self.msg.red().bold(),
            self.help.iter().map(|s| s.blue()).join("\n\t")
        )
    }

    pub fn report(self) {
        eprintln!("{}", self.to_string());
    }
}
