//! The module loader: `import Name from "./file.plt"` reads files, merges
//! their sources into one program, and rejects what a library module cannot
//! be. The loader talks to the filesystem through `FsLoader`, so these tests
//! create a real directory per test.

use std::fs;
use std::path::{Path, PathBuf};

use platipus_compiler::codegen::Web;
use platipus_compiler::diagnostics::DiagnosticBag;
use platipus_compiler::loader::FsLoader;
use platipus_compiler::pipeline;

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("platipus-loader-{name}"));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("scratch directory");
    dir
}

fn write(dir: &Path, name: &str, source: &str) -> PathBuf {
    let path = dir.join(name);
    fs::write(&path, source).expect("write fixture");
    path
}

fn codes(bag: &DiagnosticBag) -> Vec<&str> {
    let mut codes: Vec<&str> = bag.errors().map(|error| error.code).collect();
    codes.sort_unstable();
    codes.dedup();
    codes
}

#[test]
fn modules_are_merged_transitively_into_one_program() {
    let dir = scratch("transitive");
    write(
        &dir,
        "label.plt",
        "component Label {\n    Text \"label\"\n}\n",
    );
    write(
        &dir,
        "banner.plt",
        "import Label from \"./label.plt\"\n\ncomponent Banner {\n    Column {\n        Label { }\n    }\n}\n",
    );
    let entry = write(
        &dir,
        "app.plt",
        "import Banner from \"./banner.plt\"\n\napp Main {\n    Banner { }\n}\n",
    );
    let source = fs::read_to_string(&entry).expect("read entry");
    let loaded = pipeline::build_entry(&entry.display().to_string(), &source, &FsLoader, &Web)
        .expect("combined program compiles");

    let script = loaded
        .compilation
        .artifacts
        .iter()
        .find(|artifact| artifact.name == "app.js")
        .expect("app.js artifact");
    assert!(
        script.contents.contains("function Banner("),
        "{}",
        script.contents
    );
    assert!(
        script.contents.contains("function Label("),
        "{}",
        script.contents
    );
    assert_eq!(loaded.dependencies.len(), 3, "{:?}", loaded.dependencies);
}

#[test]
fn a_program_without_imports_needs_no_loader_time() {
    let dir = scratch("single");
    let entry = write(&dir, "app.plt", "app Main {\n    Column { }\n}\n");
    let source = fs::read_to_string(&entry).expect("read entry");
    let loaded = pipeline::build_entry(&entry.display().to_string(), &source, &FsLoader, &Web)
        .expect("single file compiles");
    assert_eq!(loaded.dependencies.len(), 1);
}

#[test]
fn a_missing_module_reports_the_import_decl() {
    let dir = scratch("missing");
    let entry = write(
        &dir,
        "app.plt",
        "import Banner from \"./banner.plt\"\n\napp Main {\n    Column { }\n}\n",
    );
    let source = fs::read_to_string(&entry).expect("read entry");
    let failure = pipeline::build_entry(&entry.display().to_string(), &source, &FsLoader, &Web)
        .expect_err("a missing module must fail");
    let pipeline::EntryFailure::Module { bag, .. } = failure else {
        panic!("missing module must be a module failure");
    };
    assert_eq!(codes(&bag), ["import-not-found"]);
}

#[test]
fn an_import_cycle_is_rejected() {
    let dir = scratch("cycle");
    write(
        &dir,
        "a.plt",
        "import Back from \"./app.plt\"\n\ncomponent A {\n    Column { }\n}\n",
    );
    let entry = write(
        &dir,
        "app.plt",
        "import A from \"./a.plt\"\n\napp Main {\n    A { }\n}\n",
    );
    let source = fs::read_to_string(&entry).expect("read entry");
    let failure = pipeline::build_entry(&entry.display().to_string(), &source, &FsLoader, &Web)
        .expect_err("a cycle must fail");
    let pipeline::EntryFailure::Module { bag, .. } = failure else {
        panic!("a cycle must be a module failure");
    };
    assert_eq!(codes(&bag), ["import-cycle"]);
}

#[test]
fn an_imported_file_must_not_declare_an_app() {
    let dir = scratch("module-app");
    write(&dir, "banner.plt", "app Nope {\n    Column { }\n}\n");
    let entry = write(
        &dir,
        "app.plt",
        "import Banner from \"./banner.plt\"\n\napp Main {\n    Column { }\n}\n",
    );
    let source = fs::read_to_string(&entry).expect("read entry");
    let failure = pipeline::build_entry(&entry.display().to_string(), &source, &FsLoader, &Web)
        .expect_err("a module with an app must fail");
    let pipeline::EntryFailure::Module { bag, .. } = failure else {
        panic!("must be a module failure");
    };
    assert_eq!(codes(&bag), ["import-module-has-app"]);
}

