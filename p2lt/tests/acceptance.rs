//! The acceptance package, exercised through the whole package flow.
//!
//! `p2lt/tests/package.rs` proves the package manager moves files around. This
//! proves a program actually *uses* a published package: `examples/computasi/`
//! is published to a local registry, installed into a fresh application, its
//! functions are called from a `derived` and from a click handler, the app is
//! built, and Node runs the app's own tests against the emitted DOM shim.
//!
//! Nothing here reaches into the store by hand. Every step goes through the
//! `p2lt` binary, so a break in publish, install, resolution, or codegen fails
//! the test rather than being papered over by the test's own setup.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, OnceLock};

static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
static TEMP_ID: AtomicU64 = AtomicU64::new(0);

/// The registry and the store are read from the environment, so a test that
/// changed them would make every other test in this binary read someone else's
/// directories. The lock is taken before the variables are set and held until
/// the test finishes.
fn guard() -> std::sync::MutexGuard<'static, ()> {
    ENV_LOCK
        .get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

fn temp_dir() -> PathBuf {
    for _ in 0..100 {
        let id = TEMP_ID.fetch_add(1, Ordering::Relaxed);
        let dir = std::env::temp_dir().join(format!("p2lt-acceptance-{}-{id}", std::process::id()));
        match std::fs::create_dir(&dir) {
            Ok(()) => return dir,
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => panic!("create temporary test directory: {error}"),
        }
    }
    panic!("could not create a unique temporary test directory");
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("p2lt lives inside the workspace")
        .to_path_buf()
}

fn p2lt(project: &Path, args: &[&str]) {
    let output = Command::new(env!("CARGO_BIN_EXE_p2lt"))
        .args(args)
        .current_dir(project)
        .output()
        .unwrap_or_else(|error| panic!("run p2lt {args:?}: {error}"));
    assert!(
        output.status.success(),
        "p2lt {args:?} failed\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
}

fn node_available() -> bool {
    Command::new("node")
        .arg("--version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

/// The application that consumes the package. It is written here rather than
/// checked in because the point is that a *separate* project can reach the
/// functions; a checked-in app would live beside the package it imports.
const CONSUMER: &str = r##"
import Komputasi from "computasi"

app Laporan {
    state seed = 12
    state report = ""

    // A derived has to recompute from a package function, not just from state
    // the component owns.
    derived greatestCommonDivisor = gcd(seed, 18)
    derived sorted = sortAsc([5, 3, 9, 1])
    derived hash = fnv1a32("platipus")

    fn rebuild() {
        report = toText(fib(10)) + "|" + toText(factorial(5)) + "|" + toText(collatzSteps(27)) + "|" + toText(binarySearch([1, 3, 5, 7, 9], 7))
    }

    Column {
        Text greatestCommonDivisor
        Text sorted
        Text hash
        Text report
        Button "rebuild" {
            on click { rebuild() }
        }
        Button "seed" {
            on click { seed = seed + 6 }
        }
    }
}

test thePackageFunctionsRunInsideTheConsumer {
    click "rebuild"
    expect report == "55|120|111|3"
}

test theDerivedFollowsItsImportedDependency {
    expect greatestCommonDivisor == 6
    expect join(sorted, ",") == "1,3,5,9"
    expect hash == 135729481
    click "seed"
    expect greatestCommonDivisor == 18
    expect hash == 135729481
}

test sortAscLeftTheConsumersOwnListAlone {
    expect join([5, 3, 9, 1], ",") == "5,3,9,1"
}
"##;

#[test]
fn a_published_package_is_installed_called_and_built_end_to_end() {
    let _env = guard();
    if !node_available() {
        eprintln!("skipped: node is not on the PATH");
        return;
    }
    let root = temp_dir();
    let registry = root.join("registry");
    let project = root.join("konsumen");
    let store = root.join("store");
    std::fs::create_dir_all(&registry).unwrap();
    std::fs::create_dir_all(&project).unwrap();
    unsafe {
        std::env::set_var("P2LT_REGISTRY", &registry);
        std::env::set_var("PLATIPUS_STORE", &store);
    }

    // 1. Publish the acceptance package the way a library author would. It is
    //    copied out of the repository so the test cannot write into the tree.
    let source = repo_root().join("examples").join("computasi");
    let author = root.join("penulis");
    copy_tree(&source, &author);
    p2lt(&author, &["publish"]);

    // 2. Start an unrelated project and install the package into it.
    p2lt(&project, &["init"]);
    p2lt(&project, &["install", "computasi"]);
    let manifest = std::fs::read_to_string(project.join("p2lt.toml")).unwrap();
    assert!(manifest.contains("computasi = \"0.1\""), "{manifest}");
    assert!(
        store.join("packages/computasi/src/lib.plt").exists(),
        "the package did not reach the store"
    );

    // 3. Write the consumer and build it. `build` resolves `import ... from
    //    "computasi"` through the store, so a resolution break fails here.
    std::fs::write(project.join("src").join("main.plt"), CONSUMER).unwrap();
    p2lt(&project, &["build"]);
    let dist = project.join("dist");
    for artifact in ["app.js", "dom.mjs", "tests.mjs", "index.html"] {
        assert!(dist.join(artifact).exists(), "{artifact} was not written");
    }

    // 4. Run the consumer's own tests. Node reads the emitted program and the
    //    emitted shim, so this is the app running, not the compiler.
    let output = Command::new("node")
        .arg(dist.join("tests.mjs"))
        .current_dir(&dist)
        .output()
        .unwrap_or_else(|error| panic!("run the emitted runner: {error}"));
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "the consumer's tests failed\n{stdout}\n{stderr}");
    // Sixteen, not three: the consumer declares three tests and inherits the
    // thirteen the imported package shipped with it. Both sets have to pass here,
    // because this is the run that proves the package works from outside itself.
    assert!(stdout.contains("16 passed, 0 failed"), "{stdout}");

    std::fs::remove_dir_all(&root).ok();
}

fn copy_tree(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).unwrap();
    for entry in std::fs::read_dir(from).unwrap() {
        let entry = entry.unwrap();
        let target = to.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_tree(&entry.path(), &target);
        } else {
            std::fs::copy(entry.path(), &target).unwrap();
        }
    }
}
