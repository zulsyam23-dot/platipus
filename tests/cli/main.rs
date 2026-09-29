use std::fs;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

fn binary() -> &'static str {
    env!("CARGO_BIN_EXE_platipus")
}

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("platipus-cli-{name}"));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("scratch directory");
    dir
}

fn write(dir: &Path, name: &str, source: &str) -> PathBuf {
    let path = dir.join(name);
    fs::write(&path, source).expect("write fixture");
    path
}

fn run(args: &[&str]) -> (bool, String, String) {
    run_in(None, args)
}

fn run_in(cwd: Option<&Path>, args: &[&str]) -> (bool, String, String) {
    let mut command = Command::new(binary());
    command.args(args);
    if let Some(cwd) = cwd {
        command.current_dir(cwd);
    }
    let output = command.output().expect("run the cli");
    (
        output.status.success(),
        String::from_utf8_lossy(&output.stdout).to_string(),
        String::from_utf8_lossy(&output.stderr).to_string(),
    )
}

const COUNTER: &str = r##"
component Counter {
    state count = 0
    derived doubled = count * 2

    fn increment(by: Int) {
        count += by
    }

    Column {
        Text count
        Button "+1" {
            on click {
                increment(1)
            }
        }
    }
}

app CounterApp {
    Counter { }
}
"##;

#[test]
fn build_writes_every_artifact() {
    let dir = scratch("build");
    let entry = write(&dir, "app.plt", COUNTER);
    let out = dir.join("dist");
    let (ok, stdout, stderr) = run(&[
        "build",
        entry.to_str().unwrap(),
        "-o",
        out.to_str().unwrap(),
    ]);
    assert!(ok, "stderr: {stderr}");
    for name in ["index.html", "app.css", "app.js", "program.json"] {
        assert!(out.join(name).exists(), "missing {name}");
    }
    assert!(stdout.contains("built"), "{stdout}");
}

#[test]
fn the_default_output_directory_is_dist() {
    let dir = scratch("default-out");
    write(&dir, "app.plt", COUNTER);
    let (ok, _, stderr) = run_in(Some(&dir), &["build", "app.plt"]);
    assert!(ok, "stderr: {stderr}");
    assert!(dir.join("dist").join("index.html").exists());
}

#[test]
fn check_never_writes_files() {
    let dir = scratch("check");
    let entry = write(&dir, "app.plt", COUNTER);
    let (ok, stdout, stderr) = run(&["check", entry.to_str().unwrap()]);
    assert!(ok, "stderr: {stderr}");
    assert!(stdout.contains("checked"), "{stdout}");
    assert!(!dir.join("dist").exists());
}

#[test]
fn the_check_flag_skips_writing() {
    let dir = scratch("check-flag");
    let entry = write(&dir, "app.plt", COUNTER);
    let (ok, _, stderr) = run(&["build", entry.to_str().unwrap(), "--check"]);
    assert!(ok, "stderr: {stderr}");
    assert!(!dir.join("dist").exists());
}

#[test]
fn quiet_mode_prints_nothing_on_success() {
    let dir = scratch("quiet");
    let entry = write(&dir, "app.plt", COUNTER);
    // The output goes to the scratch directory rather than the default
    // `dist` beside the source, so a test run leaves nothing in the tree.
    let out = dir.join("out");
    let (ok, stdout, stderr) = run(&[
        "build",
        entry.to_str().unwrap(),
        "-q",
        "-o",
        out.to_str().unwrap(),
    ]);
    assert!(ok, "stderr: {stderr}");
    assert_eq!(stdout, "");
    assert!(out.exists(), "a quiet build still writes its output");
}

#[test]
fn semantic_errors_fail_the_build() {
    let dir = scratch("broken");
    let entry = write(&dir, "app.plt", "app Main { state count = 0 }");
    let (ok, _, stderr) = run(&["build", entry.to_str().unwrap()]);
    assert!(!ok);
    assert!(stderr.contains("empty-app"), "{stderr}");
    assert!(!dir.join("dist").exists());
}

#[test]
fn parse_errors_are_reported_with_a_caret() {
    let dir = scratch("parse-error");
    let entry = write(&dir, "app.plt", "app Main { Column { }");
    let (ok, _, stderr) = run(&["check", entry.to_str().unwrap()]);
    assert!(!ok);
    assert!(stderr.contains("^"), "{stderr}");
}