#[test]
fn a_module_cannot_claim_a_name_the_program_already_declares() {
    let dir = scratch("collision");
    write(
        &dir,
        "banner.plt",
        "component Banner {\n    Column { }\n}\n",
    );
    let entry = write(
        &dir,
        "app.plt",
        "import Banner from \"./banner.plt\"\n\ncomponent Banner {\n    Column { }\n}\n\napp Main {\n    Column { }\n}\n",
    );
    let source = fs::read_to_string(&entry).expect("read entry");
    let failure = pipeline::build_entry(&entry.display().to_string(), &source, &FsLoader, &Web)
        .expect_err("a collision must fail");
    let pipeline::EntryFailure::Module { bag, .. } = failure else {
        panic!("must be a module failure");
    };
    assert_eq!(codes(&bag), ["import-collision"]);
}

#[test]
fn an_import_label_is_unique_across_the_program() {
    let dir = scratch("dup-label");
    write(&dir, "a.plt", "component A {\n    Column { }\n}\n");
    write(&dir, "b.plt", "component B {\n    Column { }\n}\n");
    let entry = write(
        &dir,
        "app.plt",
        "import Same from \"./a.plt\"\nimport Same from \"./b.plt\"\n\napp Main {\n    Column { }\n}\n",
    );
    let source = fs::read_to_string(&entry).expect("read entry");
    let failure = pipeline::build_entry(&entry.display().to_string(), &source, &FsLoader, &Web)
        .expect_err("a repeated label must fail");
    let pipeline::EntryFailure::Module { bag, .. } = failure else {
        panic!("must be a module failure");
    };
    assert_eq!(codes(&bag), ["duplicate-import"]);
}

#[test]
fn a_module_is_loaded_at_most_once() {
    let dir = scratch("dup-module");
    write(&dir, "a.plt", "component A {\n    Column { }\n}\n");
    let entry = write(
        &dir,
        "app.plt",
        "import A from \"./a.plt\"\nimport B from \"./a.plt\"\n\napp Main {\n    Column { }\n}\n",
    );
    let source = fs::read_to_string(&entry).expect("read entry");
    let failure = pipeline::build_entry(&entry.display().to_string(), &source, &FsLoader, &Web)
        .expect_err("a second load of the same file must fail");
    let pipeline::EntryFailure::Module { bag, .. } = failure else {
        panic!("must be a module failure");
    };
    assert_eq!(codes(&bag), ["duplicate-module"]);
}

#[test]
fn an_error_inside_a_module_lands_in_the_combined_program() {
    let dir = scratch("module-error");
    write(
        &dir,
        "banner.plt",
        "component Banner {\n    Buton \"x\" { }\n}\n",
    );
    let entry = write(
        &dir,
        "app.plt",
        "import Banner from \"./banner.plt\"\n\napp Main {\n    Banner { }\n}\n",
    );
    let source = fs::read_to_string(&entry).expect("read entry");
    let failure = pipeline::build_entry(&entry.display().to_string(), &source, &FsLoader, &Web)
        .expect_err("a module with an unknown element must fail");
    let pipeline::EntryFailure::Whole { bag, .. } = failure else {
        panic!("an element error is a whole-program failure");
    };
    assert_eq!(codes(&bag), ["unknown-element"]);
}

#[test]
fn the_loaderless_pipeline_refuses_an_unresolved_import() {
    let source = "import Banner from \"./banner.plt\"\n\napp Main {\n    Column { }\n}\n";
    let bag =
        pipeline::build_file("app.plt", source, &Web).expect_err("must fail without a loader");
    assert_eq!(codes(&bag), ["import-not-loaded"]);
}

#[test]
fn a_function_imported_from_a_module_lands_in_the_programs_output() {
    // Every existing fixture imports a `component`. The loader inlines module
    // sources into one program, so a top-level `fn` should arrive the same way
    // and be callable from the entry.
    let dir = scratch("function-import");
    write(
        &dir,
        "math.plt",
        "fn triple(n: Int) -> Int {\n    return n * 3\n}\n",
    );
    let entry = write(
        &dir,
        "app.plt",
        r#"import Math from "./math.plt"

app Main {
    state n = triple(7)
    Column { Text n }
}
"#,
    );
    let source = fs::read_to_string(&entry).expect("read entry");
    let artifacts = pipeline::build_entry(&entry.display().to_string(), &source, &FsLoader, &Web)
        .expect("a module of functions has to build")
        .compilation
        .artifacts;
    let script = artifacts
        .iter()
        .find(|artifact| artifact.name == "app.js")
        .expect("app.js")
        .contents
        .clone();
    assert!(script.contains("p_n * 3"), "{script}");
    assert!(script.contains("as triple"), "{script}");
}

