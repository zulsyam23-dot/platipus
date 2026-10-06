use std::path::{Path, PathBuf};

fn temp_dir() -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "p2lt-test-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn write_package(dir: &Path) {
    std::fs::create_dir_all(dir.join("src")).unwrap();
    std::fs::write(
        dir.join("p2lt.toml"),
        r#"[package]
name = "math"
version = "0.1.0"
edition = "2026"

[dependencies]
"#,
    )
    .unwrap();
    std::fs::write(
        dir.join("src").join("lib.plt"),
        "fn double(n: Int) -> Int {\n    n * 2\n}\n",
    )
    .unwrap();
}

#[test]
fn init_install_list_update_remove() {
    let root = temp_dir();
    let project = root.join("project");
    let registry = root.join("registry");
    std::fs::create_dir_all(&project).unwrap();
    std::fs::create_dir_all(&registry).unwrap();
    write_package(&registry.join("math"));

    unsafe {
        std::env::set_var("P2LT_REGISTRY", &registry);
    }

    platipus_p2lt::cli::run_from_args(&project, &["init".into()]).unwrap();
    let manifest = std::fs::read_to_string(project.join("p2lt.toml")).unwrap();
    assert!(manifest.contains("[package]"));
    assert!(project.join("src/main.plt").exists());

    platipus_p2lt::cli::run_from_args(&project, &["install".into(), "math".into()]).unwrap();
    assert!(project.join(".p2lt/packages/math/src/lib.plt").exists());
    let manifest = std::fs::read_to_string(project.join("p2lt.toml")).unwrap();
    assert!(manifest.contains("math = \"0.1\""));
    let lock = std::fs::read_to_string(project.join("p2lt.lock")).unwrap();
    assert!(lock.contains("name = \"math\""));

    platipus_p2lt::cli::run_from_args(&project, &["list".into()]).unwrap();

    platipus_p2lt::cli::run_from_args(&project, &["update".into()]).unwrap();
    assert!(project.join("p2lt.lock").exists());

    platipus_p2lt::cli::run_from_args(&project, &["remove".into(), "math".into()]).unwrap();
    assert!(!project.join(".p2lt/packages/math").exists());
    let manifest = std::fs::read_to_string(project.join("p2lt.toml")).unwrap();
    assert!(!manifest.contains("math"));
}
