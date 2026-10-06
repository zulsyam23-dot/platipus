Struktur Proyek Bahasa Web Application

> **Status:** Dokumen desain asli. Struktur proyek yang berlaku sekarang
> ada di [docs/architecture.md](architecture.md).

1. Tujuan Dokumen

Dokumen ini mendefinisikan struktur proyek, pembagian tanggung jawab module, aturan modularisasi, hubungan antara compiler dan runtime, serta organisasi source code untuk bahasa pemrograman yang dirancang khusus untuk membangun aplikasi web modern.

Bahasa ini tidak menjadikan HTML, CSS, dan JavaScript sebagai tiga konsep utama yang harus dipelajari pengguna secara terpisah.

Model utama bahasa:

Application
    â†“
Component
    â†“
Element
    â†“
State
    â†“
Event
    â†“
Reactive Runtime
    â†“
Web Runtime

Struktur source code harus mencerminkan model tersebut.

---

2. Prinsip Utama Struktur

2.1 Modular

Setiap domain utama memiliki module sendiri.

Contoh:

compiler/
runtime/
element/
state/
event/
style/
layout/
component/
reactive/
browser/

Tidak boleh terdapat satu file pusat yang menampung seluruh implementasi.

Contoh yang harus dihindari:

runtime/src/
â”œâ”€â”€ element.rs      # semua element
â”œâ”€â”€ event.rs        # semua event
â”œâ”€â”€ state.rs        # semua state
â””â”€â”€ runtime.rs      # seluruh runtime

Struktur tersebut akan cepat menjadi sulit dipelihara.

Struktur yang diinginkan:

runtime/src/
â”œâ”€â”€ element/
â”œâ”€â”€ event/
â”œâ”€â”€ state/
â”œâ”€â”€ style/
â”œâ”€â”€ layout/
â”œâ”€â”€ component/
â”œâ”€â”€ reactive/
â””â”€â”€ browser/

---

3. Prinsip Pembagian Module

Pembagian module harus mengikuti domain, bukan sekadar ukuran file.

Domain utama bahasa:

Lexer
Parser
AST
Semantic
Codegen

Element
Component
State
Event
Style
Layout
Reactive
Animation
Browser

Setiap domain dapat memiliki submodule.

Contoh:

state/
â”œâ”€â”€ local/
â”œâ”€â”€ shared/
â”œâ”€â”€ global/
â”œâ”€â”€ derived/
â””â”€â”€ persistent/

Hal ini membuat lokasi sebuah fitur dapat diprediksi.

Jika developer ingin mencari implementasi "DerivedState", lokasi utamanya harus jelas:

runtime/src/state/derived/

---

4. Struktur Repository Utama

Struktur awal repository:

project/
â”‚
â”œâ”€â”€ Cargo.toml
â”œâ”€â”€ Cargo.lock
â”œâ”€â”€ README.md
â”œâ”€â”€ LICENSE
â”‚
â”œâ”€â”€ docs/
â”‚   â”œâ”€â”€ prd.md
â”‚   â”œâ”€â”€ struktur.md
â”‚   â”œâ”€â”€ syntax.md
â”‚   â”œâ”€â”€ architecture.md
â”‚   â”œâ”€â”€ runtime.md
â”‚   â”œâ”€â”€ compiler.md
â”‚   â”œâ”€â”€ state.md
â”‚   â”œâ”€â”€ event.md
â”‚   â”œâ”€â”€ element.md
â”‚   â”œâ”€â”€ component.md
â”‚   â”œâ”€â”€ style.md
â”‚   â””â”€â”€ browser.md
â”‚
â”œâ”€â”€ compiler/
â”‚   â”œâ”€â”€ Cargo.toml
â”‚   â””â”€â”€ src/
â”‚
â”œâ”€â”€ runtime/
â”‚   â”œâ”€â”€ Cargo.toml
â”‚   â””â”€â”€ src/
â”‚
â”œâ”€â”€ standard/
â”‚   â”œâ”€â”€ Cargo.toml
â”‚   â””â”€â”€ src/
â”‚
â”œâ”€â”€ cli/
â”‚   â”œâ”€â”€ Cargo.toml
â”‚   â””â”€â”€ src/
â”‚
â”œâ”€â”€ tests/
â”‚   â”œâ”€â”€ parser/
â”‚   â”œâ”€â”€ semantic/
â”‚   â”œâ”€â”€ compiler/
â”‚   â”œâ”€â”€ element/
â”‚   â”œâ”€â”€ state/
â”‚   â”œâ”€â”€ event/
â”‚   â”œâ”€â”€ component/
â”‚   â”œâ”€â”€ style/
â”‚   â”œâ”€â”€ layout/
â”‚   â”œâ”€â”€ reactive/
â”‚   â”œâ”€â”€ browser/
â”‚   â””â”€â”€ integration/
â”‚
â”œâ”€â”€ examples/
â”‚   â”œâ”€â”€ counter/
â”‚   â”œâ”€â”€ form/
â”‚   â”œâ”€â”€ dashboard/
â”‚   â”œâ”€â”€ todo/
â”‚   â””â”€â”€ editor/
â”‚
â””â”€â”€ fixtures/
    â”œâ”€â”€ valid/
    â””â”€â”€ invalid/

Â«"standard/" hanya digunakan jika library standar memang diperlukan. Runtime tidak bergantung pada keberadaan standard library untuk konsep dasar UI.Â»

---

5. Workspace Cargo

Repository menggunakan Cargo Workspace.

Contoh:

[workspace]
members = [
    "compiler",
    "runtime",
    "cli",
    "standard"
]
resolver = "3"

Dependency harus memiliki arah yang jelas.

Model:

              â”Œâ”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”
              â”‚    CLI     â”‚
              â””â”€â”€â”€â”€â”€â”¬â”€â”€â”€â”€â”€â”€â”˜
                    â”‚
          â”Œâ”€â”€â”€â”€â”€â”€â”€â”€â”€â”´â”€â”€â”€â”€â”€â”€â”€â”€â”€â”
          â†“                   â†“
     â”Œâ”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”       â”Œâ”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”
     â”‚ Compiler â”‚       â”‚ Runtime  â”‚
     â””â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”˜       â””â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”˜

Compiler tidak boleh bergantung pada implementasi UI runtime secara langsung hanya untuk melakukan parsing.

---

6. Compiler

Compiler bertanggung jawab mengubah source language menjadi bentuk yang dapat dijalankan oleh target runtime.

Pipeline utama:

Source
  â†“
Lexer
  â†“
Token
  â†“
Parser
  â†“
AST
  â†“
Semantic Analysis
  â†“
Intermediate Representation
  â†“
Code Generation
  â†“
Web Output

Struktur:

