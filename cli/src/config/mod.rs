//! Command line options: what the user asked for, after parsing.

use std::path::PathBuf;

use super::command::Command;
use super::CliError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Options {
    pub command: Command,
    /// The entry program for the commands that compile one.
    pub entry: PathBuf,
    /// The argument for commands that take a name rather than a file, such as
    /// `new`.
    pub subject: Option<String>,
    pub out_dir: PathBuf,
    pub port: u16,
    pub quiet: bool,
    pub check_only: bool,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            command: Command::Help,
            entry: PathBuf::new(),
            subject: None,
            out_dir: PathBuf::from("dist"),
            port: 8080,
            quiet: false,
            check_only: false,
        }
    }
}

impl Options {
    /// The commands that compile a single entry file, and therefore need one.
    pub fn needs_entry(&self) -> bool {
        matches!(
            self.command,
            Command::Build | Command::Check | Command::Run | Command::Test | Command::Dev | Command::Format
        )
    }
}

pub fn usage(program: &str, version: &str) -> String {
    format!(
        "\
{program} {version}
A compiler for the Platipus language.

USAGE:
    {program} <COMMAND> [OPTIONS]

COMMANDS:
    new <name>           scaffold a project with an app.plt, a README, and a .gitignore
    build <entry.plt>    compile the program into the output directory
    check <entry.plt>    run every compiler stage without writing files
    run <entry.plt>      build, then print how to serve the result
    test <entry.plt>     build, then run the program's `test` blocks
    dev <entry.plt>      build, serve, and rebuild when any watched file changes
    format <entry.plt>   rewrite the file with normalised indentation
    help                 show this message
    version              show the compiler version

OPTIONS:
    -o, --out <DIR>      output directory (default: dist)
    -p, --port <PORT>    port for `dev` (default: 8080)
    -q, --quiet          only report failures
        --check          analyse only, do not write artifacts
    -h, --help           show this message

`dev` watches the entry file and everything it imports transitively.
"
    )
}

pub fn parse(program: &str, args: &[String]) -> Result<Options, CliError> {
    let mut options = Options::default();
    let mut positional: Vec<String> = Vec::new();
    let mut index = 0usize;
    while index < args.len() {
        let arg = args[index].as_str();
        match arg {
            "-h" | "--help" => {
                options.command = Command::Help;
                return Ok(options);
            }
            "-v" | "--version" => {
                options.command = Command::Version;
                return Ok(options);
            }
            "-q" | "--quiet" => options.quiet = true,
            "--check" => options.check_only = true,
            "-o" | "--out" => {
                index += 1;
                let value = args.get(index).ok_or_else(|| {
                    CliError::Usage(format!("{program}: `{arg}` needs a directory"))
                })?;
                options.out_dir = PathBuf::from(value);
            }
            other if other.starts_with("--out=") => {
                options.out_dir = PathBuf::from(&other[6..]);
            }
            "-p" | "--port" => {
                index += 1;
                let value = args.get(index).ok_or_else(|| {
                    CliError::Usage(format!("{program}: `{arg}` needs a port number"))
                })?;
                options.port = value.parse().map_err(|_| {
                    CliError::Usage(format!("{program}: `{value}` is not a port number"))
                })?;
            }
            other if other.starts_with("--port=") => {
                options.port = other[7..].parse().map_err(|_| {
                    CliError::Usage(format!(
                        "{program}: `{}` is not a port number",
                        &other[7..]
                    ))
                })?;
            }
            other if other.starts_with('-') && other.len() > 1 => {
                return Err(CliError::Usage(format!(
                    "{program}: unknown option `{other}`"
                )));
            }
            other => positional.push(other.to_string()),
        }
        index += 1;
    }
    let command = Command::from_word(positional.first().map(String::as_str));
    let command = match command {
        Some(command) => command,
        None if positional.is_empty() => Command::Help,
        None => {
            return Err(CliError::Usage(format!(
                "{program}: unknown command `{}`",
                positional[0]
            )));
        }
    };
    options.command = command;
    if command == Command::New {
        let name = positional.get(1).ok_or_else(|| {
            CliError::Usage(format!("{program}: `new` needs a project name"))
        })?;
        options.subject = Some(name.clone());
    } else if options.needs_entry() {
        let entry = positional.get(1).ok_or_else(|| {
            CliError::Usage(format!(
                "{program}: `{}` needs an entry file",
                command.word()
            ))
        })?;
        options.entry = PathBuf::from(entry);
    }
    if options.check_only {
        options.command = Command::Check;
    }
    Ok(options)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(items: &[&str]) -> Vec<String> {
        items.iter().map(|item| item.to_string()).collect()
    }

    #[test]
    fn parses_a_build_command() {
        let options = parse("platipus", &args(&["build", "app.plt"])).unwrap();
        assert_eq!(options.command, Command::Build);
        assert_eq!(options.entry, PathBuf::from("app.plt"));
        assert_eq!(options.out_dir, PathBuf::from("dist"));
    }

    #[test]
    fn parses_an_output_directory() {
        let options = parse("plt", &args(&["build", "app.plt", "-o", "out"])).unwrap();
        assert_eq!(options.out_dir, PathBuf::from("out"));
    }

    #[test]
    fn parses_an_inline_output_directory() {
        let options = parse("plt", &args(&["build", "app.plt", "--out=out"])).unwrap();
        assert_eq!(options.out_dir, PathBuf::from("out"));
    }

    #[test]
    fn parses_a_port() {
        let options = parse("plt", &args(&["dev", "app.plt", "-p", "3000"])).unwrap();
        assert_eq!(options.port, 3000);
        let options = parse("plt", &args(&["dev", "app.plt", "--port=3001"])).unwrap();
        assert_eq!(options.port, 3001);
    }

    #[test]
    fn a_bad_port_is_a_usage_error() {
        assert!(parse("platipus", &args(&["dev", "app.plt", "-p", "nope"])).is_err());
    }

    #[test]
    fn new_takes_a_name_rather_than_a_file() {
        let options = parse("platipus", &args(&["new", "my-app"])).unwrap();
        assert_eq!(options.command, Command::New);
        assert_eq!(options.subject.as_deref(), Some("my-app"));
        assert!(options.entry.as_os_str().is_empty());
    }

    #[test]
    fn new_without_a_name_is_a_usage_error() {
        assert!(parse("platipus", &args(&["new"])).is_err());
    }

    #[test]
    fn check_flag_downgrades_build() {
        let options = parse("platipus", &args(&["build", "app.plt", "--check"])).unwrap();
        assert_eq!(options.command, Command::Check);
    }

    #[test]
    fn no_arguments_prints_help() {
        let options = parse("platipus", &[]).unwrap();
        assert_eq!(options.command, Command::Help);
    }

    #[test]
    fn missing_entry_is_a_usage_error() {
        assert!(parse("platipus", &args(&["build"])).is_err());
    }

    #[test]
    fn unknown_command_is_a_usage_error() {
        assert!(parse("platipus", &args(&["frobnicate", "app.plt"])).is_err());
    }

    #[test]
    fn unknown_option_is_a_usage_error() {
        assert!(parse("platipus", &args(&["build", "app.plt", "--wat"])).is_err());
    }

    #[test]
    fn help_lists_every_command() {
        let text = usage("platipus", "0.1.0");
        for command in [
            "new", "build", "check", "run", "test", "dev", "format", "help", "version",
        ] {
            assert!(text.contains(command), "{command} missing from {text}");
        }
    }
}
