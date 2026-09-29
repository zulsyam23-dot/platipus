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