compiler/
â””â”€â”€ src/
    â”œâ”€â”€ lib.rs
    â”‚
    â”œâ”€â”€ lexer/
    â”‚   â”œâ”€â”€ mod.rs
    â”‚   â”œâ”€â”€ scanner.rs
    â”‚   â”œâ”€â”€ token.rs
    â”‚   â”œâ”€â”€ keyword.rs
    â”‚   â”œâ”€â”€ operator.rs
    â”‚   â””â”€â”€ literal.rs
    â”‚
    â”œâ”€â”€ parser/
    â”‚   â”œâ”€â”€ mod.rs
    â”‚   â”œâ”€â”€ app.rs
    â”‚   â”œâ”€â”€ component.rs
    â”‚   â”œâ”€â”€ element.rs
    â”‚   â”œâ”€â”€ state.rs
    â”‚   â”œâ”€â”€ event.rs
    â”‚   â”œâ”€â”€ style.rs
    â”‚   â”œâ”€â”€ expression.rs
    â”‚   â”œâ”€â”€ statement.rs
    â”‚   â””â”€â”€ function.rs
    â”‚
    â”œâ”€â”€ ast/
    â”‚   â”œâ”€â”€ mod.rs
    â”‚   â”œâ”€â”€ app.rs
    â”‚   â”œâ”€â”€ component.rs
    â”‚   â”œâ”€â”€ element.rs
    â”‚   â”œâ”€â”€ state.rs
    â”‚   â”œâ”€â”€ event.rs
    â”‚   â”œâ”€â”€ style.rs
    â”‚   â”œâ”€â”€ expression.rs
    â”‚   â”œâ”€â”€ statement.rs
    â”‚   â””â”€â”€ function.rs
    â”‚
    â”œâ”€â”€ semantic/
    â”‚   â”œâ”€â”€ mod.rs
    â”‚   â”œâ”€â”€ scope.rs
    â”‚   â”œâ”€â”€ symbol.rs
    â”‚   â”œâ”€â”€ type_check.rs
    â”‚   â”œâ”€â”€ component.rs
    â”‚   â”œâ”€â”€ element.rs
    â”‚   â”œâ”€â”€ state.rs
    â”‚   â”œâ”€â”€ event.rs
    â”‚   â”œâ”€â”€ expression.rs
    â”‚   â””â”€â”€ diagnostics.rs
    â”‚
    â”œâ”€â”€ ir/
    â”‚   â”œâ”€â”€ mod.rs
    â”‚   â”œâ”€â”€ module.rs
    â”‚   â”œâ”€â”€ component.rs
    â”‚   â”œâ”€â”€ element.rs
    â”‚   â”œâ”€â”€ state.rs
    â”‚   â”œâ”€â”€ event.rs
    â”‚   â”œâ”€â”€ expression.rs
    â”‚   â””â”€â”€ verifier.rs
    â”‚
    â”œâ”€â”€ codegen/
    â”‚   â”œâ”€â”€ mod.rs
    â”‚   â”œâ”€â”€ elements/
    â”‚   â”œâ”€â”€ components/
    â”‚   â”œâ”€â”€ state/
    â”‚   â”œâ”€â”€ events/
    â”‚   â”œâ”€â”€ style/
    â”‚   â”œâ”€â”€ layout/
    â”‚   â””â”€â”€ web/
    â”‚
    â””â”€â”€ diagnostics/
        â”œâ”€â”€ mod.rs
        â”œâ”€â”€ error.rs
        â”œâ”€â”€ warning.rs
        â””â”€â”€ span.rs

---

7. Lexer

Lexer hanya bertanggung jawab mengubah karakter menjadi token.

Struktur:

lexer/
â”œâ”€â”€ mod.rs
â”œâ”€â”€ scanner.rs
â”œâ”€â”€ token.rs
â”œâ”€â”€ keyword.rs
â”œâ”€â”€ operator.rs
â””â”€â”€ literal.rs

Contoh source:

Button "Save"

menjadi:

Identifier("Button")
String("Save")

Lexer tidak boleh mengetahui arti "Button" sebagai UI element.

Pengenalan bahwa "Button" merupakan primitive element adalah tanggung jawab parser/semantic/runtime registry.

---

8. Parser

Parser mengubah token menjadi AST.

Struktur:

parser/
â”œâ”€â”€ mod.rs
â”œâ”€â”€ app.rs
â”œâ”€â”€ component.rs
â”œâ”€â”€ element.rs
â”œâ”€â”€ state.rs
â”œâ”€â”€ event.rs
â”œâ”€â”€ style.rs
â”œâ”€â”€ expression.rs
â”œâ”€â”€ statement.rs
â””â”€â”€ function.rs

Parser tidak bertanggung jawab menjalankan aplikasi.

Contoh:

state count = 0

menghasilkan struktur AST state.

Parser tidak langsung melakukan:

runtime.set_state(...)

---

9. AST

AST merepresentasikan struktur bahasa.

Contoh konsep:

App
 â”œâ”€â”€ Component
 â”‚    â”œâ”€â”€ Input
 â”‚    â”œâ”€â”€ State
 â”‚    â”œâ”€â”€ Element
 â”‚    â””â”€â”€ Event

Struktur AST:

ast/
â”œâ”€â”€ app.rs
â”œâ”€â”€ component.rs
â”œâ”€â”€ element.rs
â”œâ”€â”€ state.rs
â”œâ”€â”€ event.rs
â”œâ”€â”€ style.rs
â”œâ”€â”€ expression.rs
â”œâ”€â”€ statement.rs
â””â”€â”€ function.rs

AST harus merepresentasikan bahasa, bukan detail implementasi browser.

Contoh:

Button

tidak boleh langsung menjadi:

HTMLButtonElement

di AST.

---

10. Semantic Analysis

Semantic analyzer memeriksa apakah program valid secara makna.

Contoh:

state count = 0

Button "Increment" {
    on click {
        count += 1
    }
}

Semantic analyzer memeriksa:

- "count" memang ada.
- "count" dapat diubah.
- event "click" valid.
- "Button" merupakan element yang tersedia.
- expression valid.
- scope valid.
- input component valid.
- type valid.

Struktur:

semantic/
â”œâ”€â”€ scope.rs
â”œâ”€â”€ symbol.rs
â”œâ”€â”€ type_check.rs
â”œâ”€â”€ component.rs
â”œâ”€â”€ element.rs
â”œâ”€â”€ state.rs
â”œâ”€â”€ event.rs
â”œâ”€â”€ expression.rs
â””â”€â”€ diagnostics.rs

---

11. IR

IR digunakan sebagai bentuk perantara antara semantic analysis dan code generation.

Struktur:

ir/
â”œâ”€â”€ mod.rs
â”œâ”€â”€ module.rs
â”œâ”€â”€ component.rs
â”œâ”€â”€ element.rs
â”œâ”€â”€ state.rs
â”œâ”€â”€ event.rs
â”œâ”€â”€ expression.rs
â””â”€â”€ verifier.rs

IR harus memiliki verifier.

Aturan:

Perubahan IR
      â†“
Update verifier
      â†“
Regression test
      â†“
Codegen test

IR tidak boleh berubah tanpa memperbarui verifier.

---

12. Code Generation

Codegen menerjemahkan IR menjadi target.

Struktur:

