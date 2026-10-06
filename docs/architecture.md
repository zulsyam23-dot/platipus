# Platipus Architecture

This document is the authoritative description of how the Platipus workspace
is organised. `docs/prd.md` describes intent, `docs/roadmap.md` records what is
and is not implemented per phase, and `docs/struktur.md` is the original
design brief; where those differ from this file, this file wins.

## Core principle

> A library knows the contract, never the consumer.

Every library is usable by the Platipus compiler today, and by an IDE, a test
runner, a different compiler, or a different backend tomorrow, without
modification. The compiler is an orchestrator; backends are implementations;
domain libraries own the model.

## Crate map

| Crate | Owns | Does not own |
|---|---|---|
| `platipus-diagnostics` | `Span`, `Error`, `Warning`, `DiagnosticBag`, `SourceFile` | Language, IR, backends |
| `platipus-language` | Lexer, parser, AST, event vocabulary (`EventCategory`, `category_of`), element registry (`ElementRegistry`, builtins), `#[rust]` extraction (`rust_extract`, `rust_types`) | Semantic rules, IR, platform |
| `platipus-semantic` | `SemanticChecker`, scopes, symbols, type registry, validation of events/styles/imports | Parsing, codegen |
| `platipus-ir` | `IrModule` and friends, lowering, verifier, backend contract (`Target`, `Artifact`, `RustBridge`) | Platform details, Rust/WASM |
| `platipus-reactive` | `StateStore`, `Derived`/`DerivedSet`, `EventBus`, `StateKind`, `Value`, `Scope` | Platform rendering |
| `platipus-storage` | `Storage` trait, `MemoryStorage` | Platform backends (web localStorage, fs, native) |
| `platipus-testing` | Embedded test model (`Test`, `Step`), `TEST_ACTIONS`, `is_known_action` | A runner implementation |
| `platipus-web` | The whole web backend in `codegen/`: `Web`/`WebTarget`, HTML, CSS, JS runtime, DOM shim, embedded test runner, bundle manifest | IR internals, semantic rules |
| `platipus-compiler` | `pipeline` (source → AST → semantic → IR → verify → target), `loader` (import graph), `rust` stage (Cargo build of `#[rust]` blocks) | Domain logic |
| `platipus-runtime` | Facade re-exporting `platipus-reactive` | New code |
| `platipus-standard` | Optional stdlib helpers on top of `Value` (collection, string, text, math, list, map, parse) | Core concepts |
| `platipus-cli` | `platipus` / `plt` commands | Compilation logic (delegates to compiler) |
| `platipus-p2lt` | Package manager | Compiler phases |

## Dependency graph

```text
                    platipus-diagnostics
                    /    |      |      \
                   /     |      |       \
        platipus-language     |      platipus-testing
            |        \        |        /     |
            |         \       |       /      |
        platipus-semantic  platipus-ir ──────┘
                          │        ^  \
                          │        │   \__ platipus-web
                          │        │        (backend impl)
              platipus-reactive  platipus-storage
                  ^                        ^
                  |                        |
            platipus-runtime (facade)  platipus-standard

        platipus-compiler
          = loader + pipeline + rust stage
          depends on: language, semantic, ir, web?, diagnostics
          (web is wired in by the CLI, not required by the compiler crate)

        platipus-cli, platipus-p2lt → platipus-compiler
```

Rules:

- Edges always point "downward" toward contracts. No cycles.
- `platipus-ir` never imports `platipus-web` or `platipus-compiler`.
- Domain crates never import `platipus-compiler`, `platipus-cli`, or a backend.
- `RustBridge` carries `#[rust]` results *into* the backend; the IR never
  mentions Rust or WASM.

## Public API boundaries

- `platipus_compiler::pipeline::build_entry_rust(...)` — full pipeline from
  entry file to `Compilation` (program, IR module, artifacts, warnings).
- `platipus_compiler::pipeline::analyse` — front half (parse + semantic).
- `platipus_ir::lower_program` / `verify` — IR construction and checking.
- `platipus_ir::backend::Target` — the only way to turn an `IrModule` into
  artifacts. New backends implement `Target`; they never touch `pipeline`
  internals.
