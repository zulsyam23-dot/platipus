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
    pub package_id: Option<String>,
    pub platipus_version: Option<String>,
    pub targets: Vec<String>,
    pub entry_points: Vec<String>,
    pub integrity: Option<String>,
}

impl Manifest {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: "0.1.0".to_string(),
            edition: "2026".to_string(),
            dependencies: BTreeMap::new(),
            rust_dependencies: BTreeMap::new(),
            package_id: None,
            platipus_version: None,
            targets: Vec::new(),
            entry_points: Vec::new(),
            integrity: None,
        }
    }

    pub fn to_toml(&self) -> String {
        let mut out = String::new();
        out.push_str("[package]\n");
        out.push_str(&format!("name = {:?}\n", self.name));
        out.push_str(&format!("version = {:?}\n", self.version));
        out.push_str(&format!("edition = {:?}\n", self.edition));
        if let Some(package_id) = &self.package_id {
            out.push_str(&format!("package_id = {package_id:?}\n"));
        }
        if let Some(platipus_version) = &self.platipus_version {
            out.push_str(&format!("platipus_version = {platipus_version:?}\n"));
        }
        if !self.targets.is_empty() {
            out.push_str(&format!("targets = {:?}\n", self.targets));
        }
        if !self.entry_points.is_empty() {
            out.push_str(&format!("entry_points = {:?}\n", self.entry_points));
        }
        if let Some(integrity) = &self.integrity {
            out.push_str(&format!("integrity = {integrity:?}\n"));
        }
        out.push_str("\n[dependencies]\n");
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
        let mut package_id = None;
        let mut platipus_version = None;
        let mut targets = Vec::new();
        let mut entry_points = Vec::new();
        let mut integrity = None;
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
                    "package_id" => package_id = Some(unquote(value)?),
                    "platipus_version" => platipus_version = Some(unquote(value)?),
                    "targets" => targets = parse_string_array(value)?,
                    "entry_points" => entry_points = parse_string_array(value)?,
                    "integrity" => integrity = Some(unquote(value)?),
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
            package_id,
            platipus_version,
            targets,
            entry_points,
            integrity,
        })
    }
}

/// Parses a TOML-ish string array like `["a", "b"]`.
fn parse_string_array(value: &str) -> Result<Vec<String>, String> {
    let trimmed = value.trim();
    let inner = trimmed
        .strip_prefix('[')
        .and_then(|v| v.strip_suffix(']'))
        .ok_or_else(|| format!("expected an array, found `{value}`"))?;
    if inner.trim().is_empty() {
        return Ok(Vec::new());
    }
    inner.split(',').map(|item| unquote(item.trim())).collect()
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
            package_id: None,
            platipus_version: None,
            targets: Vec::new(),
            entry_points: Vec::new(),
            integrity: None,
        };
        let parsed = Manifest::from_toml(&manifest.to_toml()).unwrap();
        assert_eq!(parsed, manifest);
    }
}