codegen/
â”œâ”€â”€ mod.rs
â”‚
â”œâ”€â”€ elements/
â”‚   â”œâ”€â”€ mod.rs
â”‚   â”œâ”€â”€ button.rs
â”‚   â”œâ”€â”€ input.rs
â”‚   â”œâ”€â”€ text.rs
â”‚   â””â”€â”€ ...
â”‚
â”œâ”€â”€ components/
â”‚   â”œâ”€â”€ mod.rs
â”‚   â””â”€â”€ component.rs
â”‚
â”œâ”€â”€ state/
â”‚   â”œâ”€â”€ mod.rs
â”‚   â”œâ”€â”€ local.rs
â”‚   â”œâ”€â”€ shared.rs
â”‚   â”œâ”€â”€ global.rs
â”‚   â””â”€â”€ derived.rs
â”‚
â”œâ”€â”€ events/
â”‚   â”œâ”€â”€ mod.rs
â”‚   â”œâ”€â”€ pointer.rs
â”‚   â”œâ”€â”€ keyboard.rs
â”‚   â”œâ”€â”€ input.rs
â”‚   â””â”€â”€ custom.rs
â”‚
â”œâ”€â”€ style/
â”‚   â”œâ”€â”€ mod.rs
â”‚   â””â”€â”€ compiler.rs
â”‚
â”œâ”€â”€ layout/
â”‚   â”œâ”€â”€ mod.rs
â”‚   â””â”€â”€ compiler.rs
â”‚
â””â”€â”€ web/
    â”œâ”€â”€ mod.rs
    â”œâ”€â”€ html.rs
    â”œâ”€â”€ css.rs
    â”œâ”€â”€ javascript.rs
    â”œâ”€â”€ wasm.rs
    â””â”€â”€ bundle.rs

HTML/CSS/JavaScript di sini adalah target implementasi, bukan konsep utama bahasa.

---

13. Runtime

Runtime adalah sistem yang menjalankan hasil compiler.

Struktur utama:

runtime/
â””â”€â”€ src/
    â”œâ”€â”€ lib.rs
    â”‚
    â”œâ”€â”€ element/
    â”œâ”€â”€ component/
    â”œâ”€â”€ state/
    â”œâ”€â”€ event/
    â”œâ”€â”€ style/
    â”œâ”€â”€ layout/
    â”œâ”€â”€ reactive/
    â”œâ”€â”€ animation/
    â”œâ”€â”€ browser/
    â”œâ”€â”€ render/
    â”œâ”€â”€ scheduler/
    â””â”€â”€ error/

---

14. Element Runtime

Element adalah primitive UI bawaan bahasa.

Element bukan salinan HTML.

Contoh primitive:

Button
Input
Text
Heading
Image

Container
Row
Column
Stack
Grid
Panel
Splitter
Scroll

Dialog
Popover
Tooltip
Tabs
Sidebar
Toolbar

List
Table
Tree
DataGrid

Canvas
Editor
CodeEditor
Viewport

Struktur:

element/
â”œâ”€â”€ mod.rs
â”‚
â”œâ”€â”€ button/
â”‚   â”œâ”€â”€ mod.rs
â”‚   â”œâ”€â”€ state.rs
â”‚   â”œâ”€â”€ event.rs
â”‚   â”œâ”€â”€ style.rs
â”‚   â””â”€â”€ render.rs
â”‚
â”œâ”€â”€ input/
â”‚   â”œâ”€â”€ mod.rs
â”‚   â”œâ”€â”€ state.rs
â”‚   â”œâ”€â”€ event.rs
â”‚   â”œâ”€â”€ style.rs
â”‚   â””â”€â”€ render.rs
â”‚
â”œâ”€â”€ text/
â”‚   â”œâ”€â”€ mod.rs
â”‚   â”œâ”€â”€ style.rs
â”‚   â””â”€â”€ render.rs
â”‚
â”œâ”€â”€ image/
â”‚   â”œâ”€â”€ mod.rs
â”‚   â”œâ”€â”€ state.rs
â”‚   â”œâ”€â”€ style.rs
â”‚   â””â”€â”€ render.rs
â”‚
â”œâ”€â”€ container/
â”‚   â”œâ”€â”€ mod.rs
â”‚   â”œâ”€â”€ layout.rs
â”‚   â”œâ”€â”€ style.rs
â”‚   â””â”€â”€ render.rs
â”‚
â”œâ”€â”€ row/
â”œâ”€â”€ column/
â”œâ”€â”€ stack/
â”œâ”€â”€ grid/
â”œâ”€â”€ panel/
â”œâ”€â”€ splitter/
â”œâ”€â”€ scroll/
â”‚
â”œâ”€â”€ list/
â”œâ”€â”€ table/
â”œâ”€â”€ tree/
â”œâ”€â”€ data_grid/
â”‚
â”œâ”€â”€ dialog/
â”œâ”€â”€ popover/
â”œâ”€â”€ tooltip/
â”œâ”€â”€ tabs/
â”œâ”€â”€ sidebar/
â”œâ”€â”€ toolbar/
â”‚
â”œâ”€â”€ canvas/
â”œâ”€â”€ editor/
â”œâ”€â”€ code_editor/
â””â”€â”€ viewport/

---

15. Struktur Internal Element

Element yang sederhana tidak harus memiliki seluruh file.

Contoh sederhana:

text/
â”œâ”€â”€ mod.rs
â”œâ”€â”€ style.rs
â””â”€â”€ render.rs

Element kompleks:

data_grid/
â”œâ”€â”€ mod.rs
â”œâ”€â”€ state.rs
â”œâ”€â”€ event.rs
â”œâ”€â”€ selection.rs
â”œâ”€â”€ sorting.rs
â”œâ”€â”€ filtering.rs
â”œâ”€â”€ virtualization.rs
â”œâ”€â”€ column.rs
â”œâ”€â”€ row.rs
â”œâ”€â”€ style.rs
â”œâ”€â”€ layout.rs
â””â”€â”€ render.rs

Prinsipnya:

Â«Kompleksitas element menentukan jumlah submodule.Â»

Jangan membuat struktur kosong hanya demi mengikuti pola.

---

16. Element Registry

Runtime membutuhkan registry untuk mengenali primitive element.

Contoh:

element/
â”œâ”€â”€ mod.rs
â”œâ”€â”€ registry.rs
â”œâ”€â”€ button/
â”œâ”€â”€ input/
â””â”€â”€ ...

Registry bertugas mengetahui:

Button
Input
Text
Container
Grid
Dialog
...

Registry tidak menjadi tempat implementasi semua element.

Implementasi tetap berada pada module masing-masing.

---

17. State System

State adalah salah satu fondasi utama bahasa.

Struktur:

state/
â”œâ”€â”€ mod.rs
â”‚
â”œâ”€â”€ local/
â”‚   â”œâ”€â”€ mod.rs
â”‚   â”œâ”€â”€ state.rs
â”‚   â””â”€â”€ storage.rs
â”‚
â”œâ”€â”€ shared/
â”‚   â”œâ”€â”€ mod.rs
â”‚   â”œâ”€â”€ state.rs
â”‚   â””â”€â”€ scope.rs
â”‚
â”œâ”€â”€ global/
â”‚   â”œâ”€â”€ mod.rs
â”‚   â””â”€â”€ state.rs
â”‚
â”œâ”€â”€ derived/
â”‚   â”œâ”€â”€ mod.rs
â”‚   â”œâ”€â”€ state.rs
â”‚   â””â”€â”€ dependency.rs
â”‚
â”œâ”€â”€ persistent/
â”‚   â”œâ”€â”€ mod.rs
â”‚   â”œâ”€â”€ state.rs
â”‚   â””â”€â”€ storage.rs
â”‚
â”œâ”€â”€ dependency/
â”‚   â”œâ”€â”€ mod.rs
â”‚   â”œâ”€â”€ graph.rs
â”‚   â”œâ”€â”€ tracker.rs
â”‚   â””â”€â”€ watcher.rs
â”‚
â”œâ”€â”€ transaction/
â”‚   â”œâ”€â”€ mod.rs
â”‚   â””â”€â”€ transaction.rs
â”‚
â””â”€â”€ scheduler/
    â”œâ”€â”€ mod.rs
    â””â”€â”€ scheduler.rs

