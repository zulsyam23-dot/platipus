use std::path::{Path, PathBuf};
use std::process::ExitCode;

use crate::config::Options;
use crate::CliError;

// A blank line next to a brace is dropped by the formatter, which owns the
// layout of a brace line, so the template leaves none: `new` then writes exactly
// what `format` would write for the same program.
const APP: &str = r#"app {component} {
    state count = 0
    derived doubled = count * 2

    Column {
        Text count
        Text doubled

        Button "+" {
            on click {
                count = count + 1
            }
        }
        Button "reset" {
            on click {
                count = 0
            }
        }
    }
}
test startsAtZero {
    expect count == 0
}
test clickingAddsOne {
    click "+"
    expect count == 1
    expect doubled == 2
}
"#;

const README: &str = r#"# {name}

A Platipus program.

## Layout

- `app.plt` is the entry program.
- `dist/` is generated; it is safe to delete.

## Commands

    plt build app.plt     compile into dist/
    plt test app.plt      run the `test` blocks in app.plt
    plt dev app.plt       serve dist/ and rebuild on every change
    plt format app.plt    normalise the indentation

`app.plt` already declares two tests, so `plt test app.plt` passes as it is.
"#;

const GITIGNORE: &str = "dist/\n";

pub fn run(options: &Options) -> Result<ExitCode, CliError> {
    let name = options
        .subject
        .as_deref()
        .ok_or_else(|| CliError::Usage("`new` needs a project name".to_string()))?;
    let created = scaffold(Path::new(name), name)?;
    if !options.quiet {
        for path in &created {
            println!("wrote {}", path.display());
        }
        println!();
        println!("created {name}/ with a working counter and two passing tests");
        println!();
        println!("    cd {name}");
        println!("    plt test app.plt");
    }
    Ok(ExitCode::SUCCESS)
}

/// A project name becomes a directory, so it has to survive being a path, and
/// it also becomes a component name, so it has to survive being an identifier.
pub fn valid_name(name: &str) -> Result<(), CliError> {
    if name.is_empty() {
        return Err(CliError::Usage("a project name cannot be empty".to_string()));
    }
    if name.starts_with('-') {
        return Err(CliError::Usage(format!(
            "`{name}` starts with `-`, which reads as an option"
        )));
    }
    if name == "." || name == ".." {
        return Err(CliError::Usage(format!("`{name}` is not a name")));
    }
    if let Some(bad) = name
        .chars()
        .find(|c| !c.is_ascii_alphanumeric() && *c != '_' && *c != '-')
    {
        return Err(CliError::Usage(format!(
            "`{name}` contains `{bad}`; use letters, digits, `-`, or `_`"
        )));
    }
    Ok(())
}

/// Turns `my-app` into `MyApp`, because the project name is reused as a
/// component name and components are identifiers.
pub fn component_name(name: &str) -> String {
    let mut out = String::new();
    let mut capitalise = true;
    for ch in name.chars() {
        if ch == '-' || ch == '_' {
            capitalise = true;
            continue;
        }
        if capitalise {
            out.extend(ch.to_uppercase());
            capitalise = false;
        } else {
            out.push(ch);
        }
    }
    if out.is_empty() || out.starts_with(|c: char| c.is_ascii_digit()) {
        out.insert(0, 'A');
        out.insert(1, 'p');
        out.insert(2, 'p');
    }
    out
}

fn scaffold(dir: &Path, name: &str) -> Result<Vec<PathBuf>, CliError> {
    valid_name(name)?;
    if dir.exists() {
        let mut entries = std::fs::read_dir(dir)
            .map_err(|error| CliError::Io(format!("cannot read `{}`: {error}", dir.display())))?
            .peekable();
        if entries.peek().is_some() {
            return Err(CliError::Io(format!(
                "`{}` already exists and is not empty",
                dir.display()
            )));
        }
    }
    let component = component_name(name);
    let mut created = Vec::new();
    for (file, contents) in [
        ("app.plt", APP.replace("{component}", &component)),
        ("README.md", README.replace("{name}", name)),
        (".gitignore", GITIGNORE.to_string()),
    ] {
        let path = dir.join(file);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|error| {
                CliError::Io(format!("cannot create `{}`: {error}", parent.display()))
            })?;
        }
        std::fs::write(&path, contents)
            .map_err(|error| CliError::Io(format!("cannot write `{}`: {error}", path.display())))?;
        created.push(path);
    }
    Ok(created)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_component_name_starts_with_a_capital() {
        assert_eq!(component_name("my-app"), "MyApp");
        assert_eq!(component_name("counter"), "Counter");
        assert_eq!(component_name("my_thing"), "MyThing");
    }

    #[test]
    fn a_component_name_never_starts_with_a_digit() {
        assert_eq!(component_name("2fast"), "App2fast");
    }

    #[test]
    fn names_that_would_escape_or_confuse_are_rejected() {
        assert!(valid_name("my-app").is_ok());
        assert!(valid_name("my_app").is_ok());
        assert!(valid_name("").is_err());
        assert!(valid_name("..").is_err());
        assert!(valid_name("../evil").is_err());
        assert!(valid_name("with/slash").is_err());
        assert!(valid_name("-q").is_err());
    }
}
