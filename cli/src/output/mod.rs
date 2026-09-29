//! What the CLI writes: artifacts to disk and diagnostics to a stream.

pub mod diagnostics;

use std::fs;
use std::path::Path;

use platipus_compiler::codegen::Artifact;
use platipus_compiler::diagnostics::Warning;

use crate::CliError;

pub fn read_entry(entry: &Path) -> Result<String, CliError> {
    if !entry.exists() {
        return Err(CliError::Io(format!(
            "cannot find `{}`",
            entry.display()
        )));
    }
    fs::read_to_string(entry)
        .map_err(|error| CliError::Io(format!("cannot read `{}`: {error}", entry.display())))
}

pub fn write_artifacts(out_dir: &Path, artifacts: &[Artifact]) -> Result<Vec<std::path::PathBuf>, CliError> {
    fs::create_dir_all(out_dir)
        .map_err(|error| CliError::Write(format!("cannot create `{}`: {error}", out_dir.display())))?;
    let mut written = Vec::new();
    for artifact in artifacts {
        let path = out_dir.join(artifact.name);
        fs::write(&path, &artifact.contents).map_err(|error| {
            CliError::Write(format!("cannot write `{}`: {error}", path.display()))
        })?;
        written.push(path);
    }
    Ok(written)
}

pub fn path_label(entry: &Path) -> String {
    entry.to_string_lossy().to_string()
}

/// Warnings are reported for their own sake, so they go to stdout and do not
/// decide the exit code.
pub fn report(file: &platipus_compiler::diagnostics::SourceFile, warnings: &[Warning]) {
    for warning in warnings {
        println!("warning: {}", warning.label());
        if let Some(span) = warning.span {
            println!("  --> {}", file.describe(span));
        }
        if let Some(help) = &warning.help {
            println!("   = help: {help}");
        }
    }
}
