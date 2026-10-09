use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
static TEMP_ID: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

fn temp_dir() -> PathBuf {
    for _ in 0..100 {
        let id = TEMP_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let dir = std::env::temp_dir().join(format!("p2lt-test-{}-{id}", std::process::id()));
        match std::fs::create_dir(&dir) {
            Ok(()) => return dir,
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => panic!("create temporary test directory: {error}"),
        }
    }
    panic!("could not create a unique temporary test directory");
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
    let _env_guard = ENV_LOCK
        .get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
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
fn http_registry_lists_and_installs_packages() {
    let _env_guard = ENV_LOCK
        .get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if std::process::Command::new("curl")
        .arg("--version")
        .output()
        .is_err()
    {
        return;
    }
    let root = temp_dir();
    let package = root.join("source");
    write_package(&package);
    let archive = platipus_p2lt::package::pack(&package, &root).unwrap();
    let archive_bytes = std::fs::read(&archive).unwrap();
    let expected_archive_bytes = archive_bytes.clone();

    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let server = std::thread::spawn(move || {
        let mut uploaded = None;
        for _ in 0..3 {
            let (stream, _) = listener.accept().unwrap();
            let mut stream = BufReader::new(stream);
            let mut headers = Vec::new();
            loop {
                let mut line = Vec::new();
                stream.read_until(b'\n', &mut line).unwrap();
                headers.extend_from_slice(&line);
                if line == b"\r\n" || line.is_empty() {
                    break;
                }
            }
            let header_text = String::from_utf8_lossy(&headers);
            let first_line = header_text.lines().next().unwrap_or_default().to_string();
            let content_length = header_text
                .lines()
                .find_map(|line| {
                    line.strip_prefix("Content-Length:")
                        .and_then(|value| value.trim().parse::<usize>().ok())
                })
                .unwrap_or(0);
            let mut upload_body = vec![0; content_length];
            stream.read_exact(&mut upload_body).unwrap();
            let (status, content_type, body): (&str, &str, &[u8]) =
                if first_line.starts_with("GET /index.txt ") {
                    ("200 OK", "text/plain", b"math\n")
                } else if first_line.starts_with("GET /packages/math.libplt ") {
                    ("200 OK", "application/octet-stream", &archive_bytes)
                } else if first_line.starts_with("PUT /packages/math.libplt ") {
                    uploaded = Some(upload_body);
                    ("201 Created", "text/plain", b"")
                } else {
                    ("404 Not Found", "text/plain", b"not found")
                };
            let stream = stream.get_mut();
            write!(
                stream,
                "HTTP/1.1 {status}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                body.len()
            )
            .unwrap();
            stream.write_all(body).unwrap();
        }
        uploaded
    });

    let project = root.join("project");
    std::fs::create_dir_all(&project).unwrap();
    platipus_p2lt::cli::run_from_args(&project, &["init".into()]).unwrap();
    let store = root.join("store");
    let old_registry_url = std::env::var_os("P2LT_REGISTRY_URL");
    let old_store = std::env::var_os("PLATIPUS_STORE");
    unsafe {
        std::env::set_var("P2LT_REGISTRY_URL", format!("http://{address}"));
        std::env::set_var("PLATIPUS_STORE", &store);
    }

    let result = (|| -> Result<_, String> {
        let names = platipus_p2lt::registry::Registry::http_list()?
            .ok_or("HTTP registry was not configured")?;
        platipus_p2lt::cli::run_from_args(&project, &["add".into(), "math".into()])?;
        let installed = store.join("packages/math/src/lib.plt").exists();
        let lock = std::fs::read_to_string(project.join("p2lt.lock"))
            .map_err(|error| error.to_string())?;
        platipus_p2lt::registry::Registry::http_publish("math", &archive)?;
        Ok((names, installed, lock))
    })();

    unsafe {
        match old_registry_url {
            Some(value) => std::env::set_var("P2LT_REGISTRY_URL", value),
            None => std::env::remove_var("P2LT_REGISTRY_URL"),
        }
        match old_store {
            Some(value) => std::env::set_var("PLATIPUS_STORE", value),
            None => std::env::remove_var("PLATIPUS_STORE"),
        }
    }
    let (names, installed, lock) = result.unwrap();
    assert_eq!(names, vec!["math".to_string()]);
    assert!(installed);
    assert!(lock.contains("source = \"registry:http://"));
    assert_eq!(server.join().unwrap().unwrap(), expected_archive_bytes);
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
    assert!(
        manifest
            .integrity
            .as_deref()
            .unwrap()
            .starts_with("fnv1a64:")
    );

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
