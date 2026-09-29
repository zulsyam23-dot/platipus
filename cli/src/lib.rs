//! The `platipus` / `plt` command line interface.
//!
//! The layout follows `docs/struktur.md`: `config` parses what was asked for,
//! `output` decides what is written, and `command` holds one module per command.

pub mod command;
pub mod config;
pub mod output;

use std::path::Path;
use std::process::ExitCode;

use crate::command::Command;
use platipus_compiler::diagnostics::CompileError;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CliError {
    Usage(String),
    Io(String),
    Compile(CompileError),
    Write(String),
}

impl std::fmt::Display for CliError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CliError::Usage(message) => write!(f, "{message}"),
            CliError::Io(message) => write!(f, "{message}"),
            CliError::Compile(error) => write!(f, "{error}"),
            CliError::Write(message) => write!(f, "{message}"),
        }
    }
}

pub fn run_from_env() -> ExitCode {
    let mut args: Vec<String> = std::env::args().collect();
    let program = args
        .first()
        .map(|name| {
            Path::new(name)
                .file_stem()
                .map(|stem| stem.to_string_lossy().to_string())
                .unwrap_or_else(|| "platipus".to_string())
        })
        .unwrap_or_else(|| "platipus".to_string());
    if args.is_empty() {
        args.push(program.clone());
    }
    args.remove(0);
    match run(&program, &args) {
        Ok(code) => code,
        Err(error) => {
            eprintln!("{program}: {error}");
            match error {
                CliError::Usage(_) => ExitCode::from(2),
                _ => ExitCode::FAILURE,
            }
        }
    }
}

pub fn run(program: &str, args: &[String]) -> Result<ExitCode, CliError> {
    let options = config::parse(program, args)?;
    match options.command {
        Command::Help => {
            print!("{}", config::usage(program, VERSION));
            Ok(ExitCode::SUCCESS)
        }
        Command::Version => {
            println!("{program} {VERSION}");
            Ok(ExitCode::SUCCESS)
        }
        other => command::dispatch(other, &options),
    }
}