#[test]
fn unknown_builtins_suggest_a_real_element() {
    let dir = scratch("suggest");
    let entry = write(&dir, "app.plt", "app Main { Buton \"x\" { } }");
    let (ok, _, stderr) = run(&["check", entry.to_str().unwrap()]);
    assert!(!ok);
    assert!(stderr.contains("did you mean `Button`"), "{stderr}");
}

#[test]
fn a_missing_entry_file_is_reported() {
    let (ok, _, stderr) = run(&["build", "definitely-not-here.plt"]);
    assert!(!ok);
    assert!(stderr.contains("cannot find"), "{stderr}");
}

#[test]
fn unknown_commands_fail_with_a_usage_code() {
    let (ok, _, stderr) = run(&["frobnicate"]);
    assert!(!ok);
    assert!(stderr.contains("unknown command"), "{stderr}");
}

#[test]
fn help_and_version_succeed() {
    let (ok, stdout, _) = run(&["--help"]);
    assert!(ok);
    assert!(stdout.contains("USAGE"), "{stdout}");

    let (ok, stdout, _) = run(&["--version"]);
    assert!(ok);
    assert!(stdout.contains(env!("CARGO_PKG_VERSION")), "{stdout}");
}

#[test]
fn no_arguments_prints_help() {
    let (ok, stdout, _) = run(&[]);
    assert!(ok);
    assert!(stdout.contains("USAGE"), "{stdout}");
}

#[test]
fn the_generated_javascript_contains_the_component() {
    let dir = scratch("inspect-js");
    let entry = write(&dir, "app.plt", COUNTER);
    let out = dir.join("dist");
    let (ok, _, stderr) = run(&[
        "build",
        entry.to_str().unwrap(),
        "-o",
        out.to_str().unwrap(),
        "-q",
    ]);
    assert!(ok, "stderr: {stderr}");
    let script = fs::read_to_string(out.join("app.js")).expect("app.js");
    assert!(script.contains("function Counter(inputs)"), "{script}");
    assert!(script.contains("plt.computed"), "{script}");
    assert!(script.contains("plt.el(\"button\""), "{script}");
}

#[test]
fn the_generated_css_carries_layout_and_user_styles() {
    let dir = scratch("inspect-css");
    let entry = write(
        &dir,
        "app.plt",
        "app Main { Column { } }\nstyle card { backgroundColor: \"#111\" }",
    );
    let out = dir.join("dist");
    let (ok, _, stderr) = run(&[
        "build",
        entry.to_str().unwrap(),
        "-o",
        out.to_str().unwrap(),
        "-q",
    ]);
    assert!(ok, "stderr: {stderr}");
    let css = fs::read_to_string(out.join("app.css")).expect("app.css");
    // The base rule carries the leading dot; matching without it also matches a
    // mention in the middle of another declaration.
    assert!(css.contains(".plt-column {"), "{css}");
    assert!(css.contains("background-color: #111;"), "{css}");
}

#[test]
fn the_build_loads_imported_modules_into_the_artifacts() {
    let dir = scratch("build-imports");
    write(
        &dir,
        "leaf.plt",
        "component Banner {\n    Text \"imported\"\n}\n",
    );
    let entry = write(
        &dir,
        "app.plt",
        "import Banner from \"./leaf.plt\"\n\napp Main {\n    Banner { }\n}\n",
    );
    let out = dir.join("dist");
    let (ok, stdout, stderr) = run(&[
        "build",
        entry.to_str().unwrap(),
        "-o",
        out.to_str().unwrap(),
    ]);
    assert!(ok, "stderr: {stderr}");
    assert!(stdout.contains("built"), "{stdout}");
    let script = fs::read_to_string(out.join("app.js")).expect("app.js");
    assert!(script.contains("function Banner("), "{script}");

    let (ok, stdout, stderr) = run(&["check", entry.to_str().unwrap()]);
    assert!(ok, "stderr: {stderr}");
    assert!(stdout.contains("checked"), "{stdout}");
}