---

18. Jenis State

Bahasa mendukung beberapa scope state.

Local

state count = 0

Scope:

Component

Shared

shared state user

Scope dapat digunakan beberapa component.

Global

global state theme = "dark"

Scope aplikasi.

Derived

derived fullName = firstName + " " + lastName

Nilainya berasal dari state lain.

Persistent

persistent state settings

Nilainya dapat disimpan dan dipulihkan.

---

19. Dependency Graph

Reactive runtime harus mengetahui hubungan:

State
  â†“
Derived State
  â†“
Component
  â†“
Element

Contoh:

count
  â†“
doubleCount
  â†“
Text

Jika:

count = 10

berubah menjadi:

count = 11

runtime hanya memperbarui bagian yang bergantung pada "count".

Tidak perlu melakukan render seluruh aplikasi.

---

20. Event System

Event dipisahkan berdasarkan domain.

event/
â”œâ”€â”€ mod.rs
â”‚
â”œâ”€â”€ pointer/
â”‚   â”œâ”€â”€ mod.rs
â”‚   â”œâ”€â”€ click.rs
â”‚   â”œâ”€â”€ double_click.rs
â”‚   â”œâ”€â”€ mouse.rs
â”‚   â””â”€â”€ pointer.rs
â”‚
â”œâ”€â”€ keyboard/
â”‚   â”œâ”€â”€ mod.rs
â”‚   â”œâ”€â”€ key_down.rs
â”‚   â”œâ”€â”€ key_up.rs
â”‚   â””â”€â”€ key_press.rs
â”‚
â”œâ”€â”€ input/
â”‚   â”œâ”€â”€ mod.rs
â”‚   â”œâ”€â”€ input.rs
â”‚   â”œâ”€â”€ change.rs
â”‚   â””â”€â”€ composition.rs
â”‚
â”œâ”€â”€ focus/
â”‚   â”œâ”€â”€ mod.rs
â”‚   â”œâ”€â”€ focus.rs
â”‚   â””â”€â”€ blur.rs
â”‚
â”œâ”€â”€ form/
â”‚   â”œâ”€â”€ mod.rs
â”‚   â”œâ”€â”€ submit.rs
â”‚   â”œâ”€â”€ reset.rs
â”‚   â””â”€â”€ validation.rs
â”‚
â”œâ”€â”€ drag/
â”‚   â”œâ”€â”€ mod.rs
â”‚   â”œâ”€â”€ drag_start.rs
â”‚   â”œâ”€â”€ drag.rs
â”‚   â”œâ”€â”€ drag_over.rs
â”‚   â”œâ”€â”€ drop.rs
â”‚   â””â”€â”€ drag_end.rs
â”‚
â”œâ”€â”€ clipboard/
â”‚   â”œâ”€â”€ mod.rs
â”‚   â”œâ”€â”€ copy.rs
â”‚   â”œâ”€â”€ cut.rs
â”‚   â””â”€â”€ paste.rs
â”‚
â”œâ”€â”€ media/
â”‚   â”œâ”€â”€ mod.rs
â”‚   â”œâ”€â”€ play.rs
â”‚   â”œâ”€â”€ pause.rs
â”‚   â”œâ”€â”€ ended.rs
â”‚   â””â”€â”€ time_update.rs
â”‚
â”œâ”€â”€ lifecycle/
â”‚   â”œâ”€â”€ mod.rs
â”‚   â”œâ”€â”€ create.rs
â”‚   â”œâ”€â”€ mount.rs
â”‚   â”œâ”€â”€ update.rs
â”‚   â””â”€â”€ destroy.rs
â”‚
â”œâ”€â”€ animation/
â”‚   â”œâ”€â”€ mod.rs
â”‚   â”œâ”€â”€ animation.rs
â”‚   â””â”€â”€ transition.rs
â”‚
â”œâ”€â”€ application/
â”‚   â”œâ”€â”€ mod.rs
â”‚   â”œâ”€â”€ state_change.rs
â”‚   â”œâ”€â”€ route_change.rs
â”‚   â””â”€â”€ component_change.rs
â”‚
â””â”€â”€ custom/
    â”œâ”€â”€ mod.rs
    â”œâ”€â”€ event.rs
    â””â”€â”€ emitter.rs

---

21. Syntax Event

Semua event menggunakan satu bentuk utama:

on click {
    ...
}

Tidak diperbolehkan membuat banyak gaya syntax untuk konsep yang sama.

Hindari:

onclick
@click
onClick
click={}

Bahasa menggunakan:

on click {
}

---

22. Event Object

Event dapat menyediakan data:

on input {
    username = event.value
}

Keyboard:

on keydown {
    if event.key == "Enter" {
        submit()
    }
}

Pointer:

on pointermove {
    position = event.position
}

---

23. Custom Event

Component dapat mengirim event:

emit selected(user)

Component parent:

UserCard user: user {
    on selected {
        selectedUser = event.value
    }
}

Implementasi custom event berada di:

event/custom/

---

24. Component

Component adalah unit utama untuk membangun UI yang dapat digunakan kembali.

Struktur runtime:

component/
â”œâ”€â”€ mod.rs
â”œâ”€â”€ instance.rs
â”œâ”€â”€ tree.rs
â”œâ”€â”€ lifecycle.rs
â”œâ”€â”€ input.rs
â”œâ”€â”€ output.rs
â”œâ”€â”€ context.rs
â””â”€â”€ registry.rs

Component memiliki:

Input
State
Element
Event
Style
Lifecycle
Output

---

25. Component Lifecycle

Lifecycle:

Create
  â†“
Mount
  â†“
Update
  â†“
Destroy

Event lifecycle:

on create {
}

on mount {
}

on update {
}

on destroy {
}

Implementasi lifecycle berada di:

component/lifecycle.rs

dan event lifecycle pada:

event/lifecycle/

---

26. Style System

Style bukan HTML/CSS mentah sebagai konsep utama bahasa.

Struktur:

style/
â”œâ”€â”€ mod.rs
â”œâ”€â”€ property/
â”‚   â”œâ”€â”€ mod.rs
â”‚   â”œâ”€â”€ color.rs
â”‚   â”œâ”€â”€ size.rs
â”‚   â”œâ”€â”€ spacing.rs
â”‚   â”œâ”€â”€ typography.rs
â”‚   â”œâ”€â”€ border.rs
â”‚   â””â”€â”€ shadow.rs
â”‚
â”œâ”€â”€ state/
â”‚   â”œâ”€â”€ mod.rs
â”‚   â”œâ”€â”€ normal.rs
â”‚   â”œâ”€â”€ hover.rs
â”‚   â”œâ”€â”€ pressed.rs
â”‚   â”œâ”€â”€ focused.rs
â”‚   â”œâ”€â”€ disabled.rs
â”‚   â””â”€â”€ selected.rs
â”‚
â”œâ”€â”€ theme/
â”‚   â”œâ”€â”€ mod.rs
â”‚   â”œâ”€â”€ theme.rs
â”‚   â””â”€â”€ token.rs
â”‚
â”œâ”€â”€ responsive/
â”‚   â”œâ”€â”€ mod.rs
â”‚   â””â”€â”€ breakpoint.rs
â”‚
â””â”€â”€ resolver.rs