#[test]
fn a_function_imported_from_a_module_type_checks_against_its_signature() {
    // Inlining is what makes the function visible, so the arity rule has to see
    // it too rather than treating an unknown callee as unchecked.
    let dir = scratch("function-import-arity");
    write(
        &dir,
        "math.plt",
        "fn triple(n: Int) -> Int {\n    return n * 3\n}\n",
    );
    let entry = write(
        &dir,
        "app.plt",
        r#"import Math from "./math.plt"

app Main {
    state n = triple(7, 8)
    Column { Text n }
}
"#,
    );
    let source = fs::read_to_string(&entry).expect("read entry");
    let failure = pipeline::build_entry(&entry.display().to_string(), &source, &FsLoader, &Web)
        .expect_err("too many arguments has to fail");
    let pipeline::EntryFailure::Whole { bag, .. } = failure else {
        panic!("an arity error is a whole-program failure");
    };
    assert_eq!(codes(&bag), ["wrong-argument-count"]);
}


// A loader is allowed to answer a path with a different file -- the Library
// Store answers the name `computasi` with
// `<store>/packages/computasi/src/lib.plt`. When it does, the file it read has
// to be reported, because a relative import inside that module means "next to
// me" and "me" has just moved. This is the shape a real package has.
struct StoreLikeLoader {
    store: PathBuf,
}

impl platipus_compiler::loader::Loader for StoreLikeLoader {
    fn read(
        &self,
        path: &str,
    ) -> Result<platipus_compiler::loader::Read, platipus_compiler::loader::ReadError> {
        let direct = Path::new(path);
        if direct.exists() {
            let source = fs::read_to_string(direct).map_err(|error| {
                platipus_compiler::loader::ReadError::Io(error.to_string())
            })?;
            return Ok(platipus_compiler::loader::Read {
                path: path.to_string(),
                source,
            });
        }
        let name = direct
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(path);
        let candidate = self.store.join(name).join("src").join("lib.plt");
        let source = fs::read_to_string(&candidate)
            .map_err(|_| platipus_compiler::loader::ReadError::NotFound)?;
        Ok(platipus_compiler::loader::Read {
            path: candidate.to_string_lossy().to_string(),
            source,
        })
    }
}

#[test]
fn a_modules_own_relative_import_is_resolved_beside_the_module() {
    let dir = scratch("package-sibling");
    let store = dir.join("store");
    fs::create_dir_all(store.join("pkg").join("src")).expect("package directory");
    fs::write(
        store.join("pkg").join("src").join("lib.plt"),
        "import Parts from \"./parts.plt\"\n\nfn total() -> Int {\n    return piece() + 1\n}\n",
    )
    .expect("write package entry");
    fs::write(
        store.join("pkg").join("src").join("parts.plt"),
        "fn piece() -> Int {\n    return 2\n}\n",
    )
    .expect("write package module");

    let entry = write(
        &dir,
        "app.plt",
        "import Pkg from \"pkg\"\n\napp Main {\n    state n = total()\n    Column { Text n }\n}\n",
    );
    let source = fs::read_to_string(&entry).expect("read entry");
    let artifacts =
        pipeline::build_entry(&entry.display().to_string(), &source, &StoreLikeLoader { store }, &Web)
            .expect("the package's own import has to resolve")
            .compilation
            .artifacts;
    let script = artifacts
        .iter()
        .find(|artifact| artifact.name == "app.js")
        .expect("app.js")
        .contents
        .clone();
    assert!(script.contains("return 2"), "{script}");
    assert!(script.contains("f_piece() + 1"), "{script}");
}

#[test]
fn a_missing_sibling_inside_a_package_is_reported_against_that_package() {
    let dir = scratch("package-missing-sibling");
    let store = dir.join("store");
    fs::create_dir_all(store.join("pkg").join("src")).expect("package directory");
    fs::write(
        store.join("pkg").join("src").join("lib.plt"),
        "import Gone from \"./gone.plt\"\n\nfn total() -> Int {\n    return 1\n}\n",
    )
    .expect("write package entry");

    let entry = write(
        &dir,
        "app.plt",
        "import Pkg from \"pkg\"\n\napp Main {\n    state n = total()\n    Column { Text n }\n}\n",
    );
    let source = fs::read_to_string(&entry).expect("read entry");
    let failure = pipeline::build_entry(
        &entry.display().to_string(),
        &source,
        &StoreLikeLoader { store },
        &Web,
    )
    .expect_err("the missing module has to fail");
    let pipeline::EntryFailure::Module { bag, .. } = failure else {
        panic!("a module that cannot be read is a module failure");
    };
    assert_eq!(codes(&bag), ["import-not-found"]);
}
