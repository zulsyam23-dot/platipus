# Struktur Repository Platipus

Dokumen ini menjelaskan susunan repository dan pembagian tanggung jawab yang
ada saat ini. Ini adalah peta struktur aktual, bukan rancangan direktori yang
belum diimplementasikan. Untuk hubungan antarbagian sistem, lihat
[architecture.md](architecture.md).

## Ringkasan Repository

```text
platipus/
├── Cargo.toml
├── Cargo.lock
├── README.md
├── LICENSE
├── cli/
├── compiler/
├── runtime/
├── p2lt/
├── libraries/
│   ├── diagnostics/
│   ├── ir/
│   ├── language/
│   ├── reactive/
│   ├── semantic/
│   ├── standard/
│   ├── storage/
│   ├── testing/
│   └── web/
├── tests/
├── fixtures/
├── examples/
└── docs/
```

`target/`, direktori hasil build seperti `dist/`, dan hasil generate tidak
termasuk source utama. Isinya dapat berubah setiap kali build atau test
dijalankan.

## Cargo Workspace

Workspace didefinisikan di `Cargo.toml` root dan menggunakan resolver versi 3.
Member-nya adalah:

| Path | Package | Tanggung jawab |
| --- | --- | --- |
| `cli/` | `platipus-cli` | Perintah `plt` untuk build, check, dev, format, membuat project, menjalankan program, dan test. |
| `compiler/` | `platipus-compiler` | Facade compiler, pipeline kompilasi, pemuatan import, dan integrasi Rust/WASM. |
| `runtime/` | `platipus-runtime` | Facade kompatibilitas yang mengekspor ulang API reaktif. |
| `p2lt/` | `p2lt` | CLI dan library pengelola package Platipus. |
| `libraries/diagnostics/` | `platipus-diagnostics` | Span, error, warning, dan kumpulan diagnostic. |
| `libraries/ir/` | `platipus-ir` | Intermediate representation, lowering, backend contract, dan verifier. |
| `libraries/language/` | `platipus-language` | Lexer, parser, AST, metadata element dan event, serta ekstraksi blok Rust. |
| `libraries/reactive/` | `platipus-reactive` | Value dan state store, derived value, serta event bus. |
| `libraries/semantic/` | `platipus-semantic` | Pemeriksaan deklarasi, scope, tipe, element, event, dan penggunaan bahasa. |
| `libraries/standard/` | `platipus-standard` | Implementasi fungsi bawaan untuk collection, list, map, math, parsing, string, dan text. |
| `libraries/storage/` | `platipus-storage` | Kontrak penyimpanan key-value dan implementasi memory. |
| `libraries/testing/` | `platipus-testing` | Kontrak bersama untuk action dan langkah test yang ditulis dalam program. |
| `libraries/web/` | `platipus-web` | Code generator target web, runtime JavaScript, CSS, HTML, dan test runner. |

Dependency path antarpaket dideklarasikan bersama di `[workspace.dependencies]`
root. Masing-masing crate memilih dependency yang dipakainya melalui
`Cargo.toml` crate tersebut.

## Alur Kompilasi

`compiler/src/pipeline.rs` mengorkestrasi tahap-tahap kompilasi. Implementasi
bahasa, semantik, IR, dan target web tinggal di crate terpisah:

```text
Source .plt
    ↓
platipus-language: lexer → parser → AST
    ↓
platipus-semantic: pemeriksaan program
    ↓
platipus-ir: lowering → verifier
    ↓
platipus-web: codegen target web
    ↓
Artifact: index.html, app.css, app.js, dom.mjs,
          tests.mjs, dan program.json
```

Jika program memakai `import`, `compiler/src/loader.rs` memuat dan
menggabungkan module sebelum analisis. Pipeline juga dapat mengompilasi blok
Rust yang diekspor ke bridge WASM.

## Struktur Source Utama

### Compiler facade: `compiler/src/`

```text
compiler/src/
├── lib.rs
├── loader.rs
├── pipeline.rs
└── rust/
    └── mod.rs
```

`lib.rs` mengekspor facade crate yang stabil. Implementasi lexer, parser,
semantic, IR, diagnostics, dan codegen berada di crate masing-masing dan
diekspos kembali melalui facade compiler.

### Bahasa dan parser: `libraries/language/src/`

```text
language/src/
├── lib.rs
├── element.rs
├── events.rs
├── rust_extract.rs
├── rust_types.rs
├── ast/
│   ├── app.rs
│   ├── component.rs
│   ├── element.rs
│   ├── event.rs
│   ├── expression.rs
│   ├── function.rs
│   ├── mod.rs
│   ├── rust.rs
│   ├── state.rs
│   ├── statement.rs
│   └── style.rs
├── lexer/
│   ├── keyword.rs
│   ├── literal.rs
│   ├── mod.rs
│   ├── operator.rs
│   ├── rust_block.rs
│   ├── scanner.rs
│   └── token.rs
└── parser/
    ├── app.rs
    ├── component.rs
    ├── element.rs
    ├── event.rs
    ├── expression.rs
    ├── function.rs
    ├── mod.rs
    ├── state.rs
    ├── statement.rs
    └── style.rs
```