- `platipus_ir::RustBridge` — exports + compiled artifact handed to the
  backend alongside the module.
- `platipus_reactive::{StateStore, DerivedSet, EventBus}` — host runtime.
- `platipus_storage::Storage` — persistence contract.
- `platipus_testing::{Test, Step, TEST_ACTIONS}` — embedded test contract.

## `#[rust]` audit

`#[rust]` blocks are a real, supported Platipus feature: an escape hatch for
trusted Rust code. The pipeline, end to end:

```text
lexer (rust_block.rs)        captures raw Rust verbatim between braces
      ↓
parser                       produces ast::RustBlock (attributes, source, span)
      ↓
semantic checker             declare_rust_exports: validates via
                             language::rust_extract::extract_rust
      ↓
ir                           nothing — IR stays Rust-free
      ↓
compiler::rust stage         extract_rust → generates a Cargo crate in
                             target/rust, builds wasm, base64-encodes it
      ↓
RustBridge { exports, wasm_b64 }   passed to Target::generate
      ↓
web backend                  embeds base64 wasm, generates __pltRust
                             wrappers converting JS types ↔ wasm ABI
```

Guarantees:

- A file with no `#[rust]` blocks produces no Rust stage and no wasm.
- Export signature validation is in `platipus-language::rust_types`.
- Errors in the embedded Rust map back to spans in the `.plt` file.
- `#[rust]` semantics are untouched by the modularisation; tests in
  `tests/rust/` guard behaviour.

## Reactive semantics

The reactive model is unchanged from before the modularisation:

```text
state count = 0
derived doubled = count * 2
on click { count += 1 }
```

- `state` is a writable signal owned by `StateStore` (`platipus-reactive`).
- `derived` recomputes when its dependencies change (`DerivedSet`).
- Handlers mutate state; a single dispatch is one transactional render in the
  web backend, and a synchronous fan-out in the host runtime.
- `shared` / `global` / `persistent` state are scope variants of the same
  store (`StateKind`), with `persistent` additionally routed through the
  platform storage (`Storage` contract; the web backend currently emits
  localStorage/sessionStorage/IndexedDB shims).

The IR mirrors `StateKind` (`IrStateKind`) and the AST mirrors it too; the
three definitions are kept in lockstep by tests because compile-time layers
must not depend on the runtime crate.

## Testing

- Every crate has unit tests (`cargo test -p <crate>`).
- Integration suites live in `tests/` and run through the compiler facade:
  `lexer`, `parser`, `semantic`, `ir`, `codegen`, `loader`, `rust`, `cli`,
  `fixtures`, `web`.
- `fixtures/valid` compiles clean, `fixtures/invalid` must fail.
- Embedded `test {}` blocks compile to a runner (`tests.mjs`) executed by
  `plt test` under Node with a DOM shim (`dom.mjs`).
- The inventories of event names and test actions are asserted by cross-checks
  (e.g. the generated runner covers exactly `TEST_ACTIONS`).

## Migration notes (v0.1 → modular)

1. `platipus-diagnostics` extracted (was `compiler::diagnostics`).
2. `platipus-language` extracted (ast, lexer, parser, events, element
   registry, `rust_extract`, `rust_types`).
3. `platipus-semantic` extracted.
4. `platipus-ir` extracted; backend contract (`Target`) moved here;
   `IrRustBridge` replaced by `RustBridge` and removed from `IrModule`.
5. `platipus-web` extracted (was `compiler::codegen`); `javascript.rs` split
   into `javascript/{mod,component,api,bootstrap,rust}.rs`.
6. `platipus-reactive` extracted from the old runtime crate; runtime is now a
   facade.
7. `platipus-testing` extracted; `IrTest`/`IrTestStep` are aliases.
8. `platipus-storage` introduced as the persistence contract.
9. Compiler reduced to orchestrator: `loader`, `pipeline`, `rust` stage.

No syntax or semantics changed during any of these steps; every stage is
guarded by the pre-existing regression suite.
