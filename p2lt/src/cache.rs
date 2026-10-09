use std::path::PathBuf;

/// Root of the global Library Store: `PLATIPUS_STORE` when set, otherwise a
/// per-user platform location. Packages never live inside a project.
pub fn store_dir() -> PathBuf {
    if let Some(env) = std::env::var_os("PLATIPUS_STORE") {
        return PathBuf::from(env);
    }
    #[cfg(windows)]
    {
        if let Some(appdata) = std::env::var_os("LOCALAPPDATA") {
            return PathBuf::from(appdata).join("platipus").join("store");
        }
    }
    #[cfg(not(windows))]
    {
        if let Some(xdg) = std::env::var_os("XDG_DATA_HOME") {
            return PathBuf::from(xdg).join("platipus");
        }
        if let Some(home) = std::env::var_os("HOME") {
            return PathBuf::from(home)
                .join(".local")
                .join("share")
                .join("platipus");
        }
    }
    std::env::temp_dir().join("platipus-store")
}

/// Installed packages live under `<store>/packages/<name>/`.
pub fn packages_dir() -> PathBuf {
    store_dir().join("packages")
}

pub fn package_dir(name: &str) -> PathBuf {
    packages_dir().join(name)
}

/// Resolves imports against the filesystem first, then the Library Store:
/// `import X from "./mod.plt"` reads a file, while `import X from "pkg"`
/// reads `<store>/packages/pkg/src/lib.plt`.
pub struct StoreLoader;

impl StoreLoader {
    pub fn new() -> Self {
        Self
    }
}

impl Default for StoreLoader {
    fn default() -> Self {
        Self::new()
    }
}

impl platipus_compiler::loader::Loader for StoreLoader {
    fn read(
        &self,
        path: &str,
    ) -> Result<platipus_compiler::loader::Read, platipus_compiler::loader::ReadError> {
        if let Ok(source) = std::fs::read_to_string(path) {
            return Ok(platipus_compiler::loader::Read {
                path: path.to_string(),
                source,
            });
        }
        let name = std::path::Path::new(path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(path);
        let candidate = package_dir(name).join("src").join("lib.plt");
        match std::fs::read_to_string(&candidate) {
            Ok(source) => Ok(platipus_compiler::loader::Read {
                // The path the caller asked for was a package name, not a file.
                // The store file is where this module actually lives, and it is
                // what its own relative imports have to be resolved against.
                path: candidate.to_string_lossy().to_string(),
                source,
            }),
            Err(_) => Err(platipus_compiler::loader::ReadError::NotFound),
        }
    }
}