Lexer menghasilkan token; parser mengubahnya menjadi AST. AST memodelkan
program dan sintaks Platipus, bukan node DOM hasil render.

### Pemeriksaan semantik: `libraries/semantic/src/`

```text
semantic/src/
├── lib.rs
├── checker.rs
├── name_resolver.rs
├── scope.rs
└── types.rs
```

`SemanticChecker` memeriksa deklarasi dan pemakaian nama, scope, tipe, serta
konstruksi bahasa lainnya. Metadata element dan event yang dipakai pemeriksaan
berasal dari `platipus-language`.

### Intermediate representation: `libraries/ir/src/`

```text
ir/src/
├── lib.rs
├── backend.rs
├── component.rs
├── element.rs
├── event.rs
├── expression.rs
├── lower.rs
├── module.rs
├── state.rs
├── statement.rs
├── style.rs
└── verifier.rs
```

`lower.rs` menurunkan AST menjadi IR. `verifier.rs` memeriksa invariant IR
sebelum target codegen menggunakannya. `backend.rs` mendefinisikan kontrak
target dan artifact.

### Generator web: `libraries/web/src/`

```text
web/src/
├── lib.rs
└── codegen/
    ├── mod.rs
    ├── expression.rs
    ├── statement.rs
    ├── components/
    ├── elements/
    ├── events/
    ├── layout/
    ├── state/
    ├── style/
    │   └── sheet.rs
    └── web/
        ├── mod.rs
        ├── bundle.rs
        ├── css.rs
        ├── dom.rs
        ├── html.rs
        ├── tests.rs
        └── javascript/
            ├── mod.rs
            ├── api.rs
            ├── bootstrap.rs
            ├── component.rs
            └── rust.rs
```

Bagian `codegen/` memetakan IR ke akses expression, statement, component,
element, event, layout, state, dan style. `codegen/web/` menyusun artifact
target, termasuk runtime JavaScript dan shim DOM untuk runner test.

### State dan runtime: `libraries/reactive/`, `libraries/storage/`, `runtime/`

```text
reactive/src/
├── lib.rs
├── derived.rs
├── event.rs
└── state.rs

storage/src/
└── lib.rs

runtime/src/
└── lib.rs
```

`platipus-reactive` berisi kontrak state, derived value, dan event bus.
`platipus-storage` mendefinisikan abstraksi penyimpanan. `platipus-runtime`
mengekspor ulang API reaktif untuk kompatibilitas; runtime browser untuk hasil
generate disertakan oleh target web.

### CLI dan package manager

```text
cli/src/
├── lib.rs
├── main.rs
├── plt.rs
├── command/
│   ├── build.rs
│   ├── check.rs
│   ├── dev.rs
│   ├── format.rs
│   ├── mod.rs
│   ├── new.rs
│   ├── run.rs
│   └── test.rs
├── config/
│   └── mod.rs
└── output/
    ├── diagnostics.rs
    └── mod.rs

p2lt/src/
├── lib.rs
├── main.rs
├── cache.rs
├── cli.rs
├── lockfile.rs
├── manifest.rs
├── registry.rs
└── resolver.rs
```

CLI utama bernama `plt`; P2LT adalah package manager terpisah. CLI membagi
penanganan opsi, implementasi command, dan output/diagnostic ke module
tersendiri.

## Test, Fixture, dan Contoh

Test integrasi Rust berada langsung di `tests/`, dikelompokkan menurut area:

```text
tests/
├── cli/
├── codegen/
├── fixtures/
├── ir/
├── lexer/
├── loader/
├── parser/
├── rust/
├── semantic/
└── web/
```

Sebagian unit test juga berada di dalam crate, di samping implementasi yang
diuji. Fixture program `.plt` yang dipakai bersama disimpan di
`fixtures/valid/` dan `fixtures/invalid/`. Test web memiliki fixture runtime
tersendiri di `tests/web/fixtures/`.

Contoh program saat ini:

```text
examples/
├── counter.plt
├── rust/
│   └── *.plt
└── showcase/
    ├── main.plt
    └── widgets.plt
```

Direktori `dist/` atau `target/` di bawah contoh adalah output build, bukan
sumber contoh utama.

## Dokumentasi

```text
docs/
├── architecture.md
├── prd.md
├── roadmap.md
└── struktur.md
```

- `prd.md` menjelaskan kebutuhan dan spesifikasi produk.
- `roadmap.md` mencatat tahap pengembangan.
- `architecture.md` menjelaskan arsitektur dan hubungan subsystem.
- `struktur.md` memetakan direktori serta pembagian tanggung jawab source.

Perbarui peta ini jika crate, module utama, atau pengelompokan test berubah.
Hindari mencantumkan direktori hasil build sebagai bagian source permanen.