const TESTED: &str = r##"
app Counter {
    state count = 0
    derived doubled = count * 2

    Column {
        Text count
        Button "+" {
            on click {
                count = count + 1
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
"##;

#[test]
fn build_writes_the_dom_shim_and_the_test_runner() {
    let dir = scratch("build-tests");
    let entry = write(&dir, "app.plt", TESTED);
    let out = dir.join("dist");
    let (ok, _, stderr) = run(&[
        "build",
        entry.to_str().unwrap(),
        "-o",
        out.to_str().unwrap(),
        "-q",
    ]);
    assert!(ok, "stderr: {stderr}");
    for name in ["dom.mjs", "tests.mjs"] {
        assert!(out.join(name).exists(), "missing {name}");
    }
    let runner = fs::read_to_string(out.join("tests.mjs")).expect("tests.mjs");
    assert!(runner.contains("installDom"), "{runner}");
    assert!(runner.contains("clickingAddsOne"), "{runner}");
}

#[test]
fn the_manifest_lists_the_programs_tests() {
    let dir = scratch("manifest-tests");
    let entry = write(&dir, "app.plt", TESTED);
    let out = dir.join("dist");
    let (ok, _, stderr) = run(&[
        "build",
        entry.to_str().unwrap(),
        "-o",
        out.to_str().unwrap(),
        "-q",
    ]);
    assert!(ok, "stderr: {stderr}");
    let manifest = fs::read_to_string(out.join("program.json")).expect("program.json");
    assert!(manifest.contains("\"tests\""), "{manifest}");
    assert!(manifest.contains("clickingAddsOne"), "{manifest}");
    assert!(manifest.contains("\"action\": \"click\""), "{manifest}");
}

#[test]
fn an_unknown_test_action_is_rejected_before_anything_runs() {
    let dir = scratch("bad-action");
    let entry = write(
        &dir,
        "app.plt",
        "app Main { Button \"x\" { } }\n\ntest hovering {\n    hover \"x\"\n}\n",
    );
    let (ok, _, stderr) = run(&["check", entry.to_str().unwrap()]);
    assert!(!ok);
    assert!(stderr.contains("unknown-test-action"), "{stderr}");
}

/// Node is needed to run the generated runner, so these are skipped without it
/// rather than failing a machine that cannot run them.
fn node_available() -> bool {
    Command::new("node")
        .arg("--version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

#[test]
fn the_test_command_runs_the_programs_tests() {
    if !node_available() {
        eprintln!("skipped: node is not on the PATH");
        return;
    }
    let dir = scratch("test-cmd");
    let entry = write(&dir, "app.plt", TESTED);
    let out = dir.join("dist");
    let (ok, stdout, stderr) = run(&[
        "test",
        entry.to_str().unwrap(),
        "-o",
        out.to_str().unwrap(),
    ]);
    assert!(ok, "stderr: {stderr}");
    assert!(stdout.contains("ok   startsAtZero"), "{stdout}");
    assert!(stdout.contains("ok   clickingAddsOne"), "{stdout}");
    assert!(stdout.contains("2 passed, 0 failed"), "{stdout}");
}

#[test]
fn a_failing_expectation_fails_the_command() {
    if !node_available() {
        eprintln!("skipped: node is not on the PATH");
        return;
    }
    let dir = scratch("test-fail");
    let entry = write(
        &dir,
        "app.plt",
        r##"
app Counter {
    state count = 0
    Button "+" {
        on click {
            count = count + 1
        }
    }
}

test wrong {
    click "+"
    expect count == 99
}
"##,
    );
    let (ok, stdout, stderr) = run(&["test", entry.to_str().unwrap(), "-o", dir.to_str().unwrap()]);
    assert!(!ok, "stdout: {stdout} stderr: {stderr}");
    assert!(stdout.contains("FAIL wrong"), "{stdout}");
    assert!(stdout.contains("expected count == 99 to hold"), "{stdout}");
}

#[test]
fn each_test_starts_from_a_fresh_mount() {
    if !node_available() {
        eprintln!("skipped: node is not on the PATH");
        return;
    }
    // The second test clicks once and expects one, which only holds if the first
    // test left nothing behind.
    let dir = scratch("test-isolated");
    let entry = write(&dir, "app.plt", TESTED);
    let (ok, stdout, stderr) = run(&["test", entry.to_str().unwrap(), "-o", dir.to_str().unwrap()]);
    assert!(ok, "stdout: {stdout} stderr: {stderr}");
    assert!(!stdout.contains("FAIL"), "{stdout}");
}

#[test]
fn help_lists_the_test_command() {
    let (ok, stdout, _) = run(&["--help"]);
    assert!(ok);
    assert!(stdout.contains("test <entry.plt>"), "{stdout}");
}

#[test]
fn new_scaffolds_a_project_that_passes_its_own_tests() {
    let dir = scratch("new");
    let (ok, stdout, stderr) = run_in(Some(&dir), &["new", "my-app"]);
    assert!(ok, "stderr: {stderr}");
    let project = dir.join("my-app");
    for name in ["app.plt", "README.md", ".gitignore"] {
        assert!(project.join(name).exists(), "missing {name}");
    }
    // The project name doubles as the component name, so `my-app` has to come
    // back out as `MyApp`.
    let source = fs::read_to_string(project.join("app.plt")).expect("app.plt");
    assert!(source.contains("app MyApp {"), "{source}");
    assert!(stdout.contains("my-app"), "{stdout}");

    // The scaffolded README tells the reader `plt test app.plt` passes as
    // written, so that claim is checked rather than assumed.
    if !node_available() {
        eprintln!("skipped: node is not on the PATH");
        return;
    }
    let (ok, stdout, stderr) = run_in(Some(&project), &["test", "app.plt"]);
    assert!(ok, "stdout: {stdout} stderr: {stderr}");
    assert!(stdout.contains("2 passed, 0 failed"), "{stdout}");
}

#[test]
fn the_scaffolded_project_is_already_formatted() {
    let dir = scratch("new-format");
    let (ok, _, stderr) = run_in(Some(&dir), &["new", "my-app"]);
    assert!(ok, "stderr: {stderr}");
    let entry = dir.join("my-app").join("app.plt");
    let scaffolded = fs::read_to_string(&entry).expect("app.plt");

    // A reader who runs `format` on a fresh project should not get a diff, so
    // the template has to be written the way the formatter would write it.
    let (ok, _, stderr) = run_in(Some(&dir), &["format", entry.to_str().unwrap()]);
    assert!(ok, "stderr: {stderr}");
    assert_eq!(fs::read_to_string(&entry).expect("app.plt"), scaffolded);
}

#[test]
fn new_refuses_to_overwrite_and_rejects_a_name_that_escapes() {
    let dir = scratch("new-guard");
    let (ok, _, stderr) = run_in(Some(&dir), &["new", "my-app"]);
    assert!(ok, "stderr: {stderr}");

    let (ok, _, stderr) = run_in(Some(&dir), &["new", "my-app"]);
    assert!(!ok);
    assert!(stderr.contains("already exists"), "{stderr}");

    // A name becomes a directory, so anything that is not a plain name is
    // refused before a single file is written.
    let (ok, _, stderr) = run_in(Some(&dir), &["new", "../escape"]);
    assert!(!ok);
    assert!(stderr.contains("use letters, digits"), "{stderr}");
    assert!(!dir.parent().unwrap().join("escape").exists());
}

const MESSY: &str = "// the entry point
app Main{state count=0
// a note inside
Column{Text count
Button \"+\"{on click{count+=1}}}}
";

#[test]
fn format_rewrites_in_place_and_keeps_comments() {
    let dir = scratch("format");
    let entry = write(&dir, "app.plt", MESSY);
    let path = entry.to_str().unwrap();
    let (ok, stdout, stderr) = run(&["format", path]);
    assert!(ok, "stderr: {stderr}");
    assert!(stdout.contains("formatted"), "{stdout}");

    // Both comments survive and only the whitespace between tokens changed, so
    // the expected text can be written out in full.
    assert_eq!(
        fs::read_to_string(&entry).expect("app.plt"),
        "\
// the entry point
app Main {
    state count = 0
    // a note inside
    Column {
        Text count
        Button \"+\" {
            on click {
                count += 1
            }
        }
    }
}
"
    );
}

#[test]
fn formatting_twice_changes_nothing() {
    let dir = scratch("format-idempotent");
    let entry = write(&dir, "app.plt", MESSY);
    let path = entry.to_str().unwrap();
    assert!(run(&["format", path]).0);
    let once = fs::read_to_string(&entry).expect("app.plt");

    let (ok, stdout, stderr) = run(&["format", path]);
    assert!(ok, "stderr: {stderr}");
    assert!(stdout.contains("already formatted"), "{stdout}");
    assert_eq!(fs::read_to_string(&entry).expect("app.plt"), once);
}

#[test]
fn format_leaves_a_file_it_cannot_lex_alone() {
    let dir = scratch("format-broken");
    let broken = "app Main {\n    Text \"unterminated\n}\n";
    let entry = write(&dir, "app.plt", broken);
    let (ok, _, stderr) = run(&["format", entry.to_str().unwrap()]);
    assert!(!ok);
    assert!(stderr.contains("^"), "{stderr}");
    assert_eq!(fs::read_to_string(&entry).expect("app.plt"), broken);
}

/// A `dev` process, killed when the test ends.
struct DevServer {
    child: Child,
    port: u16,
    log: PathBuf,
}

impl Drop for DevServer {
    fn drop(&mut self) {
        // `dev` loops forever, so a panic halfway through this test would
        // otherwise leave a listener and a rebuild loop behind.
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

const WATCHED: &str = "app Main {\n    state count = 0\n    Column { }\n}\n";
/// One byte longer than `WATCHED`, so a change is visible even if the filesystem
/// reports the same modification time.
const WATCHED_EDITED: &str = "app Main {\n    state count = 10\n    Column { }\n}\n";

/// `dev` notices a change within one poll interval, so anything that takes
/// seconds is a failure rather than a slow machine. Kept short so a regression
/// fails the suite quickly instead of stalling it.
const PATIENCE: Duration = Duration::from_secs(15);

impl DevServer {
    /// Starts `dev` on a port the operating system picks.
    ///
    /// `-p 0` binds an ephemeral port and `dev` reports the one it actually got,
    /// so the test never races another process for a fixed port. Output goes to
    /// a file rather than a pipe, because a pipe nobody drains would eventually
    /// block the child mid-rebuild.
    fn start(dir: &Path, entry: &Path, out: &Path) -> Self {
        let log = dir.join("dev.log");
        let file = fs::File::create(&log).expect("dev log");
        let errors = file.try_clone().expect("clone dev log");
        let child = Command::new(binary())
            .args([
                "dev",
                entry.to_str().unwrap(),
                "-o",
                out.to_str().unwrap(),
                "-p",
                "0",
            ])
            .stdout(Stdio::from(file))
            .stderr(Stdio::from(errors))
            .spawn()
            .expect("spawn dev");
        let port = wait_for_port(&log);
        DevServer { child, port, log }
    }

    fn log(&self) -> String {
        fs::read_to_string(&self.log).unwrap_or_default()
    }
}

fn wait_for_port(log: &Path) -> u16 {
    const PREFIX: &str = "http://127.0.0.1:";
    let deadline = Instant::now() + Duration::from_secs(60);
    while Instant::now() < deadline {
        let text = fs::read_to_string(log).unwrap_or_default();
        for line in text.lines() {
            if let Some(at) = line.find(PREFIX) {
                let digits: String = line[at + PREFIX.len()..]
                    .chars()
                    .take_while(char::is_ascii_digit)
                    .collect();
                if let Ok(port) = digits.parse() {
                    return port;
                }
            }
        }
        std::thread::sleep(Duration::from_millis(25));
    }
    panic!(
        "dev never reported a port; log:\n{}",
        fs::read_to_string(log).unwrap_or_default()
    );
}

/// Sends one request and reads the whole response, so the status line and the
/// body are both available to assert on.
fn request(port: u16, target: &str) -> (u16, String) {
    let mut stream = TcpStream::connect(("127.0.0.1", port)).expect("connect to dev");
    stream
        .set_read_timeout(Some(PATIENCE))
        .expect("read timeout");
    stream
        .write_all(
            format!("GET {target} HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n")
                .as_bytes(),
        )
        .expect("write request");
    stream.flush().expect("flush request");
    let mut raw = Vec::new();
    stream.read_to_end(&mut raw).expect("read response");
    let text = String::from_utf8_lossy(&raw).to_string();
    let status = text
        .split_whitespace()
        .nth(1)
        .and_then(|code| code.parse().ok())
        .unwrap_or(0);
    (status, text)
}

/// Reads until the accumulated bytes end with the given marker.
fn read_until(stream: &mut TcpStream, marker: &str) -> String {
    let mut seen = Vec::new();
    let mut byte = [0_u8; 1];
    loop {
        match stream.read(&mut byte) {
            Ok(0) => panic!(
                "dev closed the connection before sending {marker:?}; got {:?}",
                String::from_utf8_lossy(&seen)
            ),
            Ok(_) => {
                seen.push(byte[0]);
                if seen.ends_with(marker.as_bytes()) {
                    return String::from_utf8_lossy(&seen).to_string();
                }
            }
            Err(error) => panic!(
                "reading {marker:?} failed: {error}; got {:?}",
                String::from_utf8_lossy(&seen)
            ),
        }
    }
}

#[test]
fn dev_serves_the_build_and_refuses_to_leave_the_output_directory() {
    let dir = scratch("dev-serve");
    let entry = write(&dir, "app.plt", WATCHED);
    let out = dir.join("dist");
    let server = DevServer::start(&dir, &entry, &out);

    let (status, body) = request(server.port, "/");
    assert_eq!(status, 200, "log:\n{}", server.log());
    assert!(body.contains("<html"), "{body}");

    let (status, body) = request(server.port, "/app.js");
    assert_eq!(status, 200, "log:\n{}", server.log());
    assert!(body.contains("function Main"), "{body}");

    // A query string is not part of the path.
    let (status, _) = request(server.port, "/app.css?v=2");
    assert_eq!(status, 200, "log:\n{}", server.log());

    // Anything that could climb out of the output directory is refused rather
    // than clamped, so a traversal gets a 403 and not a different file.
    let (status, _) = request(server.port, "/../app.plt");
    assert_eq!(status, 403, "log:\n{}", server.log());

    let (status, _) = request(server.port, "/nothing-here");
    assert_eq!(status, 404, "log:\n{}", server.log());
}

#[test]
fn dev_signals_a_reload_after_the_entry_file_changes() {
    let dir = scratch("dev-reload");
    let entry = write(&dir, "app.plt", WATCHED);
    let out = dir.join("dist");
    let server = DevServer::start(&dir, &entry, &out);

    // The reload client is injected into the served page, not by the compiler,
    // so a production build carries no development code.
    let (status, body) = request(server.port, "/");
    assert_eq!(status, 200, "log:\n{}", server.log());
    assert!(body.contains("data-platipus-reload"), "{body}");

    // Subscribing is what proves `dev` has registered this connection, so the
    // edit below cannot be missed by racing the accept loop.
    let mut events = TcpStream::connect(("127.0.0.1", server.port)).expect("connect for events");
    events
        .set_read_timeout(Some(PATIENCE))
        .expect("event timeout");
    events
        .write_all(b"GET /__plt/events HTTP/1.1\r\nHost: 127.0.0.1\r\n\r\n")
        .expect("subscribe");
    events.flush().expect("flush subscribe");
    let head = read_until(&mut events, "\r\n\r\n");
    assert!(head.contains("text/event-stream"), "{head}");

    fs::write(&entry, WATCHED_EDITED).expect("edit the entry file");
    assert!(read_until(&mut events, "\n\n").contains("data: reload"));

    // The signal is only worth anything if the rebuild actually happened, so the
    // edited value is checked in the artifact the browser would be served.
    let script = fs::read_to_string(out.join("app.js")).expect("rebuilt app.js");
    assert!(script.contains("10"), "the edit did not reach the rebuild");
}

#[test]
fn dev_rebuilds_when_an_imported_file_changes() {
    let dir = scratch("dev-import");
    let leaf = write(
        &dir,
        "leaf.plt",
        "component Banner {\n    Text \"leaf-one\"\n}\n",
    );
    let entry = write(
        &dir,
        "app.plt",
        "import Banner from \"./leaf.plt\"\n\napp Main {\n    Banner { }\n}\n",
    );
    let out = dir.join("dist");
    let server = DevServer::start(&dir, &entry, &out);

    // The import reaches the build from the start, so a change in the leaf is
    // the event under test rather than the initial write.
    let (status, _) = request(server.port, "/");
    assert_eq!(status, 200, "log:\n{}", server.log());
    let script = fs::read_to_string(out.join("app.js")).expect("app.js");
    assert!(script.contains("leaf-one"), "{script}");

    let mut events = TcpStream::connect(("127.0.0.1", server.port)).expect("connect for events");
    events
        .set_read_timeout(Some(PATIENCE))
        .expect("event timeout");
    events
        .write_all(b"GET /__plt/events HTTP/1.1\r\nHost: 127.0.0.1\r\n\r\n")
        .expect("subscribe");
    events.flush().expect("flush subscribe");
    let head = read_until(&mut events, "\r\n\r\n");
    assert!(head.contains("text/event-stream"), "{head}");

    fs::write(&leaf, "component Banner {\n    Text \"leaf-two\"\n}\n").expect("edit the leaf");
    assert!(read_until(&mut events, "\n\n").contains("data: reload"));

    let script = fs::read_to_string(out.join("app.js")).expect("rebuilt app.js");
    assert!(script.contains("leaf-two"), "the leaf edit did not reach the rebuild");
}


