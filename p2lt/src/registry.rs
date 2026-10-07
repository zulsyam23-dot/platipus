//! A registry is a directory of packages. Each package is a subdirectory
//! containing a valid `p2lt.toml` and a `src/` tree. This keeps the first
//! implementation fully local and testable; a network registry server can
//! later back the same interface.

use std::path::{Path, PathBuf};

pub struct Registry {
    pub root: PathBuf,
}

impl Registry {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    /// The registry directory: `$P2LT_REGISTRY`, else `<store>/registry`
    /// so local packages are shared across projects instead of living
    /// inside each project.
    pub fn for_project(_project: &Path) -> Self {
        if let Some(env) = std::env::var_os("P2LT_REGISTRY") {
            return Self::new(env);
        }
        Self::new(crate::cache::store_dir().join("registry"))
    }

    pub fn list(&self) -> Vec<String> {
        let mut out = Vec::new();
        if let Ok(entries) = std::fs::read_dir(&self.root) {
            for entry in entries.flatten() {
                if entry.path().is_dir() {
                    if let Ok(name) = entry.file_name().into_string() {
                        out.push(name);
                    }
                }
            }
        }
        out.sort();
        out
    }

    pub fn get(&self, name: &str) -> Option<PathBuf> {
        let path = self.root.join(name);
        if path.join("p2lt.toml").exists() {
            Some(path)
        } else {
            None
        }
    }

    pub fn search(&self, query: &str) -> Vec<String> {
        self.list()
            .into_iter()
            .filter(|name| name.contains(query))
            .collect()
    }
}

/// Copies a directory tree.
pub fn copy_dir(src: &Path, dst: &Path) -> Result<(), String> {
    std::fs::create_dir_all(dst).map_err(|e| e.to_string())?;
    for entry in std::fs::read_dir(src).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let from = entry.path();
        let to = dst.join(entry.file_name());
        if from.is_dir() {
            copy_dir(&from, &to)?;
        } else {
            std::fs::copy(&from, &to).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

/// FNV-1a 64-bit over the contents of every file in `dir`, sorted by
/// relative path. Deterministic and dependency-free.
pub fn checksum(dir: &Path) -> Result<String, String> {
    let mut files = Vec::new();
    collect_files(dir, dir, &mut files)?;
    files.sort();
    let mut hash: u64 = 0xcbf29ce484222325;
    for file in files {
        let data = std::fs::read(dir.join(&file)).map_err(|e| e.to_string())?;
        for byte in data {
            hash ^= byte as u64;
            hash = hash.wrapping_mul(0x100000001b3);
        }
    }
    Ok(format!("{hash:016x}"))
}

fn collect_files(root: &Path, current: &Path, out: &mut Vec<String>) -> Result<(), String> {
    for entry in std::fs::read_dir(current).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if path.is_dir() {
            collect_files(root, &path, out)?;
        } else {
            out.push(
                path.strip_prefix(root)
                    .map_err(|_| "path error".to_string())?
                    .to_string_lossy()
                    .replace('\\', "/"),
            );
        }
    }
    Ok(())
}
