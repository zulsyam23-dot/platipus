use platipus_compiler::diagnostics::SourceFile;
use platipus_compiler::pipeline;

fn main() -> std::process::ExitCode {
    let mut args = std::env::args().skip(1);
    let Some(path) = args.next() else {
        eprintln!("usage: compile <entry.plt> [out-dir]");
        return std::process::ExitCode::from(2);
    };
    let out = args.next().unwrap_or_else(|| "dist".to_string());
    let source = match std::fs::read_to_string(&path) {
        Ok(source) => source,
        Err(error) => {
            eprintln!("cannot read `{path}`: {error}");
            return std::process::ExitCode::FAILURE;
        }
    };
    let file = SourceFile::new(&path, source.clone());
    match pipeline::build(&source) {
        Ok(compilation) => {
            for artifact in &compilation.artifacts {
                println!("{}: {} bytes", artifact.name, artifact.contents.len());
            }
            match std::fs::create_dir_all(&out) {
                Ok(()) => {}
                Err(error) => {
                    eprintln!("cannot create `{out}`: {error}");
                    return std::process::ExitCode::FAILURE;
                }
            }
            for artifact in &compilation.artifacts {
                let target = std::path::Path::new(&out).join(artifact.name);
                if let Err(error) = std::fs::write(&target, &artifact.contents) {
                    eprintln!("cannot write `{}`: {error}", target.display());
                    return std::process::ExitCode::FAILURE;
                }
            }
            std::process::ExitCode::SUCCESS
        }
        Err(bag) => {
            for diagnostic in bag.sorted() {
                match diagnostic {
                    platipus_compiler::diagnostics::Diagnostic::Error(error) => {
                        eprintln!("error: {}", error.label());
                        if let Some(span) = error.span {
                            eprintln!("  --> {}", file.describe(span));
                            eprintln!("{}", file.caret_line(span));
                        }
                        if let Some(help) = &error.help {
                            eprintln!("   = help: {help}");
                        }
                    }
                    platipus_compiler::diagnostics::Diagnostic::Warning(warning) => {
                        eprintln!("warning: {}", warning.label());
                    }
                }
            }
            std::process::ExitCode::FAILURE
        }
    }
}
