//! Dependency resolution against a local registry, plus the loader that lets
//! the Platipus compiler resolve non-path imports to installed packages.

use std::path::{Path, PathBuf};

use crate::lockfile::{LockPackage, Lockfile};
use crate::manifest::Manifest;
use crate::registry::{self, Registry};

/// Looks upward from `start` for a directory containing `p2lt.toml`.
pub fn project_root(start: &Path) -> Option<PathBuf> {
    let mut current = start.to_path_buf();
    loop {
        if current.join("p2lt.toml").exists() {
            return Some(current);
        }
        if !current.pop() {
            return None;
        }
    }
}

/// Installs `name` from the registry (or a local `.libplt` path) into the
/// global Library Store, then updates the manifest and lockfile.
pub fn install(project: &Path, name: &str) -> Result<Manifest, String> {
    let (source_manifest, dest) = if name.ends_with(".libplt") || Path::new(name).exists() {
        let path = Path::new(name);
        let manifest = crate::package::validate(path)?;
        let dest = crate::cache::package_dir(&manifest.name);
        if dest.exists() {
            std::fs::remove_dir_all(&dest).map_err(|e| e.to_string())?;
        }
        crate::package::unpack(path, &dest)?;
        (manifest, dest)
    } else {
        let registry = Registry::for_project(project);
        let source = registry
            .get(name)
            .ok_or_else(|| format!("package `{name}` not found in the registry"))?;
        let _source_manifest = read_manifest(&source)?;
        let dest = crate::cache::package_dir(name);
        if dest.exists() {
            std::fs::remove_dir_all(&dest).map_err(|e| e.to_string())?;
        }
        registry::copy_dir(&source, &dest)?;
        (read_manifest(&dest)?, dest)
    };
    let mut manifest = read_manifest(project)?;
    manifest
        .dependencies
        .insert(source_manifest.name.clone(), version_req(&source_manifest.version));
    write_all(
        project,
        &manifest,
        dest_checksum(&dest)?,
        &source_manifest.name,
        &source_manifest.version,
    )?;
    Ok(manifest)
}

fn version_req(version: &str) -> String {
    // Keep the major.minor compatibility prefix, like `http = "0.1"`.
    let mut parts = version.split('.');
    match (parts.next(), parts.next()) {
        (Some(major), Some(minor)) => format!("{major}.{minor}"),
        _ => version.to_string(),
    }
}

fn dest_checksum(dest: &Path) -> Result<String, String> {
    registry::checksum(dest)
}

/// Installs every dependency listed in the manifest.
pub fn install_all(project: &Path) -> Result<(), String> {
    let manifest = read_manifest(project)?;
    for name in manifest.dependencies.keys().cloned().collect::<Vec<_>>() {
        install(project, &name)?;
    }
    Ok(())
}

pub fn remove(project: &Path, name: &str) -> Result<Manifest, String> {
    let dest = crate::cache::package_dir(name);
    if dest.exists() {
        std::fs::remove_dir_all(&dest).map_err(|e| e.to_string())?;
    }
    let mut manifest = read_manifest(project)?;
    manifest.dependencies.remove(name);
    let mut lock = Lockfile::read(&project.join("p2lt.lock"))?;
    lock.packages.retain(|package| package.name != name);
    write_manifest(project, &manifest)?;
    lock.write(&project.join("p2lt.lock"))?;
    Ok(manifest)
}

/// Re-resolves installed packages: refresh each package directory from the
/// registry and rewrite the lockfile.
pub fn update(project: &Path) -> Result<Lockfile, String> {
    let manifest = read_manifest(project)?;
    let mut lock = Lockfile::default();
    for name in manifest.dependencies.keys() {
        let source = Registry::for_project(project)
            .get(name)
            .ok_or_else(|| format!("package `{name}` not found in the registry"))?;
        let version = read_manifest(&source)?.version;
        let dest = crate::cache::package_dir(name);
        if dest.exists() {
            std::fs::remove_dir_all(&dest).map_err(|e| e.to_string())?;
        }
        registry::copy_dir(&source, &dest)?;
        let checksum = registry::checksum(&dest)?;
        lock.packages.push(LockPackage {
            name: name.clone(),
            version,
            checksum,
        });
    }
    lock.write(&project.join("p2lt.lock"))?;
    Ok(lock)
}

pub fn list_installed(project: &Path) -> Result<Vec<String>, String> {
    let manifest = read_manifest(project)?;
    let mut out: Vec<String> = manifest.dependencies.keys().cloned().collect();
    out.sort();
    Ok(out)
}

fn read_manifest(dir: &Path) -> Result<Manifest, String> {
    let path = dir.join("p2lt.toml");
    let text = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    Manifest::from_toml(&text)
}

fn write_manifest(project: &Path, manifest: &Manifest) -> Result<(), String> {
    std::fs::write(project.join("p2lt.toml"), manifest.to_toml()).map_err(|e| e.to_string())
}

fn write_all(
    project: &Path,
    manifest: &Manifest,
    checksum: String,
    name: &str,
    version: &str,
) -> Result<(), String> {
    write_manifest(project, manifest)?;
    let mut lock = Lockfile::read(&project.join("p2lt.lock"))?;
    lock.packages.retain(|package| package.name != name);
    lock.packages.push(LockPackage {
        name: name.to_string(),
        version: version.to_string(),
        checksum,
    });
    lock.write(&project.join("p2lt.lock"))?;
    Ok(())
}
