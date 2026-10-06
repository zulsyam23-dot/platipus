//! End-to-end tests for the `#[rust]` feature (extraction, type checking,
//! diagnostics, and — when a toolchain is available — cargo compilation).

fn workdir(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("plt-rust-test-{}-{}", std::process::id(), name));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn extracts_exports_and_strips_export_markers() {
    let source = r#"#[rust]
#[export]
fn add(a: i64, b: i64) -> i64 {
    a + b
}

app Main {
    Column {
        Text add(10, 20)
    }
}
"#;
    let outcome = platipus_compiler::parser::parse("main.plt", source);
    let extraction = platipus_compiler::rust::extract_rust(&outcome.program)
        .expect("extraction")
        .expect("extraction");
    assert_eq!(extraction.exports.len(), 1);
    assert_eq!(extraction.exports[0].name, "add");
    assert!(!extraction.source.contains("#[export]"));
}

#[test]
fn unsupported_types_produce_a_semantic_error() {
    let source = r#"#[rust]
#[export]
fn bad(x: HashMap<String, i64>) -> i64 {
    0
}

app Main { Column { } }
"#;
    let outcome = platipus_compiler::parser::parse("main.plt", source);
    let bag = platipus_compiler::rust::extract_rust(&outcome.program)
        .err()
        .expect("must fail");
    assert!(
        bag.errors()
            .any(|e| e.message.contains("HashMap<String, i64>"))
    );
}

#[test]
fn cargo_check_accepts_a_valid_block() {
    let dir = workdir("check-ok");
    let source = r#"#[rust]
#[export]
fn add(a: i64, b: i64) -> i64 {
    a + b
}

app Main { Column { } }
"#;
    let outcome = platipus_compiler::parser::parse("main.plt", source);
    let compiled = platipus_compiler::rust::compile_rust(
        &outcome.program,
        &dir,
        platipus_compiler::rust::RustMode::Check,
    )
    .expect("cargo check should succeed");
    assert!(compiled.is_some());
}

#[test]
fn invalid_rust_maps_errors_back_to_the_plt_file() {
    let dir = workdir("invalid");
    let source = r#"#[rust]
#[export]
fn broken(a: i64) -> i64 {
    a + undefined_value
}

app Main { Column { } }
"#;
    let outcome = platipus_compiler::parser::parse("main.plt", source);
    let result = platipus_compiler::rust::compile_rust(
        &outcome.program,
        &dir,
        platipus_compiler::rust::RustMode::Check,
    );
    let bag = result.err().expect("must fail");
    let error = bag.errors().next().unwrap();
    assert_eq!(error.code, "rust-compile");
    assert!(error.span.is_some(), "error span must point at the .plt source");
}

#[test]
fn wasm_build_embeds_in_the_web_bundle() {
    let dir = workdir("wasm");
    let source = r#"#[rust]
#[export]
fn add(a: i64, b: i64) -> i64 {
    a + b
}

app Main {
    Column {
        Text add(10, 20)
    }
}
"#;
    let outcome = platipus_compiler::parser::parse("main.plt", source);
    let compiled = platipus_compiler::rust::compile_rust(
        &outcome.program,
        &dir,
        platipus_compiler::rust::RustMode::Build,
    )
    .expect("cargo build should succeed");
    let compiled = compiled.expect("a rust compilation");
    assert!(
        compiled.wasm_path.as_ref().is_some_and(|p| p.exists()),
        "a wasm artifact must be produced"
    );
}
