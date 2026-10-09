//! Resolving `import Name from "./module.plt"`.
//!
//! The compiler does not touch the filesystem itself; code that wants imports
//! hands in a [`Loader`]. The loader reads each imported file, validates it as
//! an app-less library module, and returns the sources concatenated in load
//! order so the ordinary single-file pipeline can compile them once. Spans stay
//! correct because every later stage sees one buffer whose bytes are the module
//! texts in order.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use crate::ast::Program;
use crate::diagnostics::{DiagnosticBag, Error, ErrorKind, Span};
use crate::parser;

/// Why reading one imported file failed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReadError {
    NotFound,
    Io(String),
}

impl ReadError {
    pub fn is_not_found(&self) -> bool {
        matches!(self, ReadError::NotFound)
    }
}

/// The one place a compile meets the filesystem.
pub trait Loader {
    /// Reads the module at `path` and reports **which file it turned out to be**.
    ///
    /// The second half of the answer is not optional bookkeeping. A relative
    /// import inside a module means "next to me", and once a loader is allowed
    /// to answer a path with a different file -- the Library Store answering
    /// `computasi` with `<store>/packages/computasi/src/lib.plt`, say -- the
    /// path the caller asked for is no longer the file the module lives in.
    /// Reporting the real path is what lets the loader resolve that module's own
    /// sibling imports beside the module rather than beside whichever file
    /// happened to import it.
    fn read(&self, path: &str) -> Result<Read, ReadError>;
}

/// A module that was read, and where it actually came from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Read {
    /// The file the source was read from, which is what later relative imports
    /// inside that source are resolved against.
    pub path: String,
    pub source: String,
}

/// A loader that reads exactly the path it is given, relative-paths resolved
/// by the caller against the importing file.
pub struct FsLoader;

impl Loader for FsLoader {
    fn read(&self, path: &str) -> Result<Read, ReadError> {
        match std::fs::read_to_string(path) {
            Ok(source) => Ok(Read {
                path: path.to_string(),
                source,
            }),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Err(ReadError::NotFound),
            Err(error) => Err(ReadError::Io(error.to_string())),
        }
    }
}

/// Every file of a program, merged into one buffer with its dependencies.
#[derive(Debug, Clone)]
pub struct Combined {
    /// The label of the whole program: the entry path.
    pub label: String,
    /// Entry source first, then each imported module's source, newline-separated.
    pub source: String,
    /// How many module files were read (0 when the entry imports nothing).
    pub modules: usize,
    /// One resolved path per file that took part, entry first.
    pub dependencies: Vec<String>,
}

/// A failure while resolving imports. The diagnostic's spans point into
/// `source`, so the caller turns these three into a `SourceFile` and renders.
#[derive(Debug, Clone)]
pub struct ModuleFailure {
    pub label: String,
    pub source: String,
    pub bag: DiagnosticBag,
}

/// Resolves `import` transitively from the entry and combines the sources.
///
/// On success the returned source can be fed straight to the pipeline; every
/// `import` in the text is satisfied by a corresponding module in the buffer.
/// On failure the caller receives the file that caused it.
pub fn resolve_and_combine(
    entry_path: &str,
    entry_source: &str,
    loader: &dyn Loader,
) -> Result<Combined, ModuleFailure> {
    let mut discovery = Discovery::new(loader);
    discovery.files.push(entry_path.to_string());
    discovery.sources.push(entry_source.to_string());

    let entry = PathBuf::from(entry_path);
    let entry_key = std::fs::canonicalize(&entry)
        .unwrap_or(entry)
        .to_string_lossy()
        .to_string();
    discovery.dependencies.push(entry_key.clone());
    discovery.seen.insert(entry_key.clone());
    discovery.active.push(entry_key);

    let program = parser::parse(entry_path.to_string(), entry_source).program;
    for (name, _) in declarations(&program) {
        discovery.declared.insert(name);
    }
    discovery.load_imports(0, &program)?;

    let mut parts = String::with_capacity(entry_source.len() + 1);
    for (index, source) in discovery.sources.iter().enumerate() {
        if index > 0 {
            parts.push('\n');
        }
        // A BOM mid-buffer would trip the lexer; each part is a fresh file
        // decoded by its own reader, so the mark is per-file baggage.
        parts.push_str(source.strip_prefix('\u{feff}').unwrap_or(source));
    }
    Ok(Combined {
        label: entry_path.to_string(),
        source: parts,
        modules: discovery.files.len() - 1,
        dependencies: discovery.dependencies,
    })
}

/// Traverses the import graph one file at a time.
struct Discovery<'l> {
    loader: &'l dyn Loader,
    files: Vec<String>,
    sources: Vec<String>,
    dependencies: Vec<String>,
    labels: HashSet<String>,
    declared: HashSet<String>,
    active: Vec<String>,
    seen: HashSet<String>,
}

impl<'l> Discovery<'l> {
    fn new(loader: &'l dyn Loader) -> Self {
        Self {
            loader,
            files: Vec::new(),
            sources: Vec::new(),
            dependencies: Vec::new(),
            labels: HashSet::new(),
            declared: HashSet::new(),
            active: Vec::new(),
            seen: HashSet::new(),
        }
    }

