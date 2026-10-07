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

    let store = root.join("store");
    unsafe {
        std::env::set_var("P2LT_REGISTRY", &registry);
        std::env::set_var("PLATIPUS_STORE", &store);
    }

    platipus_p2lt::cli::run_from_args(&project, &["init".into()]).unwrap();
    let manifest = std::fs::read_to_string(project.join("p2lt.toml")).unwrap();
    assert!(manifest.contains("[package]"));
    assert!(project.join("src/main.plt").exists());

    platipus_p2lt::cli::run_from_args(&project, &["install".into(), "math".into()]).unwrap();
    assert!(store.join("packages/math/src/lib.plt").exists());
    assert!(!project.join(".p2lt/packages").exists());
    let manifest = std::fs::read_to_string(project.join("p2lt.toml")).unwrap();
    assert!(manifest.contains("math = \"0.1\""));
    let lock = std::fs::read_to_string(project.join("p2lt.lock")).unwrap();
    assert!(lock.contains("name = \"math\""));

    platipus_p2lt::cli::run_from_args(&project, &["list".into()]).unwrap();

    platipus_p2lt::cli::run_from_args(&project, &["update".into()]).unwrap();
    assert!(project.join("p2lt.lock").exists());

    platipus_p2lt::cli::run_from_args(&project, &["remove".into(), "math".into()]).unwrap();
    assert!(!store.join("packages/math").exists());
    let manifest = std::fs::read_to_string(project.join("p2lt.toml")).unwrap();
    assert!(!manifest.contains("math"));
}

#[test]
fn pack_validate_unpack_roundtrip() {
    let root = temp_dir();
    let project = root.join("project");
    std::fs::create_dir_all(&project).unwrap();
    write_package(&project);

    let packed = platipus_p2lt::package::pack(&project, &root).unwrap();
    assert!(packed.extension().map(|e| e == "libplt").unwrap_or(false));

    let manifest = platipus_p2lt::package::validate(&packed).unwrap();
    assert_eq!(manifest.name, "math");
    assert_eq!(manifest.version, "0.1.0");
    assert!(manifest.integrity.as_deref().unwrap().starts_with("fnv1a64:"));

    let dest = root.join("unpacked");
    platipus_p2lt::package::unpack(&packed, &dest).unwrap();
    assert!(dest.join("src/lib.plt").exists());
    assert!(dest.join("p2lt.toml").exists());

    // A corrupted archive must fail validation, not silently unpack.
    let mut corrupted = std::fs::read(&packed).unwrap();
    let n = corrupted.len();
    corrupted[n - 10] ^= 0xff;
    let bad = root.join("bad.libplt");
    std::fs::write(&bad, &corrupted).unwrap();
    assert!(platipus_p2lt::package::validate(&bad).is_err());
}
