//! Command-line handling for the `p2lt` binary.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use crate::manifest::Manifest;
use crate::registry::Registry;
use crate::resolver;

fn usage() -> &'static str {
    "\
p2lt - the Platipus package manager

USAGE:
    p2lt <COMMAND> [ARGS]

COMMANDS:
    init                 create p2lt.toml, src/main.plt, .gitignore
    install <name>       install a package from the local registry
    remove <name>        remove a package and update the manifest + lockfile
    update               refresh all dependencies from the registry
    list                 list installed dependencies
    search <query>       search the registry
    build                compile the project (Platipus + Rust blocks)
    run                  build, then print how to serve the result
    publish              copy this package into the local registry
    login                record a local registry credential stamp
    logout               remove the stored credential stamp
    clean                remove target/ and dist/

The registry is a directory of packages: $P2LT_REGISTRY or
<project>/.p2lt/registry. Set P2LT_REGISTRY to share one registry.
"
}

pub fn run_from_env() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match run(&args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("p2lt: {message}");
            ExitCode::FAILURE
        }
    }
}

pub fn run(args: &[String]) -> Result<(), String> {
    let Some(command) = args.first().map(String::as_str) else {
        print!("{}", usage());
        return Ok(());
    };
    let project = std::env::current_dir().map_err(|e| e.to_string())?;
    match command {
        "init" => init(&project),
        "install" => with_arg(args, "install <name>", |name| {
            resolver::install(&project, name)?;
            println!("installed {name}");
            Ok(())
        }),
        "remove" => with_arg(args, "remove <name>", |name| {
            resolver::remove(&project, name)?;
            println!("removed {name}");
            Ok(())
        }),
        "update" => {
            resolver::update(&project)?;
            println!("updated dependencies");
            Ok(())
        }
        "list" => {
            let installed = resolver::list_installed(&project)?;
            if installed.is_empty() {
                println!("no dependencies");
            } else {
                for name in installed {
                    println!("{name}");
                }
            }
            Ok(())
        }
        "search" => with_arg(args, "search <query>", |query| {
            let registry = Registry::for_project(&project);
            let matches = registry.search(query);
            if matches.is_empty() {
                println!("no packages match `{query}`");
            } else {
                for name in matches {
                    println!("{name}");
                }
            }
            Ok(())
        }),
        "build" => build(&project),
        "run" => {
            build(&project)?;
            println!();
            println!("serve {} with any static file server", project.join("dist").display());
            Ok(())
        }
        "publish" => publish(&project),
        "login" => login(&project, true),
        "logout" => login(&project, false),
        "clean" => clean(&project),
        "help" | "--help" | "-h" => {
            print!("{}", usage());
            Ok(())
        }
        other => Err(format!("unknown command `{other}`")),
    }
}

/// Same as `run` but with an explicit project directory (for tests).
pub fn run_from_args(project: &Path, args: &[String]) -> Result<(), String> {
    match args.first().map(String::as_str) {
        Some("init") => init(project),
        Some("install") => with_arg(args, "install <name>", |name| {
            resolver::install(project, name)?;
            println!("installed {name}");
            Ok(())
        }),
        Some("remove") => with_arg(args, "remove <name>", |name| {
            resolver::remove(project, name)?;
            println!("removed {name}");
            Ok(())
        }),
        Some("update") => {
            resolver::update(project)?;
            println!("updated dependencies");
            Ok(())
        }
        Some("list") => {
            let installed = resolver::list_installed(project)?;
            for name in installed {
                println!("{name}");
            }
            Ok(())
        }
        Some("build") => build(project),
        Some("clean") => clean(project),
        Some("publish") => publish(project),
        _ => Err("unsupported command in run_from_args".into()),
    }
}

fn with_arg<F>(args: &[String], usage: &str, f: F) -> Result<(), String>
where
    F: FnOnce(&str) -> Result<(), String>,
{
    let name = args.get(1).ok_or_else(|| format!("p2lt: expected `{usage}`"))?;
    f(name)
}

fn init(project: &Path) -> Result<(), String> {
    let name = project
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("app")
        .to_string();
    let manifest_path = project.join("p2lt.toml");
    if manifest_path.exists() {
        return Err("p2lt.toml already exists".into());
    }
    let manifest = Manifest::new(name.clone());
    std::fs::write(&manifest_path, manifest.to_toml()).map_err(|e| e.to_string())?;
    std::fs::create_dir_all(project.join("src")).map_err(|e| e.to_string())?;
    std::fs::write(
        project.join("src").join("main.plt"),
        format!("app Main {{\n    Column {{\n        Text \"Hello from {name}\"\n    }}\n}}\n"),
    )
    .map_err(|e| e.to_string())?;
    std::fs::write(project.join(".gitignore"), "target/\ndist/\n.p2lt/packages/\n")
        .map_err(|e| e.to_string())?;
    std::fs::create_dir_all(project.join("tests")).map_err(|e| e.to_string())?;
    println!("initialized {name}");
    Ok(())
}

