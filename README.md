# Platipus

**A UI Application Programming Language for the modern web.**

Platipus is not HTML with different syntax, not a CSS framework, and not a JavaScript
framework. It is a language with its own programming model, where `element`, `state`,
`event`, `style`, and `component` are *language constructs* rather than three separate
technologies stitched together.

```text
Application
    ↓
Component
    ↓
Element
    ├── Property
    ├── Style
    ├── State Binding
    └── Event
    ↓
Reactive Runtime
    ↓
Web
```

## Example

```text
app CounterApp {

    state count = 0

    Column {
        Text count

        Button "+" {
            on click {
                count += 1
            }
        }
    }
}
```

One construct per concept. No `<div>`, no `@click`, no `setState()`, no `render()`.

## Workspace layout

| Crate | Responsibility |
| ------------------------ | --------------------------------------------------------------------- |
| `platipus-diagnostics`   | Spans, errors, warnings, diagnostic bag — the shared reporting contract |
| `platipus-language`      | Lexer, parser, AST, event vocabulary, element registry, `#[rust]` extraction |
| `platipus-semantic`      | Semantic analysis: scopes, types, validation |
| `platipus-ir`            | Lowered, verifiable module IR + the backend contract (`Target`, `RustBridge`) |
| `platipus-reactive`      | Reactive state, derived values, event bus (host side of the runtime contract) |
| `platipus-storage`       | Storage contract (`Storage`, `MemoryStorage`) |
| `platipus-testing`       | Embedded test model (`Test`, `Step`, `TEST_ACTIONS`) |
| `platipus-web`           | Web backend: HTML/CSS/JavaScript/DOM generation |
| `platipus-compiler`      | Orchestrator: loader, pipeline, `#[rust]` compilation stage |
| `platipus-runtime`       | Facade re-exporting `platipus-reactive` |
| `platipus-standard`      | Optional standard-library helpers |
| `platipus-cli`           | `platipus` / `plt` — new, dev, build, run, check, format, test |
| `platipus-p2lt`          | Package manager |

The dependency graph is strictly one-way and consumer-blind:

```text
diagnostics ───┬── language ─── semantic ───┐
               │       │                     │
               │       └────── ir ────────── web (backend)
               ├── reactive                   ▲
               ├── storage                    │
               └── testing ───────────────────┘
                                                 │
                                  compiler (orchestrator) ── cli / p2lt
```

Domain libraries never import the compiler, a backend, or an application.
See [`docs/architecture.md`](docs/architecture.md) for the full contract.

The runtime never depends on the standard library for core UI concepts, and the
compiler never depends on the UI runtime in order to parse.

`platipus-reactive` is the Rust side of the reactive contract: a state store, a
derived-value set, and an event bus. It is not a second renderer — the code a
browser actually executes is emitted into the generated JavaScript, and the two
implement the same contract. See [`docs/roadmap.md`](docs/roadmap.md) §4.2 for
what is and is not implemented in each crate.

## Build output

```text
dist/
├── index.html     the page shell
├── app.css        layout primitives plus your style blocks
├── app.js         the compiled program and its runtime
├── program.json   manifest: components, apis, routes, tests
├── dom.mjs        Node-side DOM shim, for `plt test`
└── tests.mjs      the runner `plt test` executes
```

There is no WASM target and no bundler. `dom.mjs` and `tests.mjs` exist so
`plt test` can drive a compiled program under Node; a browser is served
`index.html`, `app.css`, `app.js`, and `program.json`.

## Principles

1. One concept, one primary syntax.
2. Reactive by default — no manual render cycle.
3. Primitives instead of custom elements for visual variation.
4. Components for composition, not for styling.
5. The DOM is an implementation target, not the programming model.
6. Performance is a runtime design concern, not a later optimisation pass.

## CLI

```text
platipus new my-app      scaffold a project directory
cd my-app

platipus dev app.plt     serve dist/ and rebuild when the file changes
platipus build app.plt   compile into dist/
platipus test app.plt    run the `test` blocks the program declares
platipus check app.plt   analyse without writing anything
platipus format app.plt  rewrite the file with normalised indentation
platipus run app.plt     build, then print how to serve the result
```

Every command that compiles something takes the entry file to compile, so there
is no implicit project root to guess at; only `new`, `help`, and `version` do
not. `run` builds and stops; `dev` is the command that serves. `import Name from
"./file.plt"` is resolved transitively against the file that declares it and its
components, styles, themes, apis, and tests are merged into the program. `dev`
watches every file the program imports, not just the entry, so editing an
imported module triggers a rebuild.

