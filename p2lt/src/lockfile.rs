//! `p2lt.lock` — the deterministic lockfile.

use std::collections::BTreeMap;
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LockPackage {
    pub name: String,
    pub version: String,
    pub checksum: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Lockfile {
    pub packages: Vec<LockPackage>,
}

impl Lockfile {
    pub fn to_toml(&self) -> String {
        let mut out = String::new();
        let mut packages = self.packages.clone();
        packages.sort_by(|a, b| a.name.cmp(&b.name));
        for package in &packages {
            out.push_str("[[package]]\n");
            out.push_str(&format!("name = {:?}\n", package.name));
            out.push_str(&format!("version = {:?}\n", package.version));
            out.push_str(&format!("checksum = {:?}\n", package.checksum));
            out.push('\n');
        }
        out
    }

    pub fn from_toml(text: &str) -> Result<Self, String> {
        let mut packages = Vec::new();
        let mut current: BTreeMap<String, String> = BTreeMap::new();
        for raw in text.lines() {
            let line = raw.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if line == "[[package]]" {
                if !current.is_empty() {
                    packages.push(package_from(current)?);
                    current = BTreeMap::new();
                }
                continue;
            }
            let (key, value) = line
                .split_once('=')
                .ok_or_else(|| format!("invalid line: {line}"))?;
            current.insert(key.trim().to_string(), value.trim().trim_matches('"').to_string());
        }
        if !current.is_empty() {
            packages.push(package_from(current)?);
        }
        Ok(Self { packages })
    }

    pub fn read(path: &Path) -> Result<Self, String> {
        match std::fs::read_to_string(path) {
            Ok(text) => Self::from_toml(&text),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Self::default()),
            Err(e) => Err(e.to_string()),
        }
    }

    pub fn write(&self, path: &Path) -> Result<(), String> {
        std::fs::write(path, self.to_toml()).map_err(|e| e.to_string())
    }
}

fn package_from(map: BTreeMap<String, String>) -> Result<LockPackage, String> {
    Ok(LockPackage {
        name: map.get("name").cloned().ok_or("lock entry missing name")?,
        version: map.get("version").cloned().ok_or("lock entry missing version")?,
        checksum: map
            .get("checksum")
            .cloned()
            .ok_or("lock entry missing checksum")?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrips() {
        let lock = Lockfile {
            packages: vec![
                LockPackage {
                    name: "http".into(),
                    version: "0.1.2".into(),
                    checksum: "abc".into(),
                },
                LockPackage {
                    name: "json".into(),
                    version: "0.1.5".into(),
                    checksum: "def".into(),
                },
            ],
        };
        let parsed = Lockfile::from_toml(&lock.to_toml()).unwrap();
        assert_eq!(parsed, lock);
        assert_eq!(lock.to_toml(), lock.to_toml());
    }
}
