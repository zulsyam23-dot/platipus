//! Runs the browser harnesses as part of `cargo test`.
//!
//! Each harness is a Node script that imports the generated `app.js` and the
//! generated DOM shim, so it can only run after a real compile. The test builds
//! the fixture into a scratch directory, hands that directory to the script, and
//! keeps nothing behind. Without Node on `PATH` the tests report that they were
//! skipped instead of failing, so a Rust-only checkout still tests clean.

use std::path::{Path, PathBuf};
use std::process::Command;

use platipus_compiler::codegen::Web;
use platipus_compiler::loader::FsLoader;
use platipus_compiler::pipeline;

struct Harness {
    script: &'static str,
    fixture: &'static str,
}

const HARNESSES: &[Harness] = &[
    Harness { script: "run.mjs", fixture: "../../examples/counter.plt" },
    Harness { script: "templates.mjs", fixture: "fixtures/templates.plt" },
    Harness { script: "lifecycle.mjs", fixture: "fixtures/lifecycle.plt" },
    Harness { script: "events.mjs", fixture: "fixtures/events.plt" },
    Harness { script: "runtime.mjs", fixture: "fixtures/runtime.plt" },
    Harness { script: "ui.mjs", fixture: "fixtures/ui.plt" },
    Harness { script: "components.mjs", fixture: "fixtures/components.plt" },
    Harness { script: "showcase.mjs", fixture: "../../examples/showcase/main.plt" },
];

fn here() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("tests").join("web")
}

fn node() -> Option<PathBuf> {
    for candidate in ["node", "node.exe"] {
        let found = Command::new(candidate)
            .arg("--version")
            .output()
            .is_ok_and(|output| output.status.success());
        if found {
            return Some(PathBuf::from(candidate));
        }
    }
    None
}

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("platipus-web-{name}"));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("scratch directory");
    dir
}

#[test]
fn the_harnesses_run_against_a_fresh_build() {
    let Some(node) = node() else {
        eprintln!("skipping the web harnesses: node is not on PATH");
        return;
    };
    let root = here();
    let mut failures = Vec::new();
    for harness in HARNESSES {
        let fixture = root.join(harness.fixture);
        let source = std::fs::read_to_string(&fixture)
            .unwrap_or_else(|error| panic!("read {}: {error}", fixture.display()));
        let out = scratch(harness.script);
        let compilation = pipeline::build_entry(
            &fixture.display().to_string(),
            &source,
            &FsLoader,
            &Web,
        )
        .unwrap_or_else(|failure| panic!("{}: {failure:?}", fixture.display()))
        .compilation;
        for artifact in &compilation.artifacts {
            std::fs::write(out.join(artifact.name), &artifact.contents)
                .unwrap_or_else(|error| panic!("write {}: {error}", artifact.name));
        }
        let output = Command::new(&node)
            .arg(root.join(harness.script))
            .arg(&out)
            .output()
            .unwrap_or_else(|error| panic!("run {}: {error}", harness.script));
        let _ = std::fs::remove_dir_all(&out);
        if !output.status.success() {
            failures.push(format!(
                "{}:\n{}\n{}",
                harness.script,
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            ));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
