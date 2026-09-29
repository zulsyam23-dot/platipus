use std::process::ExitCode;

use crate::command::build;
use crate::config::Options;
use crate::CliError;

/// Builds and then says how to serve the result. It does not serve anything
/// itself; `dev` is the command that does.
pub fn run(options: &Options) -> Result<ExitCode, CliError> {
    let code = build::run(options)?;
    if !options.quiet {
        println!();
        println!(
            "serve {} with any static file server",
            options.out_dir.display()
        );
        println!("or run `dev` to serve it and rebuild on every change");
    }
    Ok(code)
}