fn build(project: &Path) -> Result<(), String> {
    let entry = project.join("src").join("main.plt");
    if !entry.exists() {
        return Err("src/main.plt not found; run `p2lt init` first".into());
    }
    let source = std::fs::read_to_string(&entry).map_err(|e| e.to_string())?;
    let label = entry.display().to_string();
    let loader = PackageLoader::new(project.to_path_buf());
    let rust_dependencies =
        crate::manifest::Manifest::from_toml(&std::fs::read_to_string(project.join("p2lt.toml")).map_err(|e| e.to_string())?)?
            .rust_dependencies
            .into_iter()
            .collect::<Vec<_>>();
    let rust = platipus_compiler::pipeline::RustBuild {
        workdir: project.to_path_buf(),
        mode: platipus_compiler::rust::RustMode::Build,
        rust_dependencies,
    };
    match platipus_compiler::pipeline::build_entry_rust(
        &label,
        &source,
        &loader,
        &platipus_compiler::codegen::Web,
        Some(&rust),
    ) {
        Ok(loaded) => {
            let out_dir = project.join("dist");
            std::fs::create_dir_all(&out_dir).map_err(|e| e.to_string())?;
            for artifact in &loaded.compilation.artifacts {
                let path = out_dir.join(artifact.name);
                std::fs::write(&path, &artifact.contents).map_err(|e| e.to_string())?;
            }
            println!("built {} into {}", entry.display(), out_dir.display());
            Ok(())
        }
        Err(failure) => {
            match failure {
                platipus_compiler::pipeline::EntryFailure::Module { label, source, bag } => {
                    for error in bag.errors() {
                        eprintln!("{label}: {error}");
                    }
                    let _ = source;
                }
                platipus_compiler::pipeline::EntryFailure::Whole { file, bag } => {
                    for error in bag.errors() {
                        if let Some(span) = error.span {
                            eprintln!("{}: {error}", file.describe(span));
                        } else {
                            eprintln!("{}: {error}", file.path);
                        }
                    }
                }
            }
            Err("build failed".into())
        }
    }
}

fn publish(project: &Path) -> Result<(), String> {
    let manifest = Manifest::from_toml(&std::fs::read_to_string(project.join("p2lt.toml")).map_err(|e| e.to_string())?)?;
    let src = project.join("src");
    if !src.join("lib.plt").exists() && !src.join("main.plt").exists() {
        return Err("nothing to publish: src/lib.plt missing".into());
    }
    let registry = Registry::for_project(project);
    let dest = registry.root.join(&manifest.name);
    if dest.exists() {
        std::fs::remove_dir_all(&dest).map_err(|e| e.to_string())?;
    }
    std::fs::create_dir_all(&registry.root).map_err(|e| e.to_string())?;
    std::fs::create_dir_all(&dest).map_err(|e| e.to_string())?;
    // Copy only the package payload, never build artifacts or the registry.
    for entry in ["p2lt.toml", "p2lt.lock", "src", "assets", "rust", "tests", "README.md"] {
        let from = project.join(entry);
        if from.exists() {
            let to = dest.join(entry);
            if from.is_dir() {
                crate::registry::copy_dir(&from, &to)?;
            } else {
                std::fs::copy(&from, &to).map_err(|e| e.to_string())?;
            }
        }
    }
    println!("published {} to {}", manifest.name, registry.root.display());
    Ok(())
}

fn login(project: &Path, enabled: bool) -> Result<(), String> {
    let stamp = project.join(".p2lt").join("credentials");
    if enabled {
        std::fs::create_dir_all(project.join(".p2lt")).map_err(|e| e.to_string())?;
        std::fs::write(&stamp, "local=true\n").map_err(|e| e.to_string())?;
        println!("logged in (local registry)");
    } else {
        let _ = std::fs::remove_file(&stamp);
        println!("logged out");
    }
    Ok(())
}

fn clean(project: &Path) -> Result<(), String> {
    for dir in ["target", "dist"] {
        let path = project.join(dir);
        if path.exists() {
            std::fs::remove_dir_all(&path).map_err(|e| e.to_string())?;
            println!("removed {dir}/");
        }
    }
    Ok(())
}

/// Resolves `import Name from "pkg-name"` against `.p2lt/packages/<name>/`.
struct PackageLoader {
    project: PathBuf,
}

impl PackageLoader {
    fn new(project: PathBuf) -> Self {
        Self { project }
    }
}

impl platipus_compiler::loader::Loader for PackageLoader {
    fn read(&self, path: &str) -> Result<String, platipus_compiler::loader::ReadError> {
        if let Ok(text) = std::fs::read_to_string(path) {
            return Ok(text);
        }
        // Non-path import: treat the last path component as a package name.
        let name = Path::new(path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(path);
        let candidate = self
            .project
            .join(".p2lt")
            .join("packages")
            .join(name)
            .join("src")
            .join("lib.plt");
        match std::fs::read_to_string(candidate) {
            Ok(text) => Ok(text),
            Err(_) => Err(platipus_compiler::loader::ReadError::NotFound),
        }
    }
}
