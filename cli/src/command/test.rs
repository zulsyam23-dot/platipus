use std::path::Path;
use std::process::{Command as Process, ExitCode};

use crate::command::build;
use crate::config::Options;
use crate::CliError;

/// Builds the program, then runs the `test` blocks it declares.
///
/// The runner is JavaScript that needs Node, so it is kept out of `cargo test`:
/// the Rust suites test the compiler, and this command tests a compiled program.
pub fn run(options: &Options) -> Result<ExitCode, CliError> {
    let code = build::run(options)?;
    if code != ExitCode::SUCCESS {
        return Ok(code);
    }
    run_node(&options.out_dir.join("tests.mjs"))
}

pub fn run_node(runner: &Path) -> Result<ExitCode, CliError> {
    let status = Process::new("node")
        .arg(runner)
        .status()
        .map_err(|error| {
            CliError::Io(format!(
                "cannot run `node {}`: {error}; a test run needs Node on the PATH",
                runner.display()
            ))
        })?;
    if status.success() {
        return Ok(ExitCode::SUCCESS);
    }
    // A failing test is not a compiler failure, so the exit code is passed
    // through rather than reported as an error.
    Ok(ExitCode::from(status.code().unwrap_or(1) as u8))
}
