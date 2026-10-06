//! One module per command. The enum is the only place that knows the whole set.

pub mod build;
pub mod check;
pub mod dev;
pub mod format;
pub mod new;
pub mod run;
pub mod test;

use std::process::ExitCode;

use crate::config::Options;
use crate::CliError;

/// Whether the Rust stage should type-check or fully build.
#[derive(Debug, Clone, Copy)]
pub enum RustMode {
    Check,
    Build,
}

impl From<RustMode> for platipus_compiler::rust::RustMode {
    fn from(mode: RustMode) -> Self {
        match mode {
            RustMode::Check => platipus_compiler::rust::RustMode::Check,
            RustMode::Build => platipus_compiler::rust::RustMode::Build,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Command {
    New,
    Build,
    Check,
    Run,
    Test,
    Dev,
    Format,
    Help,
    Version,
}

impl Command {
    pub fn from_word(word: Option<&str>) -> Option<Self> {
        Some(match word? {
            "new" => Command::New,
            "build" => Command::Build,
            "check" => Command::Check,
            "run" => Command::Run,
            "test" => Command::Test,
            "dev" => Command::Dev,
            "format" => Command::Format,
            "help" => Command::Help,
            "version" => Command::Version,
            _ => return None,
        })
    }

    pub const fn word(self) -> &'static str {
        match self {
            Command::New => "new",
            Command::Build => "build",
            Command::Check => "check",
            Command::Run => "run",
            Command::Test => "test",
            Command::Dev => "dev",
            Command::Format => "format",
            Command::Help => "help",
            Command::Version => "version",
        }
    }
}

pub fn dispatch(command: Command, options: &Options) -> Result<ExitCode, CliError> {
    match command {
        Command::New => new::run(options),
        Command::Build => build::run(options),
        Command::Check => check::run(options),
        Command::Run => run::run(options),
        Command::Test => test::run(options),
        Command::Dev => dev::run(options),
        Command::Format => format::run(options),
        Command::Help | Command::Version => unreachable!("handled before dispatch"),
    }
}