    /// Reads everything the file at `index` imports.
    fn load_imports(&mut self, index: usize, program: &Program) -> Result<(), ModuleFailure> {
        for import in &program.imports {
            // An empty path is a fixture error caught by the IR verifier on the
            // combined program; there is nothing to read for it.
            if import.path.trim().is_empty() {
                continue;
            }
            self.import_file(index, import)?;
        }
        Ok(())
    }

    /// Reads, validates, and registers one `import` on behalf of `importer`.
    fn import_file(
        &mut self,
        importer: usize,
        import: &crate::ast::ImportDecl,
    ) -> Result<(), ModuleFailure> {
        let label = import.name.as_str().to_string();
        if !self.labels.insert(label.clone()) {
            return Err(self.fail(
                importer,
                Error::new(
                    ErrorKind::Semantic,
                    "duplicate-import",
                    format!("`{label}` is already imported"),
                )
                .with_span(import.span),
            ));
        }

        let importer_path = Path::new(&self.files[importer]);
        let base = importer_path.parent().unwrap_or_else(|| Path::new("."));
        let requested = base.join(&import.path);
        let read = match self.loader.read(&requested.to_string_lossy()) {
            Ok(read) => read,
            Err(ReadError::NotFound) => {
                return Err(self.fail(
                    importer,
                    Error::new(
                        ErrorKind::Semantic,
                        "import-not-found",
                        format!("cannot find `{}` imported as `{label}`", import.path),
                    )
                    .with_span(import.span),
                ));
            }
            Err(ReadError::Io(message)) => {
                return Err(self.fail(
                    importer,
                    Error::new(
                        ErrorKind::Semantic,
                        "import-read-error",
                        format!(
                            "cannot read `{}` imported as `{label}`: {message}",
                            import.path
                        ),
                    )
                    .with_span(import.span),
                ));
            }
        };
        // The loader may have answered a package name rather than a path, so
        // everything below works from the file it says it read, not from the one
        // that was asked for.
        let key = std::fs::canonicalize(&read.path)
            .unwrap_or_else(|_| PathBuf::from(&read.path))
            .to_string_lossy()
            .to_string();
        let source = read.source;

        if self.active.contains(&key) {
            return Err(self.fail(
                importer,
                Error::new(
                    ErrorKind::Semantic,
                    "import-cycle",
                    format!(
                        "`{label}` imports `{}`, which is already being loaded",
                        import.path
                    ),
                )
                .with_span(import.span),
            ));
        }
        if self.seen.contains(&key) {
            return Err(self.fail(
                importer,
                Error::new(
                    ErrorKind::Semantic,
                    "duplicate-module",
                    format!(
                        "`{}` is imported more than once (as `{label}`)",
                        import.path
                    ),
                )
                .with_span(import.span),
            ));
        }

        // A module is a library: no `app`, and no declaration name that the
        // program already owns. Parse errors in the module are fatal: merging
        // a partially parsed module would hide the actual problem.
        let outcome = parser::parse(key.clone(), &source);
        if !outcome.errors.is_empty() {
            return Err(self.fail_owned(
                key,
                source,
                outcome.errors[0].clone(),
            ));
        }
        let module = outcome.program;
        if let Some(app) = &module.app {
            return Err(self.fail_owned(
                key,
                source,
                Error::new(
                    ErrorKind::Semantic,
                    "import-module-has-app",
                    "an imported file must be an app-less library, but this one declares an `app`",
                )
                .with_span(app.span),
            ));
        }
        for (name, span) in declarations(&module) {
            if !self.declared.insert(name.clone()) {
                return Err(self.fail_owned(
                    key,
                    source,
                    Error::new(
                        ErrorKind::Semantic,
                        "import-collision",
                        format!("`{name}` is already declared in the program"),
                    )
                    .with_span(span),
                ));
            }
        }

        self.files.push(key.clone());
        self.sources.push(source);
        self.dependencies.push(key.clone());
        self.seen.insert(key.clone());
        self.active.push(key);
        let index = self.files.len() - 1;
        let result = self.load_imports(index, &module);
        self.active.pop();
        result
    }

    fn fail(&self, index: usize, error: Error) -> ModuleFailure {
        self.fail_owned(
            self.files[index].clone(),
            self.sources[index].clone(),
            error,
        )
    }

    fn fail_owned(&self, label: String, source: String, error: Error) -> ModuleFailure {
        let mut bag = DiagnosticBag::new();
        bag.error(error);
        ModuleFailure { label, source, bag }
    }
}

/// The names a program makes visible, paired with the span that declares them.
fn declarations(program: &Program) -> Vec<(String, Span)> {
    let mut out = Vec::new();
    if let Some(app) = &program.app {
        out.push((app.name.as_str().to_string(), app.span));
    }
    out.extend(
        program
            .components
            .iter()
            .map(|item| (item.name.as_str().to_string(), item.span)),
    );
    out.extend(
        program
            .styles
            .iter()
            .map(|item| (item.name.as_str().to_string(), item.span)),
    );
    out.extend(
        program
            .themes
            .iter()
            .map(|item| (item.name.as_str().to_string(), item.span)),
    );
    out.extend(
        program
            .apis
            .iter()
            .map(|item| (item.name.as_str().to_string(), item.span)),
    );
    out.extend(
        program
            .tests
            .iter()
            .map(|item| (item.name.as_str().to_string(), item.span)),
    );
    out
}
