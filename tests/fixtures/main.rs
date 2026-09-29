//! Every file in `fixtures/invalid/` must fail to compile with the diagnostic
//! its name claims. The file name is the expectation, so a fixture cannot drift
//! away from what it is supposed to prove without the test noticing.

use std::fs;
use std::path::{Path, PathBuf};

use platipus_compiler::codegen::Web;
use platipus_compiler::pipeline;

fn fixtures() -> Vec<PathBuf> {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("fixtures")
        .join("invalid");
    let mut paths: Vec<PathBuf> = fs::read_dir(&dir)
        .expect("fixtures/invalid exists")
        .map(|entry| entry.expect("directory entry").path())
        .filter(|path| path.extension().is_some_and(|extension| extension == "plt"))
        .collect();
    paths.sort();
    assert!(!paths.is_empty(), "no invalid fixtures found in {}", dir.display());
    paths
}

fn expected_code(path: &Path) -> String {
    path.file_stem()
        .expect("fixture has a name")
        .to_string_lossy()
        .to_string()
}

#[test]
fn every_invalid_fixture_fails_with_its_named_diagnostic() {
    let mut failures = Vec::new();
    for path in fixtures() {
        let source = fs::read_to_string(&path).expect("read fixture");
        let expected = expected_code(&path);
        let label = path.display().to_string();
        let bag = match pipeline::build_file(&label, &source, &Web) {
            Ok(_) => {
                failures.push(format!("{label}: compiled cleanly, expected `{expected}`"));
                continue;
            }
            Err(bag) => bag,
        };
        let found: Vec<&str> = bag.errors().map(|error| error.code).collect();
        if !found.contains(&expected.as_str()) {
            failures.push(format!("{label}: expected `{expected}`, got {found:?}"));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn every_valid_fixture_compiles() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("fixtures")
        .join("valid");
    let mut paths: Vec<PathBuf> = fs::read_dir(&dir)
        .expect("fixtures/valid exists")
        .map(|entry| entry.expect("directory entry").path())
        .filter(|path| path.extension().is_some_and(|extension| extension == "plt"))
        .collect();
    paths.sort();
    assert!(!paths.is_empty(), "no valid fixtures found in {}", dir.display());
    let mut failures = Vec::new();
    for path in paths {
        let source = fs::read_to_string(&path).expect("read fixture");
        let label = path.display().to_string();
        if let Err(bag) = pipeline::build_file(&label, &source, &Web) {
            let found: Vec<&str> = bag.errors().map(|error| error.code).collect();
            failures.push(format!("{label}: {found:?}"));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
