//! Dependency resolution against local and HTTP registries, GitHub repositories,
//! and local `.libplt` archives.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::lockfile::{LockPackage, Lockfile};
use crate::manifest::Manifest;
use crate::registry::{self, Registry};

static TEMP_ID: AtomicU64 = AtomicU64::new(0);

struct TempDir(PathBuf);

impl TempDir {
    fn new(label: &str) -> Result<Self, String> {
        let root = std::env::temp_dir();
        for _ in 0..100 {
            let id = TEMP_ID.fetch_add(1, Ordering::Relaxed);
            let path = root.join(format!("p2lt-{label}-{}-{id}", std::process::id()));
            match std::fs::create_dir(&path) {
                Ok(()) => return Ok(Self(path)),
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(error) => return Err(error.to_string()),
            }
        }
        Err("could not allocate a temporary directory".into())
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[derive(Debug, PartialEq, Eq)]
struct GitHubSource {
    owner: String,
    repository: String,
    reference: Option<String>,
}

impl GitHubSource {
    fn parse(input: &str) -> Result<Option<Self>, String> {
        let normalized = input
            .strip_prefix("https://")
            .or_else(|| input.strip_prefix("http://"))
            .unwrap_or(input);
        let Some(path) = normalized.strip_prefix("github.com/") else {
            return Ok(None);
        };
        let (path, reference) = match path.split_once('#') {
            Some((path, reference)) => (path, Some(reference)),
            None => (path, None),
        };
        let path = path.strip_suffix(".git").unwrap_or(path);
        let mut parts = path.split('/');
        let owner = parts.next().unwrap_or_default();
        let repository = parts.next().unwrap_or_default();
        if parts.next().is_some()
            || !valid_github_segment(owner)
            || !valid_github_segment(repository)
        {
            return Err(format!(
                "expected `github.com/owner/repository[#ref]`, found `{input}`"
            ));
        }
        let reference = reference
            .map(str::to_string)
            .filter(|reference| !reference.is_empty());
        if reference.as_deref().is_some_and(|reference| {
            reference.starts_with('-')
                || reference.chars().any(char::is_whitespace)
                || reference.contains("..")
        }) {
            return Err(format!("invalid GitHub ref in `{input}`"));
        }
        Ok(Some(Self {
            owner: owner.to_string(),
            repository: repository.to_string(),
            reference,
        }))
    }

    fn source(&self) -> String {
        let mut source = format!("github.com/{}/{}", self.owner, self.repository);
        if let Some(reference) = &self.reference {
            source.push('#');
            source.push_str(reference);
        }
        source
    }

    fn clone(&self, locked_revision: Option<&str>) -> Result<(TempDir, String), String> {
        if locked_revision.is_some_and(|revision| {
            !matches!(revision.len(), 40 | 64)
                || !revision.bytes().all(|byte| byte.is_ascii_hexdigit())
        }) {
            return Err("lockfile contains an invalid Git commit revision".into());
        }
        let checkout = TempDir::new("github")?;
        let url = format!("https://github.com/{}/{}.git", self.owner, self.repository);
        let mut command = Command::new("git");
        command.arg("clone").arg("--quiet").arg("--depth").arg("1");
        if locked_revision.is_some() {
            command.arg("--no-checkout");
        } else if let Some(reference) = &self.reference {
            command.arg("--branch").arg(reference);
        }
        let output = command
            .arg(&url)
            .arg(&checkout.0)
            .output()
            .map_err(|error| format!("could not start git to fetch {url}: {error}"))?;
        if !output.status.success() {
            return Err(format!(
                "GitHub clone failed: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            ));
        }
        if let Some(revision) = locked_revision {
            let fetch = Command::new("git")
                .arg("-C")
                .arg(&checkout.0)
                .args(["fetch", "--quiet", "--depth", "1", "origin", revision])
                .output()
                .map_err(|error| format!("could not fetch locked GitHub revision: {error}"))?;
            if !fetch.status.success() {
                return Err(format!(
                    "could not fetch locked GitHub revision: {}",
                    String::from_utf8_lossy(&fetch.stderr).trim()
                ));
            }
            let checkout_revision = Command::new("git")
                .arg("-C")
                .arg(&checkout.0)
                .args(["checkout", "--quiet", "FETCH_HEAD"])
                .output()
                .map_err(|error| format!("could not check out GitHub revision: {error}"))?;
            if !checkout_revision.status.success() {
                return Err(format!(
                    "could not check out GitHub revision: {}",
                    String::from_utf8_lossy(&checkout_revision.stderr).trim()
                ));
            }
        }
        let output = Command::new("git")
            .arg("-C")
            .arg(&checkout.0)
            .args(["rev-parse", "HEAD"])
            .output()
            .map_err(|error| format!("could not read GitHub revision: {error}"))?;
        if !output.status.success() {
            return Err(format!(
                "could not read GitHub revision: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            ));
        }
        let revision = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Ok((checkout, revision))
    }
}

fn valid_github_segment(value: &str) -> bool {
    !value.is_empty()
        && value != "."
        && value != ".."
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
}

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

/// Installs a package by name, GitHub source, or local `.libplt` archive.
pub fn install(project: &Path, identifier: &str) -> Result<Manifest, String> {
    if identifier.ends_with(".libplt") || Path::new(identifier).exists() {
        let archive = Path::new(identifier);
        let manifest = crate::package::validate(archive)?;
        registry::validate_package_name(&manifest.name)?;
        let temp = TempDir::new("archive")?;
        crate::package::unpack(archive, &temp.0)?;
        return finish_install(project, &temp.0, manifest, None, None);
    }

    if let Some(source) = GitHubSource::parse(identifier)? {
        return install_github(project, &source);
    }

    let manifest = if let Some(base) = Registry::http_url() {
        install_http(project, identifier, &base)?
    } else {
        let local_registry = Registry::for_project(project);
        let source = local_registry
            .get(identifier)
            .ok_or_else(|| format!("package `{identifier}` not found in the local registry"))?;
        let manifest = read_manifest(&source)?;
        registry::validate_package_name(&manifest.name)?;
        if manifest.name != identifier {
            return Err(format!(
                "local registry package `{identifier}` contains manifest for `{}`",
                manifest.name
            ));
        }
        finish_install(project, &source, manifest, None, None)?
    };
    Ok(manifest)
}

fn install_github(project: &Path, source: &GitHubSource) -> Result<Manifest, String> {
    install_github_at(project, source, None)
}

fn install_github_at(
    project: &Path,
    source: &GitHubSource,
    locked_revision: Option<&str>,
) -> Result<Manifest, String> {
    let (checkout, revision) = source.clone(locked_revision)?;
    let manifest = read_manifest(&checkout.0)?;
    registry::validate_package_name(&manifest.name)?;
    finish_install(
        project,
        &checkout.0,
        manifest,
        Some(source.source()),
        Some(revision),
    )
}

fn install_http(project: &Path, name: &str, base: &str) -> Result<Manifest, String> {
    let temp = TempDir::new("registry")?;
    let archive = temp.0.join("package.libplt");
    Registry::http_download_from(base, name, &archive)?;
    let manifest = crate::package::validate(&archive)?;
    registry::validate_package_name(&manifest.name)?;
    if manifest.name != name {
        return Err(format!(
            "registry package `{name}` contains manifest for `{}`",
            manifest.name
        ));
    }
    let unpacked = temp.0.join("unpacked");
    crate::package::unpack(&archive, &unpacked)?;
    finish_install(
        project,
        &unpacked,
        manifest,
        Some(format!("registry:{}", base.trim_end_matches('/'))),
        None,
    )
}

fn finish_install(
    project: &Path,
    source_dir: &Path,
    source_manifest: Manifest,
    source: Option<String>,
    revision: Option<String>,
) -> Result<Manifest, String> {
    let name = source_manifest.name.as_str();
    let packages = crate::cache::packages_dir();
    std::fs::create_dir_all(&packages).map_err(|error| error.to_string())?;
    let stage = unique_child(&packages, &format!(".{name}-stage"))?;
    let _stage_cleanup = TempDir(stage.clone());
    copy_package_payload(source_dir, &stage)?;
    let staged_manifest = read_manifest(&stage)?;
    if staged_manifest.name != name || staged_manifest.version != source_manifest.version {
        let _ = std::fs::remove_dir_all(&stage);
        return Err("package manifest changed while installing".into());
    }
    let dest = crate::cache::package_dir(name);
    replace_directory(&stage, &dest)?;

    let mut manifest = read_manifest(project)?;
    manifest.dependencies.insert(
        name.to_string(),
        if source.is_some() {
            source_manifest.version.clone()
        } else {
            version_req(&source_manifest.version)
        },
    );
    write_all(
        project,
        &manifest,
        dest_checksum(&dest)?,
        name,
        &source_manifest.version,
        source,
        revision,
    )?;
    Ok(manifest)
}

fn unique_child(parent: &Path, prefix: &str) -> Result<PathBuf, String> {
    for _ in 0..100 {
        let id = TEMP_ID.fetch_add(1, Ordering::Relaxed);
        let path = parent.join(format!("{prefix}-{}-{id}", std::process::id()));
        match std::fs::create_dir(&path) {
            Ok(()) => return Ok(path),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error.to_string()),
        }
    }
    Err("could not allocate a staging directory".into())
}

fn copy_package_payload(source: &Path, dest: &Path) -> Result<(), String> {
    for entry in ["p2lt.toml", "src", "assets", "rust", "tests", "README.md"] {
        let from = source.join(entry);
        if !from.exists() {
            continue;
        }
        let to = dest.join(entry);
        if from.is_dir() {
            registry::copy_dir(&from, &to)?;
        } else {
            std::fs::copy(&from, &to).map_err(|error| error.to_string())?;
        }
    }
    if !dest.join("p2lt.toml").exists() {
        return Err("package source is missing p2lt.toml".into());
    }
    if !dest.join("src").join("lib.plt").exists() && !dest.join("src").join("main.plt").exists() {
        return Err("package source is missing src/lib.plt or src/main.plt".into());
    }
    Ok(())
}

fn replace_directory(stage: &Path, dest: &Path) -> Result<(), String> {
    let had_old = dest.exists();
    let backup = if had_old {
        let file_name = dest
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or("invalid package destination name")?;
        let mut backup = None;
        for _ in 0..100 {
            let id = TEMP_ID.fetch_add(1, Ordering::Relaxed);
            let candidate =
                dest.with_file_name(format!(".{file_name}-backup-{}-{id}", std::process::id()));
            if !candidate.exists() {
                backup = Some(candidate);
                break;
            }
        }
        Some(backup.ok_or("could not allocate a package backup path")?)
    } else {
        None
    };
    if had_old {
        let backup = backup.as_ref().ok_or("package backup path is missing")?;
        std::fs::rename(dest, backup).map_err(|error| error.to_string())?;
    }
    if let Err(error) = std::fs::rename(stage, dest) {
        if let Some(backup) = &backup {
            let _ = std::fs::rename(backup, dest);
        }
        let _ = std::fs::remove_dir_all(stage);
        return Err(error.to_string());
    }
    if let Some(backup) = backup {
        std::fs::remove_dir_all(backup).map_err(|error| error.to_string())?;
    }
    Ok(())
}

fn version_req(version: &str) -> String {
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
    let lock = Lockfile::read(&project.join("p2lt.lock"))?;
    for name in manifest.dependencies.keys() {
        if let Some(package) = lock.packages.iter().find(|package| package.name == *name) {
            if let Some(source) = &package.source {
                if let Some(github) = GitHubSource::parse(source)? {
                    install_github_at(project, &github, package.revision.as_deref())?;
                } else if let Some(base) = source.strip_prefix("registry:") {
                    install_http(project, name, base)?;
                } else {
                    install(project, name)?;
                }
                continue;
            }
        }
        install(project, name)?;
    }
    Ok(())
}

pub fn remove(project: &Path, name: &str) -> Result<Manifest, String> {
    registry::validate_package_name(name)?;
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

/// Refreshes all direct dependencies from their recorded source.
pub fn update(project: &Path) -> Result<Lockfile, String> {
    let manifest = read_manifest(project)?;
    let old_lock = Lockfile::read(&project.join("p2lt.lock"))?;
    let mut lock = Lockfile::default();
    for name in manifest.dependencies.keys() {
        let old = old_lock
            .packages
            .iter()
            .find(|package| package.name == *name);
        let source = old.and_then(|package| package.source.as_deref());
        let installed = if let Some(source) = source {
            if let Some(github) = GitHubSource::parse(source)? {
                install_github(project, &github)?;
            } else if let Some(base) = source.strip_prefix("registry:") {
                install_http(project, name, base)?;
            } else {
                install(project, name)?;
            }
            Lockfile::read(&project.join("p2lt.lock"))?
                .packages
                .into_iter()
                .find(|package| package.name == *name)
                .ok_or_else(|| format!("updated lock entry for `{name}` is missing"))?
        } else if Registry::http_url().is_some() {
            install(project, name)?;
            Lockfile::read(&project.join("p2lt.lock"))?
                .packages
                .into_iter()
                .find(|package| package.name == *name)
                .ok_or_else(|| format!("updated lock entry for `{name}` is missing"))?
        } else {
            let registry = Registry::for_project(project);
            let directory = registry
                .get(name)
                .ok_or_else(|| format!("package `{name}` not found in the local registry"))?;
            let source_manifest = read_manifest(&directory)?;
            let package_dir = crate::cache::package_dir(name);
            let packages = crate::cache::packages_dir();
            std::fs::create_dir_all(&packages).map_err(|error| error.to_string())?;
            let stage = unique_child(&packages, &format!(".{name}-stage"))?;
            let _stage_cleanup = TempDir(stage.clone());
            copy_package_payload(&directory, &stage)?;
            replace_directory(&stage, &package_dir)?;
            LockPackage {
                name: name.clone(),
                version: source_manifest.version,
                checksum: dest_checksum(&package_dir)?,
                source: None,
                revision: None,
            }
        };
        lock.packages.push(installed);
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
    // A raw io error here says "the system cannot find the file specified",
    // which is true and useless: the reader does not know whether they cloned
    // the wrong repository or pointed at a directory that is not a package. Say
    // which directory was checked.
    let text = std::fs::read_to_string(&path).map_err(|error| {
        format!(
            "no p2lt.toml in {}: {error}",
            dir.display()
        )
    })?;
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
    source: Option<String>,
    revision: Option<String>,
) -> Result<(), String> {
    write_manifest(project, manifest)?;
    let mut lock = Lockfile::read(&project.join("p2lt.lock"))?;
    lock.packages.retain(|package| package.name != name);
    lock.packages.push(LockPackage {
        name: name.to_string(),
        version: version.to_string(),
        checksum,
        source,
        revision,
    });
    lock.write(&project.join("p2lt.lock"))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_github_repository_and_ref() {
        assert_eq!(
            GitHubSource::parse("github.com/user/repo#main").unwrap(),
            Some(GitHubSource {
                owner: "user".into(),
                repository: "repo".into(),
                reference: Some("main".into()),
            })
        );
    }

    #[test]
    fn accepts_github_url_with_git_suffix() {
        assert_eq!(
            GitHubSource::parse("https://github.com/user/repo.git").unwrap(),
            Some(GitHubSource {
                owner: "user".into(),
                repository: "repo".into(),
                reference: None,
            })
        );
    }

    #[test]
    fn rejects_unsafe_github_paths_and_refs() {
        assert!(GitHubSource::parse("github.com/user/../repo").is_err());
        assert!(GitHubSource::parse("github.com/user/repo#--upload-pack=x").is_err());
    }
}
