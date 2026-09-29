use std::path::PathBuf;

use crate::ast::Program;
use crate::codegen::{Artifact, Target};
use crate::diagnostics::{DiagnosticBag, Error, ErrorKind, SourceFile, Span, Warning};
use crate::ir::{IrError, IrModule, lower_program, verify};
use crate::loader::{self, Loader};
use crate::parser::parse;
use crate::semantic::SemanticChecker;

pub fn compile(source: &str) -> Result<Program, Box<FailedCompilation>> {
    compile_named("main.plt", source)
}

pub fn compile_named(path: &str, source: &str) -> Result<Program, Box<FailedCompilation>> {
    let outcome = parse(path, source);
    let program = outcome.program;
    let mut bag = DiagnosticBag::new();
    for error in &outcome.errors {
        bag.error(error.clone());
    }
    if bag.has_errors() {
        return Err(Box::new(FailedCompilation {
            program,
            diagnostics: bag,
        }));
    }
    let semantic = SemanticChecker::new().check(&program);
    bag.extend(semantic);
    if bag.has_errors() {
        return Err(Box::new(FailedCompilation {
            program,
            diagnostics: bag,
        }));
    }
    Ok(program)
}

/// A program plus the diagnostics that made it unusable, returned by the
/// convenience `compile` helpers so callers can still inspect partial ASTs.
#[derive(Debug, Clone)]
pub struct FailedCompilation {
    pub program: Program,
    pub diagnostics: DiagnosticBag,
}

/// The result of a full compile, including the artifacts a target produced.
#[derive(Debug, Clone, PartialEq)]
pub struct Compilation {
    pub program: Program,
    pub module: IrModule,
    pub artifacts: Vec<Artifact>,
    pub warnings: Vec<crate::diagnostics::Warning>,
}

/// Runs every compiler stage and a codegen target in order.
pub fn build(source: &str) -> Result<Compilation, DiagnosticBag> {
    build_file("main.plt", source, &crate::codegen::Web)
}

pub fn build_file(
    path: &str,
    source: &str,
    target: &dyn Target,
) -> Result<Compilation, DiagnosticBag> {
    let (program, bag) = analyse(path, source)?;
    imports_without_a_loader(&program)?;
    let warnings = bag.warnings().cloned().collect();
    finish(program, path, warnings, target)
}

/// The result of compiling an entry with its modules: everything `build_file`
/// produces plus the single buffer the diagnostics point into and the list of
/// files that took part (what `dev` watches).
#[derive(Debug, Clone, PartialEq)]
pub struct Loaded {
    pub compilation: Compilation,
    pub file: SourceFile,
    pub dependencies: Vec<PathBuf>,
}

/// Why `build_entry` produced nothing.
#[derive(Debug, Clone)]
pub enum EntryFailure {
    /// A module could not be read or validated; the bag's spans point into
    /// `source`, so the caller renders them against `SourceFile::new(label,
    /// source)`.
    Module {
        label: String,
        source: String,
        bag: DiagnosticBag,
    },
    /// The combined program failed a later stage; spans point into `file`.
    Whole {
        file: SourceFile,
        bag: DiagnosticBag,
    },
}

/// Compiles the entry and every file it imports, transitively, as one program.
pub fn build_entry(
    path: &str,
    source: &str,
    loader: &dyn Loader,
    target: &dyn Target,
) -> Result<Loaded, EntryFailure> {
    let combined = loader::resolve_and_combine(path, source, loader).map_err(|failure| {
        EntryFailure::Module {
            label: failure.label,
            source: failure.source,
            bag: failure.bag,
        }
    })?;
    let file = SourceFile::new(&combined.label, &combined.source);
    let (program, bag) = match analyse(&combined.label, &combined.source) {
        Ok((program, bag)) => (program, bag),
        Err(bag) => return Err(EntryFailure::Whole { file, bag }),
    };
    let warnings = bag.warnings().cloned().collect();
    let compilation = match finish(program, &combined.label, warnings, target) {
        Ok(compilation) => compilation,
        Err(bag) => return Err(EntryFailure::Whole { file, bag }),
    };
    Ok(Loaded {
        compilation,
        file,
        dependencies: combined
            .dependencies
            .into_iter()
            .map(PathBuf::from)
            .collect(),
    })
}

/// Runs the stages after analysis on one program.
fn finish(
    program: Program,
    path: &str,
    warnings: Vec<Warning>,
    target: &dyn Target,
) -> Result<Compilation, DiagnosticBag> {
    let module = match lower_program(&program) {
        Some(module) => module,
        None => return Err(lowering_failure(path)),
    };
    if let Err(errors) = verify(&module) {
        return Err(ir_failure(path, errors));
    }
    let artifacts = match target.generate(&module) {
        Ok(artifacts) => artifacts,
        Err(error) => {
            let mut bag = DiagnosticBag::new();
            bag.error(error.into_error());
            return Err(bag);
        }
    };
    Ok(Compilation {
        program,
        module,
        artifacts,
        warnings,
    })
}

/// The single-file entry points have no loader, so an `import` asked to read a
/// file is refused rather than silently dropped.
fn imports_without_a_loader(program: &Program) -> Result<(), DiagnosticBag> {
    let imports: Vec<_> = program
        .imports
        .iter()
        .filter(|import| !import.path.trim().is_empty())
        .collect();
    if imports.is_empty() {
        return Ok(());
    }
    let mut bag = DiagnosticBag::new();
    for import in imports {
        bag.error(
            Error::new(
                ErrorKind::Semantic,
                "import-not-loaded",
                format!(
                    "`{}` imports `{}`, but this compilation has no file loader",
                    import.name.as_str(),
                    import.path
                ),
            )
            .with_span(import.span)
            .with_help("compile through the CLI or `pipeline::build_entry` with a loader"),
        );
    }
    Err(bag)
}

/// Runs every stage up to and including semantic analysis.
pub fn analyse(path: &str, source: &str) -> Result<(Program, DiagnosticBag), DiagnosticBag> {
    let outcome = parse(path, source);
    let program = outcome.program;
    let mut bag = DiagnosticBag::new();
    for error in &outcome.errors {
        bag.error(error.clone());
    }
    if bag.has_errors() {
        return Err(bag);
    }
    let semantic = SemanticChecker::new().check(&program);
    bag.extend(semantic);
    if bag.has_errors() {
        return Err(bag);
    }
    Ok((program, bag))
}

fn lowering_failure(path: &str) -> DiagnosticBag {
    let mut bag = DiagnosticBag::new();
    bag.error(
        Error::new(
            ErrorKind::Ir,
            "lowering-failed",
            "the program could not be lowered into the intermediate representation",
        )
        .with_span(Span::empty())
        .with_note(crate::diagnostics::Note::at(
            format!("lowering {path} did not produce a module"),
            Span::empty(),
        )),
    );
    bag
}

fn ir_failure(path: &str, errors: Vec<IrError>) -> DiagnosticBag {
    let mut bag = DiagnosticBag::new();
    for error in errors {
        let mut converted = Error::new(ErrorKind::Ir, error.code, error.message);
        if let Some(span) = error.span {
            converted = converted.with_span(span);
        }
        bag.error(converted);
    }
    if !bag.has_errors() {
        bag.error(Error::new(
            ErrorKind::Ir,
            "ir-invalid",
            format!("the intermediate representation of {path} is invalid"),
        ));
    }
    bag
}
