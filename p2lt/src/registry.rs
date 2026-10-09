//! Local and HTTP package registry access.

use std::path::{Path, PathBuf};
use std::process::Command;

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

    pub fn http_url() -> Option<String> {
        std::env::var("P2LT_REGISTRY_URL")
            .ok()
            .map(|url| url.trim_end_matches('/').to_string())
            .filter(|url| !url.is_empty())
    }

    /// Lists package names from the configured HTTP registry's `index.txt`.
    pub fn http_list() -> Result<Option<Vec<String>>, String> {
        let Some(base) = Self::http_url() else {
            return Ok(None);
        };
        validate_http_url(&base)?;
        let output = curl_output(&format!("{base}/index.txt"))?;
        let mut names = Vec::new();
        for line in String::from_utf8(output)
            .map_err(|_| "registry index is not UTF-8".to_string())?
            .lines()
        {
            let name = line.trim();
            if name.is_empty() || name.starts_with('#') {
                continue;
            }
            validate_package_name(name)?;
            names.push(name.to_string());
        }
        names.sort();
        names.dedup();
        Ok(Some(names))
    }

    /// Downloads the latest archive at `/packages/{name}.libplt`.
    pub fn http_download(name: &str, dest: &Path) -> Result<(), String> {
        let base = Self::http_url().ok_or("P2LT_REGISTRY_URL is not configured")?;
        Self::http_download_from(&base, name, dest)
    }

    pub fn http_download_from(base: &str, name: &str, dest: &Path) -> Result<(), String> {
        validate_package_name(name)?;
        validate_http_url(base)?;
        let base = base.trim_end_matches('/');
        curl_to_file(&format!("{base}/packages/{name}.libplt"), dest)
    }

    /// Publishes a package archive using HTTP PUT.
    pub fn http_publish(name: &str, archive: &Path) -> Result<(), String> {
        let base = Self::http_url().ok_or("P2LT_REGISTRY_URL is not configured")?;
        Self::http_publish_to(&base, name, archive)
    }

    pub fn http_publish_to(base: &str, name: &str, archive: &Path) -> Result<(), String> {
        validate_package_name(name)?;
        validate_http_url(base)?;
        let base = base.trim_end_matches('/');
        let url = format!("{base}/packages/{name}.libplt");
        let output = Command::new("curl")
            .args([
                "--fail",
                "--silent",
                "--show-error",
                "--location",
                "--max-time",
                "60",
                "--request",
                "PUT",
                "--upload-file",
            ])
            .arg(archive)
            .arg(url)
            .output()
            .map_err(|error| format!("could not start curl for HTTP registry: {error}"))?;
        if !output.status.success() {
            return Err(format!(
                "registry publish failed: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            ));
        }
        Ok(())
    }
}

fn validate_http_url(url: &str) -> Result<(), String> {
    if !(url.starts_with("http://") || url.starts_with("https://"))
        || url.contains('@')
        || url.chars().any(char::is_whitespace)
    {
        return Err("P2LT_REGISTRY_URL must be an http(s) URL without credentials".into());
    }
    Ok(())
}

pub fn validate_package_name(name: &str) -> Result<(), String> {
    if name.is_empty()
        || !name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
        || name == "."
        || name == ".."
    {
        return Err(format!("invalid package name `{name}`"));
    }
    Ok(())
}

fn curl_output(url: &str) -> Result<Vec<u8>, String> {
    let output = Command::new("curl")
        .args([
            "--fail",
            "--silent",
            "--show-error",
            "--location",
            "--max-time",
            "30",
            url,
        ])
        .output()
        .map_err(|error| format!("could not start curl for HTTP registry: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "registry request failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    Ok(output.stdout)
}

fn curl_to_file(url: &str, dest: &Path) -> Result<(), String> {
    let output = Command::new("curl")
        .args([
            "--fail",
            "--silent",
            "--show-error",
            "--location",
            "--max-time",
            "60",
            "--output",
        ])
        .arg(dest)
        .arg(url)
        .output()
        .map_err(|error| format!("could not start curl for HTTP registry: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "registry request failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    Ok(())
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
