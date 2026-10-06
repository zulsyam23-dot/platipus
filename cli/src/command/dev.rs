use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::{Duration, SystemTime};

use crate::command::build;
use crate::config::Options;
use crate::output::write_artifacts;
use crate::CliError;

/// The path a browser listens on for reload notifications.
const EVENTS_PATH: &str = "/__plt/events";
const MARKER: &str = "<script data-platipus-reload";
const POLL: Duration = Duration::from_millis(50);

pub fn run(options: &Options) -> Result<ExitCode, CliError> {
    let listener = bind(options.port)?;
    let port = listener
        .local_addr()
        .map(|address| address.port())
        .unwrap_or(options.port);
    println!("serving {} at http://127.0.0.1:{port}", options.out_dir.display());
    println!("watching {}", options.entry.display());
    println!("press Ctrl-C to stop");

    let mut watched: Vec<(PathBuf, Option<Stamp>)> = Vec::new();
    rebuild(options, &mut watched)?;
    let mut subscribers: Vec<TcpStream> = Vec::new();
    listener
        .set_nonblocking(true)
        .map_err(|error| CliError::Io(format!("cannot listen: {error}")))?;

    loop {
        match listener.accept() {
            Ok((stream, _)) => {
                let _ = stream.set_nonblocking(false);
                if let Err(error) = serve(stream, options, &mut subscribers) {
                    eprintln!("dev: {error}");
                }
            }
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {}
            Err(error) => return Err(CliError::Io(format!("cannot accept: {error}"))),
        }
        if snapshots_change(&watched) {
            match rebuild(options, &mut watched) {
                Ok(()) => notify(&mut subscribers),
                Err(error) => {
                    // The files have moved since the last good build. Re-stamp
                    // what we can see now so a steady-state broken entry does
                    // not spam the poll loop; the next change starts over.
                    for (path, stamp) in watched.iter_mut() {
                        *stamp = Stamp::of(path);
                    }
                    eprintln!("dev: {error}");
                }
            }
        }
        std::thread::sleep(POLL);
    }
}

/// Everything the compile consulted, snapshot against whatever changed since
/// the last rebuild.
pub fn snapshots_change(watched: &[(PathBuf, Option<Stamp>)]) -> bool {
    watched
        .iter()
        .any(|(path, stored)| Stamp::of(path) != *stored)
}

fn bind(port: u16) -> Result<TcpListener, CliError> {
    TcpListener::bind(("127.0.0.1", port))
        .map_err(|error| CliError::Io(format!("cannot listen on port {port}: {error}")))
}

fn rebuild(options: &Options, watched: &mut Vec<(PathBuf, Option<Stamp>)>) -> Result<(), CliError> {
    let built = build::compile_entry(&options.entry, crate::command::RustMode::Build)?;
    let written = write_artifacts(&options.out_dir, &built.compilation.artifacts)?;
    inject_client(&options.out_dir);
    if !options.quiet {
        for artifact in &written {
            println!("wrote {}", artifact.display());
        }
        println!("rebuilt {}", built.label);
    }
    *watched = built
        .dependencies
        .into_iter()
        .map(|path| {
            let stamp = Stamp::of(&path);
            (path, stamp)
        })
        .collect();
    Ok(())
}

/// A file's identity for change detection.
///
/// Length is part of it because a save that lands inside one filesystem
/// timestamp tick is otherwise invisible.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Stamp {
    pub len: u64,
    pub modified: Option<SystemTime>,
}

impl Stamp {
    pub fn of(path: &Path) -> Option<Self> {
        let metadata = std::fs::metadata(path).ok()?;
        Some(Stamp {
            len: metadata.len(),
            modified: metadata.modified().ok(),
        })
    }
}

pub fn decide_rebuild(previous: &Stamp, current: &Stamp) -> bool {
    previous != current
}

/// Adds the reload client to the generated page.
///
/// It is injected here rather than emitted by the compiler so that a production
/// build carries no development code.
pub fn inject_client(out_dir: &Path) -> Option<()> {
    let path = out_dir.join("index.html");
    let html = std::fs::read_to_string(&path).ok()?;
    if html.contains(MARKER) {
        return Some(());
    }
    let client = format!("<script data-platipus-reload>new EventSource(\"{EVENTS_PATH}\").onmessage=()=>location.reload()</script>\n");
    let updated = match html.find("</body>") {
        Some(at) => {
            let (before, rest) = html.split_at(at);
            format!("{before}{client}{rest}")
        }
        None => format!("{html}{client}"),
    };
    std::fs::write(&path, updated).ok()
}

fn notify(subscribers: &mut Vec<TcpStream>) {
    subscribers.retain_mut(|stream| {
        stream
            .write_all(b"data: reload\n\n")
            .and_then(|()| stream.flush())
            .is_ok()
    });
}