---

27. Style Syntax

Contoh:

Button "Save" {
    style {
        background: "#2563eb"
        color: "white"
        radius: 8
        padding: 12
    }
}

State style:

style {
    normal {
        background: "#2563eb"
    }

    hover {
        background: "#1d4ed8"
    }

    pressed {
        background: "#1e40af"
    }

    disabled {
        opacity: 0.5
    }
}

---

28. Layout

Layout merupakan subsystem tersendiri.

layout/
â”œâ”€â”€ mod.rs
â”œâ”€â”€ node.rs
â”œâ”€â”€ constraints.rs
â”œâ”€â”€ measurement.rs
â”œâ”€â”€ alignment.rs
â”œâ”€â”€ sizing.rs
â”œâ”€â”€ spacing.rs
â”œâ”€â”€ overflow.rs
â”œâ”€â”€ responsive.rs
â”œâ”€â”€ scroll.rs
â”œâ”€â”€ resize.rs
â””â”€â”€ engine.rs

Primitive layout:

Container
Row
Column
Stack
Grid
Panel
Splitter
Spacer
Scroll

---

29. Layout Responsibility

Layout engine menangani:

- ukuran
- posisi
- padding
- margin/spacing
- alignment
- minimum size
- maximum size
- wrapping
- overflow
- scrolling
- responsive layout
- nested layout
- resizing

Element tidak boleh masing-masing memiliki implementasi layout engine sendiri.

Element menggunakan subsystem:

element
   â†“
layout

---

30. Reactive Runtime

Reactive runtime adalah penghubung antara state, event dan UI.

Struktur:

reactive/
â”œâ”€â”€ mod.rs
â”œâ”€â”€ signal.rs
â”œâ”€â”€ dependency.rs
â”œâ”€â”€ effect.rs
â”œâ”€â”€ watcher.rs
â”œâ”€â”€ scheduler.rs
â”œâ”€â”€ batch.rs
â”œâ”€â”€ update.rs
â””â”€â”€ invalidation.rs

Model:

Event
  â†“
State Change
  â†“
Dependency Tracking
  â†“
Invalidation
  â†“
Scheduler
  â†“
Minimal UI Update

---

31. Scheduler

Scheduler bertugas mengatur update reactive.

Contoh:

State A berubah
State B berubah
State C berubah
        â†“
     Batch
        â†“
Dependency calculation
        â†“
UI update

Tujuannya menghindari update berulang yang tidak diperlukan.

---

32. Invalidation

Runtime harus dapat menentukan bagian yang perlu diperbarui.

Contoh:

state count
   â†“
Text count

Ketika "count" berubah:

count invalid
   â†“
Text invalid

Element lain yang tidak bergantung pada "count" tidak perlu dirender ulang.

---

33. Animation

Animation subsystem:

animation/
â”œâ”€â”€ mod.rs
â”œâ”€â”€ animation.rs
â”œâ”€â”€ transition.rs
â”œâ”€â”€ timeline.rs
â”œâ”€â”€ keyframe.rs
â””â”€â”€ scheduler.rs

Animation harus dapat berintegrasi dengan:

Element
Style
State
Event
Reactive Runtime

Contoh konsep:

on click {
    open = true
}

Kemudian style/runtime dapat menentukan transition.

---

34. Browser API

Browser functionality dikelompokkan secara terpisah.

browser/
â”œâ”€â”€ mod.rs
â”œâ”€â”€ fetch/
â”œâ”€â”€ websocket/
â”œâ”€â”€ storage/
â”œâ”€â”€ clipboard/
â”œâ”€â”€ file/
â”œâ”€â”€ url/
â”œâ”€â”€ history/
â”œâ”€â”€ media/
â”œâ”€â”€ canvas/
â”œâ”€â”€ worker/
â””â”€â”€ notification/

Contoh:

browser.fetch(...)

atau API tingkat bahasa yang sesuai dengan desain akhir.

Browser API tidak boleh membuat developer harus menggunakan DOM secara langsung untuk kebutuhan UI normal.

---

35. Render System

Rendering dipisahkan dari element definition.

render/
â”œâ”€â”€ mod.rs
â”œâ”€â”€ tree.rs
â”œâ”€â”€ node.rs
â”œâ”€â”€ renderer.rs
â”œâ”€â”€ mount.rs
â”œâ”€â”€ patch.rs
â””â”€â”€ diff.rs

Element mendefinisikan perilakunya.

Render system menentukan bagaimana tree tersebut diwujudkan pada target.

Model:

Element
   â†“
Render Node
   â†“
Render Tree
   â†“
Renderer
   â†“
Browser

---

36. DOM Bukan Model Utama

Bahasa tidak mendesain aplikasi berdasarkan:

DOM
 â†“
HTML
 â†“
JavaScript

Model utama:

Application
 â†“
Component
 â†“
Element
 â†“
State
 â†“
Event
 â†“
Reactive Runtime
 â†“
Renderer
 â†“
Browser

DOM dapat digunakan sebagai implementasi target browser.

---

37. CLI

CLI menjadi interface pengguna untuk compiler.

Struktur:

cli/
â””â”€â”€ src/
    â”œâ”€â”€ main.rs
    â”œâ”€â”€ command/
    â”‚   â”œâ”€â”€ mod.rs
    â”‚   â”œâ”€â”€ build.rs
    â”‚   â”œâ”€â”€ dev.rs
    â”‚   â”œâ”€â”€ run.rs
    â”‚   â”œâ”€â”€ check.rs
    â”‚   â””â”€â”€ format.rs
    â”‚
    â”œâ”€â”€ config/
    â”‚   â””â”€â”€ mod.rs
    â”‚
    â””â”€â”€ output/
        â”œâ”€â”€ mod.rs
        â””â”€â”€ diagnostics.rs

Contoh command:

lang build
lang dev
lang run
lang check
lang format

---

38. Testing

Testing tidak boleh hanya mengandalkan satu jenis test.

Struktur:

tests/
â”œâ”€â”€ lexer/
â”œâ”€â”€ parser/
â”œâ”€â”€ semantic/
â”œâ”€â”€ ir/
â”œâ”€â”€ codegen/
â”œâ”€â”€ element/
â”œâ”€â”€ state/
â”œâ”€â”€ event/
â”œâ”€â”€ component/
â”œâ”€â”€ style/
â”œâ”€â”€ layout/
â”œâ”€â”€ reactive/
â”œâ”€â”€ browser/
â””â”€â”€ integration/

---

39. Lexer Test

Contoh:

Input:
Button "Save"

Expected:

Identifier("Button")
String("Save")

---

40. Parser Test

Input:

state count = 0

Harus menghasilkan AST state yang benar.

Test harus memastikan:

name = count
value = 0

---

41. Semantic Test

Program:

state count = 0

Button "Add" {
    on click {
        count += 1
    }
}

harus valid.

Sedangkan:

Button "Add" {
    on unknown_event {
    }
}

harus menghasilkan diagnostic yang sesuai jika event tersebut memang tidak tersedia.

