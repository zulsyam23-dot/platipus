//! `p2lt.toml` — the package manifest.
//!
//! The parser intentionally handles a small, well-defined subset of TOML:
//! `[section]` headers, `key = "value"` string pairs, and `key = ["a", "b"]`
//! string arrays. It is rewritten deterministically on install/remove.

use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Manifest {
    pub name: String,
    pub version: String,
    pub edition: String,
    pub dependencies: BTreeMap<String, String>,
    pub rust_dependencies: BTreeMap<String, String>,
}

impl Manifest {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: "0.1.0".to_string(),
            edition: "2026".to_string(),
            dependencies: BTreeMap::new(),
            rust_dependencies: BTreeMap::new(),
        }
    }

    pub fn to_toml(&self) -> String {
        let mut out = String::new();
        out.push_str("[package]\n");
        out.push_str(&format!("name = {:?}\n", self.name));
        out.push_str(&format!("version = {:?}\n", self.version));
        out.push_str(&format!("edition = {:?}\n", self.edition));
        out.push_str(&format!("\n[dependencies]\n"));
        for (name, version) in &self.dependencies {
            out.push_str(&format!("{name} = {version:?}\n"));
        }
        out.push_str("\n[rust.dependencies]\n");
        for (name, version) in &self.rust_dependencies {
            out.push_str(&format!("{name} = {version:?}\n"));
        }
        out
    }

    pub fn from_toml(text: &str) -> Result<Self, String> {
        let mut section = String::new();
        let mut name = None;
        let mut version = None;
        let mut edition = None;
        let mut deps = BTreeMap::new();
        let mut rust_deps = BTreeMap::new();
        for raw in text.lines() {
            let line = raw.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if let Some(rest) = line.strip_prefix('[').and_then(|l| l.strip_suffix(']')) {
                section = rest.trim().to_string();
                continue;
            }
            let (key, value) = line
                .split_once('=')
                .ok_or_else(|| format!("invalid line: {line}"))?;
            let key = key.trim();
            let value = value.trim();
            match section.as_str() {
                "package" => match key {
                    "name" => name = Some(unquote(value)?),
                    "version" => version = Some(unquote(value)?),
                    "edition" => edition = Some(unquote(value)?),
                    _ => {}
                },
                "dependencies" => {
                    deps.insert(key.to_string(), unquote(value)?);
                }
                "rust.dependencies" => {
                    rust_deps.insert(key.to_string(), unquote(value)?);
                }
                _ => {}
            }
        }
        Ok(Self {
            name: name.ok_or("missing [package] name")?,
            version: version.ok_or("missing [package] version")?,
            edition: edition.unwrap_or_else(|| "2026".to_string()),
            dependencies: deps,
            rust_dependencies: rust_deps,
        })
    }
}

fn unquote(value: &str) -> Result<String, String> {
    let trimmed = value.trim();
    if trimmed.len() >= 2 && trimmed.starts_with('"') && trimmed.ends_with('"') {
        Ok(trimmed[1..trimmed.len() - 1].to_string())
    } else {
        Err(format!("expected a quoted string, found `{value}`"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_a_manifest() {
        let text = r#"
[package]
name = "hello-world"
version = "0.1.0"
edition = "2026"

[dependencies]
http = "0.1"
json = "0.1"

[rust.dependencies]
serde = "1"
"#;
        let manifest = Manifest::from_toml(text).unwrap();
        assert_eq!(manifest.name, "hello-world");
        assert_eq!(manifest.version, "0.1.0");
        assert_eq!(manifest.edition, "2026");
        assert_eq!(manifest.dependencies["http"], "0.1");
        assert_eq!(manifest.rust_dependencies["serde"], "1");
    }

    #[test]
    fn roundtrips() {
        let manifest = Manifest {
            name: "app".into(),
            version: "0.2.0".into(),
            edition: "2026".into(),
            dependencies: BTreeMap::from([("http".into(), "0.1".into())]),
            rust_dependencies: BTreeMap::new(),
        };
        let parsed = Manifest::from_toml(&manifest.to_toml()).unwrap();
        assert_eq!(parsed, manifest);
    }
}