fn serve(
    mut stream: TcpStream,
    options: &Options,
    subscribers: &mut Vec<TcpStream>,
) -> Result<(), String> {
    let mut buffer = [0_u8; 2048];
    let read = stream.read(&mut buffer).map_err(|error| error.to_string())?;
    let request = String::from_utf8_lossy(&buffer[..read]);
    let target = request.split_whitespace().nth(1).unwrap_or("/");
    if target == EVENTS_PATH {
        return subscribe(stream, subscribers);
    }
    let path = match resolve(&options.out_dir, target) {
        Some(path) => path,
        None => return respond(&mut stream, 403, "text/plain", b"forbidden"),
    };
    match std::fs::read(&path) {
        Ok(contents) => {
            let body = contents;
            respond(&mut stream, 200, content_type(&path), &body)
        }
        Err(_) => respond(&mut stream, 404, "text/plain", b"not found"),
    }
}

fn subscribe(mut stream: TcpStream, subscribers: &mut Vec<TcpStream>) -> Result<(), String> {
    let head = "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nCache-Control: no-cache\r\nConnection: keep-alive\r\n\r\n";
    stream
        .write_all(head.as_bytes())
        .map_err(|error| error.to_string())?;
    stream.flush().map_err(|error| error.to_string())?;
    subscribers.push(stream);
    Ok(())
}

/// Maps a request path onto a file inside `root`.
///
/// Anything that could leave the directory is refused rather than clamped, so a
/// traversal attempt is an error instead of a quiet different file.
pub fn resolve(root: &Path, requested: &str) -> Option<PathBuf> {
    let trimmed = requested.split(['?', '#']).next().unwrap_or("");
    let relative = trimmed.trim_start_matches('/');
    if relative.is_empty() {
        return Some(root.join("index.html"));
    }
    if relative.starts_with('/') || relative.contains('\0') {
        return None;
    }
    let mut path = root.to_path_buf();
    for segment in relative.split('/') {
        match segment {
            "" | "." => {}
            ".." => return None,
            other => path.push(other),
        }
    }
    if path.starts_with(root) {
        Some(path)
    } else {
        None
    }
}

pub fn content_type(path: &Path) -> &'static str {
    match path
        .extension()
        .and_then(|extension| extension.to_str())
        .unwrap_or("")
    {
        "html" => "text/html; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "js" | "mjs" => "text/javascript; charset=utf-8",
        "json" => "application/json",
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "ico" => "image/x-icon",
        "wasm" => "application/wasm",
        _ => "application/octet-stream",
    }
}

fn respond(stream: &mut TcpStream, status: u16, content_type: &str, body: &[u8]) -> Result<(), String> {
    let head = format!(
        "HTTP/1.1 {status} {}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        reason(status),
        body.len()
    );
    stream
        .write_all(head.as_bytes())
        .and_then(|()| stream.write_all(body))
        .and_then(|()| stream.flush())
        .map_err(|error| error.to_string())
}

fn reason(status: u16) -> &'static str {
    match status {
        200 => "OK",
        403 => "Forbidden",
        404 => "Not Found",
        _ => "Unknown",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_traversal_is_refused_rather_than_clamped() {
        let root = Path::new("dist");
        assert_eq!(resolve(root, "/../secret"), None);
        assert_eq!(resolve(root, "/a/../../secret"), None);
        assert_eq!(resolve(root, "/a/./b"), Some(PathBuf::from("dist/a/b")));
    }

    #[test]
    fn the_root_serves_the_index() {
        assert_eq!(
            resolve(Path::new("dist"), "/"),
            Some(PathBuf::from("dist/index.html"))
        );
    }

    #[test]
    fn a_query_string_is_not_part_of_the_path() {
        assert_eq!(
            resolve(Path::new("dist"), "/app.js?v=2"),
            Some(PathBuf::from("dist/app.js"))
        );
    }

    #[test]
    fn content_types_cover_what_the_codegen_emits() {
        assert!(content_type(Path::new("a.html")).starts_with("text/html"));
        assert!(content_type(Path::new("a.css")).starts_with("text/css"));
        assert!(content_type(Path::new("a.js")).starts_with("text/javascript"));
        assert!(content_type(Path::new("a.mjs")).starts_with("text/javascript"));
        assert!(content_type(Path::new("a.json")).contains("json"));
    }

    #[test]
    fn only_a_changed_file_rebuilds() {
        let before = Stamp {
            len: 10,
            modified: Some(SystemTime::UNIX_EPOCH),
        };
        assert!(!decide_rebuild(&before, &before));
        let longer = Stamp {
            len: 11,
            modified: Some(SystemTime::UNIX_EPOCH),
        };
        assert!(decide_rebuild(&before, &longer));
        let newer = Stamp {
            len: 10,
            modified: Some(SystemTime::UNIX_EPOCH + Duration::from_secs(1)),
        };
        assert!(decide_rebuild(&before, &newer));
    }

    #[test]
    fn a_watched_set_changes_when_any_file_moves() {
        let dir = std::env::temp_dir().join("platipus-dev-stamp");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("scratch directory");
        let one = dir.join("one.plt");
        let two = dir.join("two.plt");
        std::fs::write(&one, "1").expect("write one");
        std::fs::write(&two, "2").expect("write two");
        let watched: Vec<(PathBuf, Option<Stamp>)> = [&one, &two]
            .iter()
            .map(|path| {
                let stamp = Stamp::of(path);
                ((*path).clone(), stamp)
            })
            .collect();
        assert!(!snapshots_change(&watched));
        std::fs::write(&one, "12").expect("rewrite one");
        assert!(snapshots_change(&watched));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