---

42. State Test

State test harus mencakup:

local
shared
global
derived
persistent
dependency
transaction

Contoh:

count = 1
doubleCount = count * 2

Ketika:

count = 2

maka:

doubleCount = 4

---

43. Event Test

Test event harus memastikan:

event
 â†“
handler
 â†“
state mutation
 â†“
reactive update

Contoh:

click
 â†“
count += 1
 â†“
Text update

---

44. Element Test

Setiap primitive element yang kompleks dapat memiliki test sendiri.

Contoh:

tests/element/button/
â”œâ”€â”€ creation.rs
â”œâ”€â”€ state.rs
â”œâ”€â”€ event.rs
â”œâ”€â”€ style.rs
â””â”€â”€ interaction.rs

Untuk element sederhana dapat digabung secukupnya.

---

45. Integration Test

Integration test menguji pipeline penuh.

Source
  â†“
Compiler
  â†“
Generated Output
  â†“
Runtime
  â†“
Browser
  â†“
Expected Behavior

Contoh:

counter.lang

menghasilkan aplikasi yang ketika tombol ditekan:

0
1
2
3

---

46. End-to-End Test

E2E harus menguji aplikasi sebenarnya.

Minimal:

source
 â†“
compile
 â†“
build
 â†“
serve
 â†“
execute
 â†“
interaction
 â†“
verify output

Jangan menganggap:

cargo test

saja sudah cukup untuk memastikan compiler dan runtime bekerja.

---

47. Fixtures

Program contoh untuk testing disimpan pada:

fixtures/
â”œâ”€â”€ valid/
â””â”€â”€ invalid/

Contoh:

fixtures/
â”œâ”€â”€ valid/
â”‚   â”œâ”€â”€ basic_app/
â”‚   â”œâ”€â”€ counter/
â”‚   â”œâ”€â”€ state/
â”‚   â”œâ”€â”€ event/
â”‚   â”œâ”€â”€ component/
â”‚   â””â”€â”€ layout/
â”‚
â””â”€â”€ invalid/
    â”œâ”€â”€ unknown_element/
    â”œâ”€â”€ unknown_event/
    â”œâ”€â”€ invalid_state/
    â””â”€â”€ type_error/

---

48. Examples

Examples adalah program yang dapat dibaca developer.

examples/
â”œâ”€â”€ counter/
â”œâ”€â”€ todo/
â”œâ”€â”€ form/
â”œâ”€â”€ dashboard/
â”œâ”€â”€ data_table/
â”œâ”€â”€ editor/
â””â”€â”€ application/

Examples bukan pengganti integration test.

---

49. Dokumentasi

Dokumentasi dipisahkan berdasarkan domain.

docs/
â”œâ”€â”€ prd.md
â”œâ”€â”€ struktur.md
â”œâ”€â”€ syntax.md
â”œâ”€â”€ architecture.md
â”œâ”€â”€ compiler.md
â”œâ”€â”€ runtime.md
â”œâ”€â”€ element.md
â”œâ”€â”€ component.md
â”œâ”€â”€ state.md
â”œâ”€â”€ event.md
â”œâ”€â”€ style.md
â”œâ”€â”€ layout.md
â”œâ”€â”€ reactive.md
â””â”€â”€ browser.md

Pembagian:

prd.md

menjelaskan apa yang ingin dibuat.

struktur.md

menjelaskan bagaimana source code diorganisasikan.

architecture.md

menjelaskan bagaimana subsystem saling berhubungan.

syntax.md

menjelaskan cara pengguna menulis bahasa.

---

50. Aturan Dependency

Dependency antar-module harus memiliki arah yang jelas.

Contoh:

Compiler
   â†“
AST
   â†“
Semantic
   â†“
IR
   â†“
Codegen

Runtime:

Element
   â†“
Layout
   â†“
Render

State
   â†“
Reactive
   â†“
Render

Event
   â†“
State
   â†“
Reactive

Tidak boleh membuat dependency circular tanpa alasan arsitektural yang kuat.

---

51. Element Tidak Boleh Menjadi Runtime Besar

Contoh buruk:

button.rs

berisi:

Button
State
Event
Style
Layout
Renderer
Scheduler
Browser API

Ini harus dipecah.

Contoh yang benar:

button/
â”œâ”€â”€ mod.rs
â”œâ”€â”€ state.rs
â”œâ”€â”€ event.rs
â”œâ”€â”€ style.rs
â””â”€â”€ render.rs

---

52. Module "mod.rs"

"mod.rs" digunakan terutama sebagai entry point module.

Contoh:

pub mod state;
pub mod event;
pub mod style;
pub mod render;

"mod.rs" tidak boleh menjadi tempat menampung seluruh implementasi.

---

53. Shared Abstraction

Jangan membuat abstraction hanya karena dua module memiliki nama yang mirip.

Contoh:

Button
Input
Dialog

tidak otomatis harus menggunakan satu abstraction besar.

Abstraction dibuat ketika memang terdapat behavior yang benar-benar sama.

Contoh yang masuk akal:

Focusable
Interactive
Stylable
Mountable

Namun abstraction harus tetap kecil dan jelas.

---

54. Jangan Overengineering

Struktur modular bukan berarti semua hal harus memiliki 20 layer.

Contoh element sederhana:

text/
â”œâ”€â”€ mod.rs
â””â”€â”€ render.rs

sudah cukup.

Tidak perlu:

text/
â”œâ”€â”€ mod.rs
â”œâ”€â”€ state.rs
â”œâ”€â”€ state_manager.rs
â”œâ”€â”€ event.rs
â”œâ”€â”€ event_manager.rs
â”œâ”€â”€ renderer.rs
â”œâ”€â”€ render_pipeline.rs
â”œâ”€â”€ abstraction.rs
â””â”€â”€ factory.rs

jika element tersebut tidak membutuhkan semuanya.

---

55. Aturan Penambahan Element Baru

Ketika menambahkan element baru:

NewElement

buat:

runtime/src/element/new_element/

Minimal:

mod.rs

Jika kompleksitas membutuhkan:

state.rs
event.rs
style.rs
layout.rs
render.rs

tambahkan hanya yang diperlukan.

Kemudian tambahkan registry:

element/registry.rs

dan compiler/codegen bila diperlukan.

Testing:

tests/element/new_element/

---

56. Aturan Penambahan Event Baru

Event baru harus ditempatkan berdasarkan kategorinya.

Contoh:

pointer
keyboard
input
focus
form
drag
clipboard
media
lifecycle
animation
application
custom

Jangan membuat:

event/all_events.rs

yang terus membesar.

---

57. Aturan Penambahan State Baru

State baru harus masuk ke domain yang tepat.

Contoh:

Local
Shared
Global
Derived
Persistent

Jika fitur membutuhkan dependency tracking:

state/dependency/

Jika membutuhkan persistence:

state/persistent/

---

58. Aturan Perubahan Syntax

Perubahan syntax harus memperbarui:

lexer
parser
AST
semantic
IR
codegen
tests
docs

Tidak boleh hanya mengubah parser.

Alur:

Syntax change
      â†“
Lexer
      â†“
Parser
      â†“
AST
      â†“
Semantic
      â†“
IR
      â†“
Codegen
      â†“
Runtime
      â†“
Tests
      â†“
Documentation

---