## Documentation

See [`docs/`](docs/). [`docs/prd.md`](docs/prd.md) is what the language is meant
to be, [`docs/guide.md`](docs/guide.md) is how to write Platipus code,
[`docs/library.md`](docs/library.md) is how to create and consume libraries
(including the `.libplt` / `p2lt` flow), [`docs/struktur.md`](docs/struktur.md)
is the structure the code is organised towards, and
[`docs/roadmap.md`](docs/roadmap.md) is what is actually implemented today.
Read the roadmap before trusting a feature: it states, per phase, what works
and what does not.

## Status

v0.1 — the core language works end to end: it compiles to a browser program,
`plt test` runs a program's own `test` blocks, and `plt dev` serves and rebuilds.
The browser APIs are real too: `fetch`, clipboard, files, `WebSocket`, and
local/session/IndexedDB storage. `Canvas`, `Editor`, `CodeEditor`, `DataGrid`,
and `Tree` all have working runtime support, each with a documented gap list.
Pure Platipus computation is proved by a package you can read:
[`examples/algoritma/`](examples/algoritma/) implements gcd, primality,
Fibonacci, factorial, Collatz, a non-mutating sort, binary search, palindromes
and FNV-1a, with a test block per algorithm stating the value it produced.

Type checking is real but partial: operand types, annotations, and arity are
enforced, while return types and anything inferred from a non-literal are not.
Not yet real: an LSP or debugger, fine-grained rendering, list virtualisation,
code splitting, tree shaking, and production output. See
[`docs/roadmap.md`](docs/roadmap.md) for the itemised list.

## License

MIT — see [LICENSE](LICENSE).

## Native Rust blocks

A Platipus file may embed a trusted Rust block; the compiler extracts it,
generates a small Cargo crate, and compiles it with the Rust toolchain that is
already on your `PATH`.

```plt
#[rust]
#[export]
fn add(a: i64, b: i64) -> i64 {
    a + b
}

app Main {
    Column {
        Text add(10, 20)
    }
}
```

Only functions marked `#[export]` (or the whole block when the block itself is
attributed `#[rust] #[export]`) become callable from Platipus. Exported
signatures use the supported scalar types - `i64, i32, i16, i8, u64, u32, u16,
u8, f64, f32, bool`. Anything else is a compile error, never a crash:
`Rust function 'foo' uses unsupported type 'HashMap<K,V>'`.

For the web target the crate is built for `wasm32-unknown-unknown`, the wasm
module is embedded in `app.js` as base64, and the generated bridge exposes each
export as `__pltRust.<name>` with automatic `BigInt`/number conversion; `i64`/
`u64` returns arrive as JS numbers. Cargo diagnostics are mapped back to the
`.plt` source location, so you never edit a temporary file. See
`examples/rust/`.

## P2LT package manager

`p2lt` is the package manager for Platipus packages, analogous to npm/cargo/go
mod within this ecosystem.

```bash
p2lt init                 # p2lt.toml, src/main.plt, .gitignore, tests/
p2lt add math             # install from a registry and record it in the lockfile
p2lt add github.com/user/repo
p2lt add github.com/user/repo#main
p2lt install ./math.libplt # install a local archive
p2lt list                 # installed dependencies
p2lt update               # refresh from the registry + rewrite p2lt.lock
p2lt remove math          # delete package + manifest/lockfile entries
p2lt search math          # search the registry
p2lt publish              # publish locally, or PUT to P2LT_REGISTRY_URL
p2lt build                # resolve, compile Platipus, extract + build Rust,
                          # write dist/
p2lt run                  # build, then serve hint
p2lt clean                # remove target/ and dist/
```

By default, the registry is a local directory (`$P2LT_REGISTRY`, otherwise the
shared Platipus store). Set `P2LT_REGISTRY_URL` to use an HTTP(S) registry.
Its minimal endpoint contract is documented in
[`docs/package-registry.md`](docs/package-registry.md). GitHub sources are
fetched with `git`; HTTP registries are accessed with `curl`. Cargo builds Rust
crates; P2LT manages Platipus packages. Installing a package may execute native
code, so packages are compiled, never eval'd; dependency build scripts are not
executed by P2LT.

| Crate                  | Responsibility                                              |
| ---------------------- | ----------------------------------------------------------- |
| `platipus-p2lt`        | Manifest, lockfile, local/HTTP registry, GitHub resolver + CLI |
