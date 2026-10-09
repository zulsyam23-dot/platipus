//! `.libplt` — the distributable package archive.
//!
//! Format (all integers little-endian):
//!
//! ```text
//! magic[4]            "PLT1"
//! manifest_len: u32
//! manifest: bytes     (p2lt.toml contents, integrity field filled in)
//! entry_count: u32
//! per entry:
//!   path_len: u16
//!   path: bytes
//!   data_len: u64
//!   data: bytes
//! payload_checksum: u64   FNV-1a over every entry path + data, in order
//! ```
//!
//! Paths are relative POSIX-style paths inside the archive. Entries are
//! sorted by path so the archive is deterministic.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::manifest::Manifest;
use crate::registry::checksum;

const MAGIC: &[u8; 4] = b"PLT1";
type ArchiveContents = (Manifest, BTreeMap<String, Vec<u8>>, bool);

/// Creates `<out_dir>/<name>-<version>.libplt` from a package directory.
/// The directory must contain a `p2lt.toml`. The manifest's `integrity`
/// field is recomputed before packing.
pub fn pack(project: &Path, out_dir: &Path) -> Result<PathBuf, String> {
    let manifest_path = project.join("p2lt.toml");
    let text = std::fs::read_to_string(&manifest_path).map_err(|e| e.to_string())?;
    let mut manifest = Manifest::from_toml(&text)?;

    let payload_checksum = checksum(project)?;
    manifest.integrity = Some(format!("fnv1a64:{payload_checksum}"));

    let mut entries = BTreeMap::new();
    collect(project, project, &mut entries)?;
    // The package itself is regenerated on unpack; do not ship local state.
    entries.remove("p2lt.toml");

    let manifest_bytes = manifest.to_toml().into_bytes();
    let mut out = Vec::new();
    out.extend_from_slice(MAGIC);
    out.extend_from_slice(&(manifest_bytes.len() as u32).to_le_bytes());
    out.extend_from_slice(&manifest_bytes);
    out.extend_from_slice(&(entries.len() as u32).to_le_bytes());
    let mut entry_checksum: u64 = 0xcbf29ce484222325;
    for (path, data) in &entries {
        out.extend_from_slice(&(path.len() as u16).to_le_bytes());
        out.extend_from_slice(path.as_bytes());
        out.extend_from_slice(&(data.len() as u64).to_le_bytes());
        out.extend_from_slice(data);
        let mut h = entry_checksum;
        for b in path.bytes().chain(data.iter().copied()) {
            h ^= b as u64;
            h = h.wrapping_mul(0x100000001b3);
        }
        entry_checksum = h;
    }
    out.extend_from_slice(&entry_checksum.to_le_bytes());

    std::fs::create_dir_all(out_dir).map_err(|e| e.to_string())?;
    let dest = out_dir.join(format!("{}-{}.libplt", manifest.name, manifest.version));
    std::fs::write(&dest, &out).map_err(|e| e.to_string())?;
    validate(&dest)?;
    Ok(dest)
}

/// Reads and checks an archive without installing it. Returns the manifest.
pub fn validate(path: &Path) -> Result<Manifest, String> {
    let (manifest, _, ok) = read_archive(path)?;
    if !ok {
        return Err("payload checksum mismatch".into());
    }
    if manifest.name.trim().is_empty() {
        return Err("manifest is missing a package name".into());
    }
    if manifest.version.trim().is_empty() {
        return Err("manifest is missing a package version".into());
    }
    Ok(manifest)
}

/// Extracts an archive into `dest`, returning the embedded manifest.
pub fn unpack(path: &Path, dest: &Path) -> Result<Manifest, String> {
    let (manifest, entries, ok) = read_archive(path)?;
    if !ok {
        return Err("payload checksum mismatch".into());
    }
    std::fs::create_dir_all(dest).map_err(|e| e.to_string())?;
    std::fs::write(dest.join("p2lt.toml"), manifest.to_toml()).map_err(|e| e.to_string())?;
    for (path, data) in entries {
        let file = dest.join(&path);
        if let Some(parent) = file.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        std::fs::write(&file, &data).map_err(|e| e.to_string())?;
    }
    Ok(manifest)
}

fn read_archive(path: &Path) -> Result<ArchiveContents, String> {
    let bytes = std::fs::read(path).map_err(|e| e.to_string())?;
    let mut cursor = 0usize;
    let take = |cursor: &mut usize, n: usize| -> Result<&[u8], String> {
        if *cursor + n > bytes.len() {
            return Err("truncated archive".into());
        }
        let slice = &bytes[*cursor..*cursor + n];
        *cursor += n;
        Ok(slice)
    };
    if take(&mut cursor, 4)? != MAGIC {
        return Err("not a .libplt archive (bad magic)".into());
    }
    let manifest_len = u32::from_le_bytes(take(&mut cursor, 4)?.try_into().unwrap()) as usize;
    let manifest_bytes = take(&mut cursor, manifest_len)?;
    let manifest = Manifest::from_toml(
        std::str::from_utf8(manifest_bytes).map_err(|_| "manifest is not UTF-8".to_string())?,
    )?;
    let entry_count = u32::from_le_bytes(take(&mut cursor, 4)?.try_into().unwrap()) as usize;
    let mut entries = BTreeMap::new();
    let mut entry_checksum: u64 = 0xcbf29ce484222325;
    for _ in 0..entry_count {
        let path_len = u16::from_le_bytes(take(&mut cursor, 2)?.try_into().unwrap()) as usize;
        let path = std::str::from_utf8(take(&mut cursor, path_len)?)
            .map_err(|_| "entry path is not UTF-8".to_string())?
            .to_string();
        if path.contains("..") || path.starts_with('/') || path.contains('\\') {
            return Err(format!("unsafe entry path `{path}`"));
        }
        let data_len = u64::from_le_bytes(take(&mut cursor, 8)?.try_into().unwrap()) as usize;
        let data = take(&mut cursor, data_len)?.to_vec();
        let mut h = entry_checksum;
        for b in path.bytes().chain(data.iter().copied()) {
            h ^= b as u64;
            h = h.wrapping_mul(0x100000001b3);
        }
        entry_checksum = h;
        entries.insert(path, data);
    }
    let stored = u64::from_le_bytes(take(&mut cursor, 8)?.try_into().unwrap());
    if cursor != bytes.len() {
        return Err("trailing bytes after archive".into());
    }
    Ok((manifest, entries, stored == entry_checksum))
}

fn collect(root: &Path, dir: &Path, out: &mut BTreeMap<String, Vec<u8>>) -> Result<(), String> {
    for entry in std::fs::read_dir(dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        let name = entry.file_name();
        if name == ".p2lt" || name == "target" || name == "dist" || name == ".git" {
            continue;
        }
        if path.is_dir() {
            collect(root, &path, out)?;
        } else {
            let relative = path
                .strip_prefix(root)
                .map_err(|e| e.to_string())?
                .to_string_lossy()
                .replace('\\', "/");
            out.insert(relative, std::fs::read(&path).map_err(|e| e.to_string())?);
        }
    }
    Ok(())
}
