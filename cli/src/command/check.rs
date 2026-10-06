use std::process::ExitCode;

use crate::command::build::compile_entry;
use crate::config::Options;
use crate::output::report;
use crate::CliError;

pub fn run(options: &Options) -> Result<ExitCode, CliError> {
    let built = compile_entry(&options.entry, crate::command::RustMode::Check)?;
    report(&built.file, &built.compilation.warnings);
    if !options.quiet {
        println!(
            "checked {}: {} components, {} apis",
            built.label,
            built.compilation.module.components.len(),
            built.compilation.module.apis.len()
        );
    }
    Ok(ExitCode::SUCCESS)
}