59. Aturan Perubahan IR

IR merupakan kontrak internal compiler.

Jika IR berubah:

IR
 â†“
Verifier update
 â†“
Codegen update
 â†“
Regression test

Verifier harus selalu mencerminkan aturan IR terbaru.

---

60. Aturan Bug Compiler

Setiap bug compiler harus memiliki regression test.

Alur:

Bug ditemukan
     â†“
Buat test yang gagal
     â†“
Perbaiki compiler
     â†“
Test berhasil
     â†“
Test tetap dipertahankan

Jangan hanya memperbaiki source code tanpa test.

---

61. Aturan Runtime

Runtime harus diuji pada level:

Unit
 â†“
Subsystem
 â†“
Integration
 â†“
End-to-End

Contoh:

Button
 â†“
click
 â†“
state change
 â†“
reactive update
 â†“
render

harus dapat diuji sebagai satu alur.

---

62. Struktur Akhir yang Direkomendasikan

Struktur repository secara keseluruhan:

project/
â”‚
â”œâ”€â”€ Cargo.toml
â”œâ”€â”€ Cargo.lock
â”œâ”€â”€ README.md
â”œâ”€â”€ LICENSE
â”‚
â”œâ”€â”€ docs/
â”‚   â”œâ”€â”€ prd.md
â”‚   â”œâ”€â”€ struktur.md
â”‚   â”œâ”€â”€ syntax.md
â”‚   â”œâ”€â”€ architecture.md
â”‚   â”œâ”€â”€ compiler.md
â”‚   â”œâ”€â”€ runtime.md
â”‚   â”œâ”€â”€ element.md
â”‚   â”œâ”€â”€ component.md
â”‚   â”œâ”€â”€ state.md
â”‚   â”œâ”€â”€ event.md
â”‚   â”œâ”€â”€ style.md
â”‚   â”œâ”€â”€ layout.md
â”‚   â”œâ”€â”€ reactive.md
â”‚   â””â”€â”€ browser.md
â”‚
â”œâ”€â”€ compiler/
â”‚   â”œâ”€â”€ Cargo.toml
â”‚   â””â”€â”€ src/
â”‚       â”œâ”€â”€ lib.rs
â”‚       â”œâ”€â”€ lexer/
â”‚       â”œâ”€â”€ parser/
â”‚       â”œâ”€â”€ ast/
â”‚       â”œâ”€â”€ semantic/
â”‚       â”œâ”€â”€ ir/
â”‚       â”œâ”€â”€ codegen/
â”‚       â””â”€â”€ diagnostics/
â”‚
â”œâ”€â”€ runtime/
â”‚   â”œâ”€â”€ Cargo.toml
â”‚   â””â”€â”€ src/
â”‚       â”œâ”€â”€ lib.rs
â”‚       â”œâ”€â”€ element/
â”‚       â”‚   â”œâ”€â”€ registry.rs
â”‚       â”‚   â”œâ”€â”€ button/
â”‚       â”‚   â”œâ”€â”€ input/
â”‚       â”‚   â”œâ”€â”€ text/
â”‚       â”‚   â”œâ”€â”€ image/
â”‚       â”‚   â”œâ”€â”€ container/
â”‚       â”‚   â”œâ”€â”€ row/
â”‚       â”‚   â”œâ”€â”€ column/
â”‚       â”‚   â”œâ”€â”€ stack/
â”‚       â”‚   â”œâ”€â”€ grid/
â”‚       â”‚   â”œâ”€â”€ list/
â”‚       â”‚   â”œâ”€â”€ table/
â”‚       â”‚   â”œâ”€â”€ dialog/
â”‚       â”‚   â”œâ”€â”€ popover/
â”‚       â”‚   â”œâ”€â”€ tooltip/
â”‚       â”‚   â”œâ”€â”€ tabs/
â”‚       â”‚   â”œâ”€â”€ sidebar/
â”‚       â”‚   â”œâ”€â”€ toolbar/
â”‚       â”‚   â”œâ”€â”€ canvas/
â”‚       â”‚   â”œâ”€â”€ editor/
â”‚       â”‚   â””â”€â”€ code_editor/
â”‚       â”‚
â”‚       â”œâ”€â”€ component/
â”‚       â”‚   â”œâ”€â”€ instance.rs
â”‚       â”‚   â”œâ”€â”€ tree.rs
â”‚       â”‚   â”œâ”€â”€ lifecycle.rs
â”‚       â”‚   â”œâ”€â”€ input.rs
â”‚       â”‚   â”œâ”€â”€ output.rs
â”‚       â”‚   â”œâ”€â”€ context.rs
â”‚       â”‚   â””â”€â”€ registry.rs
â”‚       â”‚
â”‚       â”œâ”€â”€ state/
â”‚       â”‚   â”œâ”€â”€ local/
â”‚       â”‚   â”œâ”€â”€ shared/
â”‚       â”‚   â”œâ”€â”€ global/
â”‚       â”‚   â”œâ”€â”€ derived/
â”‚       â”‚   â”œâ”€â”€ persistent/
â”‚       â”‚   â”œâ”€â”€ dependency/
â”‚       â”‚   â”œâ”€â”€ transaction/
â”‚       â”‚   â””â”€â”€ scheduler/
â”‚       â”‚
â”‚       â”œâ”€â”€ event/
â”‚       â”‚   â”œâ”€â”€ pointer/
â”‚       â”‚   â”œâ”€â”€ keyboard/
â”‚       â”‚   â”œâ”€â”€ input/
â”‚       â”‚   â”œâ”€â”€ focus/
â”‚       â”‚   â”œâ”€â”€ form/
â”‚       â”‚   â”œâ”€â”€ drag/
â”‚       â”‚   â”œâ”€â”€ clipboard/
â”‚       â”‚   â”œâ”€â”€ media/
â”‚       â”‚   â”œâ”€â”€ lifecycle/
â”‚       â”‚   â”œâ”€â”€ animation/
â”‚       â”‚   â”œâ”€â”€ application/
â”‚       â”‚   â””â”€â”€ custom/
â”‚       â”‚
â”‚       â”œâ”€â”€ style/
â”‚       â”‚   â”œâ”€â”€ property/
â”‚       â”‚   â”œâ”€â”€ state/
â”‚       â”‚   â”œâ”€â”€ theme/
â”‚       â”‚   â”œâ”€â”€ responsive/
â”‚       â”‚   â””â”€â”€ resolver.rs
â”‚       â”‚
â”‚       â”œâ”€â”€ layout/
â”‚       â”‚   â”œâ”€â”€ node.rs
â”‚       â”‚   â”œâ”€â”€ constraints.rs
â”‚       â”‚   â”œâ”€â”€ measurement.rs
â”‚       â”‚   â”œâ”€â”€ alignment.rs
â”‚       â”‚   â”œâ”€â”€ sizing.rs
â”‚       â”‚   â”œâ”€â”€ spacing.rs
â”‚       â”‚   â”œâ”€â”€ overflow.rs
â”‚       â”‚   â”œâ”€â”€ responsive.rs
â”‚       â”‚   â”œâ”€â”€ scroll.rs
â”‚       â”‚   â”œâ”€â”€ resize.rs
â”‚       â”‚   â””â”€â”€ engine.rs
â”‚       â”‚
â”‚       â”œâ”€â”€ reactive/
â”‚       â”‚   â”œâ”€â”€ signal.rs
â”‚       â”‚   â”œâ”€â”€ dependency.rs
â”‚       â”‚   â”œâ”€â”€ effect.rs
â”‚       â”‚   â”œâ”€â”€ watcher.rs
â”‚       â”‚   â”œâ”€â”€ scheduler.rs
â”‚       â”‚   â”œâ”€â”€ batch.rs
â”‚       â”‚   â”œâ”€â”€ update.rs
â”‚       â”‚   â””â”€â”€ invalidation.rs
â”‚       â”‚
â”‚       â”œâ”€â”€ animation/
â”‚       â”‚   â”œâ”€â”€ animation.rs
â”‚       â”‚   â”œâ”€â”€ transition.rs
â”‚       â”‚   â”œâ”€â”€ timeline.rs
â”‚       â”‚   â””â”€â”€ keyframe.rs
â”‚       â”‚
â”‚       â”œâ”€â”€ render/
â”‚       â”‚   â”œâ”€â”€ tree.rs
â”‚       â”‚   â”œâ”€â”€ node.rs
â”‚       â”‚   â”œâ”€â”€ renderer.rs
â”‚       â”‚   â”œâ”€â”€ mount.rs
â”‚       â”‚   â”œâ”€â”€ patch.rs
â”‚       â”‚   â””â”€â”€ diff.rs
â”‚       â”‚
â”‚       â”œâ”€â”€ browser/
â”‚       â”‚   â”œâ”€â”€ fetch/
â”‚       â”‚   â”œâ”€â”€ websocket/
â”‚       â”‚   â”œâ”€â”€ storage/
â”‚       â”‚   â”œâ”€â”€ clipboard/
â”‚       â”‚   â”œâ”€â”€ file/
â”‚       â”‚   â”œâ”€â”€ url/
â”‚       â”‚   â”œâ”€â”€ history/
â”‚       â”‚   â”œâ”€â”€ media/
â”‚       â”‚   â”œâ”€â”€ canvas/
â”‚       â”‚   â”œâ”€â”€ worker/
â”‚       â”‚   â””â”€â”€ notification/
â”‚       â”‚
â”‚       â”œâ”€â”€ scheduler/
â”‚       â””â”€â”€ error/
â”‚
â”œâ”€â”€ standard/
â”‚   â”œâ”€â”€ Cargo.toml
â”‚   â””â”€â”€ src/
â”‚
â”œâ”€â”€ cli/
â”‚   â”œâ”€â”€ Cargo.toml
â”‚   â””â”€â”€ src/
â”‚       â”œâ”€â”€ main.rs
â”‚       â”œâ”€â”€ command/
â”‚       â”œâ”€â”€ config/
â”‚       â””â”€â”€ output/
â”‚
â”œâ”€â”€ tests/
â”‚   â”œâ”€â”€ lexer/
â”‚   â”œâ”€â”€ parser/
â”‚   â”œâ”€â”€ semantic/
â”‚   â”œâ”€â”€ ir/
â”‚   â”œâ”€â”€ codegen/
â”‚   â”œâ”€â”€ element/
â”‚   â”œâ”€â”€ state/
â”‚   â”œâ”€â”€ event/
â”‚   â”œâ”€â”€ component/
â”‚   â”œâ”€â”€ style/
â”‚   â”œâ”€â”€ layout/
â”‚   â”œâ”€â”€ reactive/
â”‚   â”œâ”€â”€ browser/
â”‚   â””â”€â”€ integration/
â”‚
â”œâ”€â”€ fixtures/
â”‚   â”œâ”€â”€ valid/
â”‚   â””â”€â”€ invalid/
â”‚
â””â”€â”€ examples/
    â”œâ”€â”€ counter/
    â”œâ”€â”€ todo/
    â”œâ”€â”€ form/
    â”œâ”€â”€ dashboard/
    â”œâ”€â”€ data_table/
    â”œâ”€â”€ editor/
    â””â”€â”€ application/

