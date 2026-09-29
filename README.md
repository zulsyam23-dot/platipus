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

| Crate                    | Responsibility                                                        |
| ------------------------ | --------------------------------------------------------------------- |
| `platipus-compiler`      | Lexer → Parser → AST → Semantic → IR (verified) → Codegen → Web output |
| `platipus-runtime`       | Host side of the state, derived-value, and event contracts             |
| `platipus-standard`      | Optional standard-library helpers                                      |
| `platipus-cli`           | `platipus` / `plt` — new, dev, build, run, check, format, test          |

The runtime never depends on the standard library for core UI concepts, and the
compiler never depends on the UI runtime in order to parse.

`platipus-runtime` is the Rust side of the reactive contract: a state store, a
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
to be, [`docs/struktur.md`](docs/struktur.md) is the structure the code is
organised towards, and [`docs/roadmap.md`](docs/roadmap.md) is what is actually
implemented today. Read the roadmap before trusting a feature: it states, per
phase, what works and what does not.

## Status

v0.1 — the core language works end to end: it compiles to a browser program,
`plt test` runs a program's own `test` blocks, and `plt dev` serves and rebuilds.
The browser APIs are real too: `fetch`, clipboard, files, `WebSocket`, and
local/session/IndexedDB storage. `Canvas`, `Editor`, `CodeEditor`, `DataGrid`,
and `Tree` all have working runtime support, each with a documented gap list.
Not yet real: type checking, an LSP or debugger, fine-grained rendering, and
production output. See [`docs/roadmap.md`](docs/roadmap.md) for the itemised
list.

## License

MIT — see [LICENSE](LICENSE).
