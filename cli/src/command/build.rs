use std::path::{Path, PathBuf};
use std::process::ExitCode;

use platipus_compiler::codegen::Web;
use platipus_compiler::diagnostics::{CompileError, SourceFile};
use platipus_compiler::loader::FsLoader;
use platipus_compiler::pipeline::{self, Compilation, EntryFailure};

use crate::config::Options;
use crate::output::{diagnostics, path_label, read_entry, report, write_artifacts};
use crate::CliError;

/// A successful compile plus what a caller needs to report on it: the buffer
/// the diagnostics point into, and the files that took part (which `dev`
/// watches).
pub struct Built {
    pub compilation: Compilation,
    pub file: SourceFile,
    pub label: String,
    pub dependencies: Vec<PathBuf>,
}

/// Compiles the entry file and everything it imports through every stage,
/// turning a diagnostic bag into a rendered report.
///
/// `check` and `build` both need this, and so does `dev` on every rebuild.
pub fn compile_entry(entry: &Path, rust_mode: crate::command::RustMode) -> Result<Built, CliError> {
    let source = read_entry(entry)?;
    let label = path_label(entry);
    let workdir = entry
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));
    let rust = pipeline::RustBuild {
        workdir,
        mode: rust_mode.into(),
        rust_dependencies: Vec::new(),
    };
    match pipeline::build_entry_rust(&label, &source, &FsLoader, &Web, Some(&rust)) {
        Ok(loaded) => Ok(Built {
            compilation: loaded.compilation,
            file: loaded.file,
            label,
            dependencies: loaded.dependencies,
        }),
        Err(EntryFailure::Module {
            label,
            source,
            bag,
        }) => {
            let file = SourceFile::new(label, source);
            eprint!("{}", diagnostics::render(&file, &bag));
            Err(CliError::Compile(CompileError::from_bag(bag)))
        }
        Err(EntryFailure::Whole { file, bag }) => {
            eprint!("{}", diagnostics::render(&file, &bag));
            Err(CliError::Compile(CompileError::from_bag(bag)))
        }
    }
}

pub fn run(options: &Options) -> Result<ExitCode, CliError> {
    let built = compile_entry(&options.entry, crate::command::RustMode::Build)?;
    let written = write_artifacts(&options.out_dir, &built.compilation.artifacts)?;
    report(&built.file, &built.compilation.warnings);
    if !options.quiet {
        for artifact in &written {
            println!("wrote {}", artifact.display());
        }
        println!("built {} into {}", built.label, options.out_dir.display());
    }
    Ok(ExitCode::SUCCESS)
}