---

63. Hubungan Antarbagian

Arsitektur secara konseptual:

                         SOURCE
                           â”‚
                           â–¼
                        COMPILER
                           â”‚
             â”Œâ”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”´â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”
             â”‚                           â”‚
           Parser                    Semantic
             â”‚                           â”‚
             â–¼                           â–¼
            AST                           IR
                                         â”‚
                                         â–¼
                                       Codegen
                                         â”‚
                                         â–¼
                                      RUNTIME
                                         â”‚
       â”Œâ”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”¬â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”¬â”€â”€â”€â”´â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”
       â”‚              â”‚              â”‚                  â”‚
       â–¼              â–¼              â–¼                  â–¼
    Element         State          Event              Style
       â”‚              â”‚              â”‚                  â”‚
       â””â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”´â”€â”€â”€â”€â”€â”€â”¬â”€â”€â”€â”€â”€â”€â”€â”´â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”˜
                             â–¼
                         Reactive
                             â”‚
                             â–¼
                           Layout
                             â”‚
                             â–¼
                           Render
                             â”‚
                             â–¼
                          Browser

---

64. Prinsip Paling Penting

Struktur ini mengikuti beberapa aturan utama:

1. Satu domain memiliki module sendiri

state/
event/
element/
component/
layout/
reactive/

2. Element tidak dikumpulkan dalam satu file besar

element/button/
element/input/
element/dialog/

3. Event dipisahkan berdasarkan kategori

event/pointer/
event/keyboard/
event/input/
event/focus/

4. State dipisahkan berdasarkan scope dan mekanisme

state/local/
state/shared/
state/global/
state/derived/
state/persistent/

5. Compiler dipisahkan berdasarkan pipeline

lexer
parser
ast
semantic
ir
codegen

6. Runtime tidak menjadi satu module besar

element
component
state
event
style
layout
reactive
render
browser

7. HTML/CSS/JavaScript bukan model utama bahasa

Ketiganya merupakan detail target web/runtime.

8. Fitur kompleks boleh dipecah lebih jauh

Tetapi fitur sederhana tidak perlu dibuat terlalu kompleks.

9. Setiap perubahan penting harus diuji

Terutama:

Syntax
AST
Semantic
IR
Codegen
Runtime
Reactive behavior

10. Tidak ada architectural rewrite tanpa alasan yang jelas

Struktur dapat berkembang, tetapi perubahan besar harus dilakukan berdasarkan kebutuhan nyata dan tetap menjaga compatibility fitur yang sudah ada.

---

65. Kesimpulan

Struktur proyek ini dirancang agar bahasa dapat berkembang dari compiler sederhana menjadi platform untuk membangun aplikasi web kompleks seperti:

Website
Dashboard
SaaS
Admin Panel
Browser IDE
Figma-like Application
Document Editor
Productivity Application
Data Application
Browser-based Desktop Application

Tanpa membuat source code compiler maupun runtime berubah menjadi satu kumpulan file besar.

Fondasi arsitekturnya adalah:

Language
   â†“
Component
   â†“
Element
   â†“
State + Event
   â†“
Reactive System
   â†“
Layout + Style
   â†“
Render
   â†“
Browser

Dengan struktur Rust yang mengikuti domain tersebut, setiap bagian dapat dikembangkan, diuji, dan dipelihara secara independen tanpa kehilangan hubungan antar-subsystem.
