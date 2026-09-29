Struktur Proyek Bahasa Web Application

1. Tujuan Dokumen

Dokumen ini mendefinisikan struktur proyek, pembagian tanggung jawab module, aturan modularisasi, hubungan antara compiler dan runtime, serta organisasi source code untuk bahasa pemrograman yang dirancang khusus untuk membangun aplikasi web modern.

Bahasa ini tidak menjadikan HTML, CSS, dan JavaScript sebagai tiga konsep utama yang harus dipelajari pengguna secara terpisah.

Model utama bahasa:

Application
    ↓
Component
    ↓
Element
    ↓
State
    ↓
Event
    ↓
Reactive Runtime
    ↓
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
├── element.rs      # semua element
├── event.rs        # semua event
├── state.rs        # semua state
└── runtime.rs      # seluruh runtime

Struktur tersebut akan cepat menjadi sulit dipelihara.

Struktur yang diinginkan:

runtime/src/
├── element/
├── event/
├── state/
├── style/
├── layout/
├── component/
├── reactive/
└── browser/

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
├── local/
├── shared/
├── global/
├── derived/
└── persistent/

Hal ini membuat lokasi sebuah fitur dapat diprediksi.

Jika developer ingin mencari implementasi "DerivedState", lokasi utamanya harus jelas:

runtime/src/state/derived/

---

4. Struktur Repository Utama

Struktur awal repository:

project/
│
├── Cargo.toml
├── Cargo.lock
├── README.md
├── LICENSE
│
├── docs/
│   ├── prd.md
│   ├── struktur.md
│   ├── syntax.md
│   ├── architecture.md
│   ├── runtime.md
│   ├── compiler.md
│   ├── state.md
│   ├── event.md
│   ├── element.md
│   ├── component.md
│   ├── style.md
│   └── browser.md
│
├── compiler/
│   ├── Cargo.toml
│   └── src/
│
├── runtime/
│   ├── Cargo.toml
│   └── src/
│
├── standard/
│   ├── Cargo.toml
│   └── src/
│
├── cli/
│   ├── Cargo.toml
│   └── src/
│
├── tests/
│   ├── parser/
│   ├── semantic/
│   ├── compiler/
│   ├── element/
│   ├── state/
│   ├── event/
│   ├── component/
│   ├── style/
│   ├── layout/
│   ├── reactive/
│   ├── browser/
│   └── integration/
│
├── examples/
│   ├── counter/
│   ├── form/
│   ├── dashboard/
│   ├── todo/
│   └── editor/
│
└── fixtures/
    ├── valid/
    └── invalid/

«"standard/" hanya digunakan jika library standar memang diperlukan. Runtime tidak bergantung pada keberadaan standard library untuk konsep dasar UI.»

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

              ┌────────────┐
              │    CLI     │
              └─────┬──────┘
                    │
          ┌─────────┴─────────┐
          ↓                   ↓
     ┌──────────┐       ┌──────────┐
     │ Compiler │       │ Runtime  │
     └──────────┘       └──────────┘

Compiler tidak boleh bergantung pada implementasi UI runtime secara langsung hanya untuk melakukan parsing.

---

6. Compiler

Compiler bertanggung jawab mengubah source language menjadi bentuk yang dapat dijalankan oleh target runtime.

Pipeline utama:

Source
  ↓
Lexer
  ↓
Token
  ↓
Parser
  ↓
AST
  ↓
Semantic Analysis
  ↓
Intermediate Representation
  ↓
Code Generation
  ↓
Web Output

Struktur:

compiler/
└── src/
    ├── lib.rs
    │
    ├── lexer/
    │   ├── mod.rs
    │   ├── scanner.rs
    │   ├── token.rs
    │   ├── keyword.rs
    │   ├── operator.rs
    │   └── literal.rs
    │
    ├── parser/
    │   ├── mod.rs
    │   ├── app.rs
    │   ├── component.rs
    │   ├── element.rs
    │   ├── state.rs
    │   ├── event.rs
    │   ├── style.rs
    │   ├── expression.rs
    │   ├── statement.rs
    │   └── function.rs
    │
    ├── ast/
    │   ├── mod.rs
    │   ├── app.rs
    │   ├── component.rs
    │   ├── element.rs
    │   ├── state.rs
    │   ├── event.rs
    │   ├── style.rs
    │   ├── expression.rs
    │   ├── statement.rs
    │   └── function.rs
    │
    ├── semantic/
    │   ├── mod.rs
    │   ├── scope.rs
    │   ├── symbol.rs
    │   ├── type_check.rs
    │   ├── component.rs
    │   ├── element.rs
    │   ├── state.rs
    │   ├── event.rs
    │   ├── expression.rs
    │   └── diagnostics.rs
    │
    ├── ir/
    │   ├── mod.rs
    │   ├── module.rs
    │   ├── component.rs
    │   ├── element.rs
    │   ├── state.rs
    │   ├── event.rs
    │   ├── expression.rs
    │   └── verifier.rs
    │
    ├── codegen/
    │   ├── mod.rs
    │   ├── elements/
    │   ├── components/
    │   ├── state/
    │   ├── events/
    │   ├── style/
    │   ├── layout/
    │   └── web/
    │
    └── diagnostics/
        ├── mod.rs
        ├── error.rs
        ├── warning.rs
        └── span.rs

---

7. Lexer

Lexer hanya bertanggung jawab mengubah karakter menjadi token.

Struktur:

lexer/
├── mod.rs
├── scanner.rs
├── token.rs
├── keyword.rs
├── operator.rs
└── literal.rs

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
├── mod.rs
├── app.rs
├── component.rs
├── element.rs
├── state.rs
├── event.rs
├── style.rs
├── expression.rs
├── statement.rs
└── function.rs

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
 ├── Component
 │    ├── Input
 │    ├── State
 │    ├── Element
 │    └── Event

Struktur AST:

ast/
├── app.rs
├── component.rs
├── element.rs
├── state.rs
├── event.rs
├── style.rs
├── expression.rs
├── statement.rs
└── function.rs

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
├── scope.rs
├── symbol.rs
├── type_check.rs
├── component.rs
├── element.rs
├── state.rs
├── event.rs
├── expression.rs
└── diagnostics.rs

---

11. IR

IR digunakan sebagai bentuk perantara antara semantic analysis dan code generation.

Struktur:

ir/
├── mod.rs
├── module.rs
├── component.rs
├── element.rs
├── state.rs
├── event.rs
├── expression.rs
└── verifier.rs

IR harus memiliki verifier.

Aturan:

Perubahan IR
      ↓
Update verifier
      ↓
Regression test
      ↓
Codegen test

IR tidak boleh berubah tanpa memperbarui verifier.

---

12. Code Generation

Codegen menerjemahkan IR menjadi target.

Struktur:

codegen/
├── mod.rs
│
├── elements/
│   ├── mod.rs
│   ├── button.rs
│   ├── input.rs
│   ├── text.rs
│   └── ...
│
├── components/
│   ├── mod.rs
│   └── component.rs
│
├── state/
│   ├── mod.rs
│   ├── local.rs
│   ├── shared.rs
│   ├── global.rs
│   └── derived.rs
│
├── events/
│   ├── mod.rs
│   ├── pointer.rs
│   ├── keyboard.rs
│   ├── input.rs
│   └── custom.rs
│
├── style/
│   ├── mod.rs
│   └── compiler.rs
│
├── layout/
│   ├── mod.rs
│   └── compiler.rs
│
└── web/
    ├── mod.rs
    ├── html.rs
    ├── css.rs
    ├── javascript.rs
    ├── wasm.rs
    └── bundle.rs

HTML/CSS/JavaScript di sini adalah target implementasi, bukan konsep utama bahasa.

---

13. Runtime

Runtime adalah sistem yang menjalankan hasil compiler.

Struktur utama:

runtime/
└── src/
    ├── lib.rs
    │
    ├── element/
    ├── component/
    ├── state/
    ├── event/
    ├── style/
    ├── layout/
    ├── reactive/
    ├── animation/
    ├── browser/
    ├── render/
    ├── scheduler/
    └── error/

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
├── mod.rs
│
├── button/
│   ├── mod.rs
│   ├── state.rs
│   ├── event.rs
│   ├── style.rs
│   └── render.rs
│
├── input/
│   ├── mod.rs
│   ├── state.rs
│   ├── event.rs
│   ├── style.rs
│   └── render.rs
│
├── text/
│   ├── mod.rs
│   ├── style.rs
│   └── render.rs
│
├── image/
│   ├── mod.rs
│   ├── state.rs
│   ├── style.rs
│   └── render.rs
│
├── container/
│   ├── mod.rs
│   ├── layout.rs
│   ├── style.rs
│   └── render.rs
│
├── row/
├── column/
├── stack/
├── grid/
├── panel/
├── splitter/
├── scroll/
│
├── list/
├── table/
├── tree/
├── data_grid/
│
├── dialog/
├── popover/
├── tooltip/
├── tabs/
├── sidebar/
├── toolbar/
│
├── canvas/
├── editor/
├── code_editor/
└── viewport/

---

15. Struktur Internal Element

Element yang sederhana tidak harus memiliki seluruh file.

Contoh sederhana:

text/
├── mod.rs
├── style.rs
└── render.rs

Element kompleks:

data_grid/
├── mod.rs
├── state.rs
├── event.rs
├── selection.rs
├── sorting.rs
├── filtering.rs
├── virtualization.rs
├── column.rs
├── row.rs
├── style.rs
├── layout.rs
└── render.rs

Prinsipnya:

«Kompleksitas element menentukan jumlah submodule.»

Jangan membuat struktur kosong hanya demi mengikuti pola.

---

16. Element Registry

Runtime membutuhkan registry untuk mengenali primitive element.

Contoh:

element/
├── mod.rs
├── registry.rs
├── button/
├── input/
└── ...

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
├── mod.rs
│
├── local/
│   ├── mod.rs
│   ├── state.rs
│   └── storage.rs
│
├── shared/
│   ├── mod.rs
│   ├── state.rs
│   └── scope.rs
│
├── global/
│   ├── mod.rs
│   └── state.rs
│
├── derived/
│   ├── mod.rs
│   ├── state.rs
│   └── dependency.rs
│
├── persistent/
│   ├── mod.rs
│   ├── state.rs
│   └── storage.rs
│
├── dependency/
│   ├── mod.rs
│   ├── graph.rs
│   ├── tracker.rs
│   └── watcher.rs
│
├── transaction/
│   ├── mod.rs
│   └── transaction.rs
│
└── scheduler/
    ├── mod.rs
    └── scheduler.rs

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
  ↓
Derived State
  ↓
Component
  ↓
Element

Contoh:

count
  ↓
doubleCount
  ↓
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
├── mod.rs
│
├── pointer/
│   ├── mod.rs
│   ├── click.rs
│   ├── double_click.rs
│   ├── mouse.rs
│   └── pointer.rs
│
├── keyboard/
│   ├── mod.rs
│   ├── key_down.rs
│   ├── key_up.rs
│   └── key_press.rs
│
├── input/
│   ├── mod.rs
│   ├── input.rs
│   ├── change.rs
│   └── composition.rs
│
├── focus/
│   ├── mod.rs
│   ├── focus.rs
│   └── blur.rs
│
├── form/
│   ├── mod.rs
│   ├── submit.rs
│   ├── reset.rs
│   └── validation.rs
│
├── drag/
│   ├── mod.rs
│   ├── drag_start.rs
│   ├── drag.rs
│   ├── drag_over.rs
│   ├── drop.rs
│   └── drag_end.rs
│
├── clipboard/
│   ├── mod.rs
│   ├── copy.rs
│   ├── cut.rs
│   └── paste.rs
│
├── media/
│   ├── mod.rs
│   ├── play.rs
│   ├── pause.rs
│   ├── ended.rs
│   └── time_update.rs
│
├── lifecycle/
│   ├── mod.rs
│   ├── create.rs
│   ├── mount.rs
│   ├── update.rs
│   └── destroy.rs
│
├── animation/
│   ├── mod.rs
│   ├── animation.rs
│   └── transition.rs
│
├── application/
│   ├── mod.rs
│   ├── state_change.rs
│   ├── route_change.rs
│   └── component_change.rs
│
└── custom/
    ├── mod.rs
    ├── event.rs
    └── emitter.rs

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
├── mod.rs
├── instance.rs
├── tree.rs
├── lifecycle.rs
├── input.rs
├── output.rs
├── context.rs
└── registry.rs

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
  ↓
Mount
  ↓
Update
  ↓
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
├── mod.rs
├── property/
│   ├── mod.rs
│   ├── color.rs
│   ├── size.rs
│   ├── spacing.rs
│   ├── typography.rs
│   ├── border.rs
│   └── shadow.rs
│
├── state/
│   ├── mod.rs
│   ├── normal.rs
│   ├── hover.rs
│   ├── pressed.rs
│   ├── focused.rs
│   ├── disabled.rs
│   └── selected.rs
│
├── theme/
│   ├── mod.rs
│   ├── theme.rs
│   └── token.rs
│
├── responsive/
│   ├── mod.rs
│   └── breakpoint.rs
│
└── resolver.rs

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
├── mod.rs
├── node.rs
├── constraints.rs
├── measurement.rs
├── alignment.rs
├── sizing.rs
├── spacing.rs
├── overflow.rs
├── responsive.rs
├── scroll.rs
├── resize.rs
└── engine.rs

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
   ↓
layout

---

30. Reactive Runtime

Reactive runtime adalah penghubung antara state, event dan UI.

Struktur:

reactive/
├── mod.rs
├── signal.rs
├── dependency.rs
├── effect.rs
├── watcher.rs
├── scheduler.rs
├── batch.rs
├── update.rs
└── invalidation.rs

Model:

Event
  ↓
State Change
  ↓
Dependency Tracking
  ↓
Invalidation
  ↓
Scheduler
  ↓
Minimal UI Update

---

31. Scheduler

Scheduler bertugas mengatur update reactive.

Contoh:

State A berubah
State B berubah
State C berubah
        ↓
     Batch
        ↓
Dependency calculation
        ↓
UI update

Tujuannya menghindari update berulang yang tidak diperlukan.

---

32. Invalidation

Runtime harus dapat menentukan bagian yang perlu diperbarui.

Contoh:

state count
   ↓
Text count

Ketika "count" berubah:

count invalid
   ↓
Text invalid

Element lain yang tidak bergantung pada "count" tidak perlu dirender ulang.

---

33. Animation

Animation subsystem:

animation/
├── mod.rs
├── animation.rs
├── transition.rs
├── timeline.rs
├── keyframe.rs
└── scheduler.rs

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
├── mod.rs
├── fetch/
├── websocket/
├── storage/
├── clipboard/
├── file/
├── url/
├── history/
├── media/
├── canvas/
├── worker/
└── notification/

Contoh:

browser.fetch(...)

atau API tingkat bahasa yang sesuai dengan desain akhir.

Browser API tidak boleh membuat developer harus menggunakan DOM secara langsung untuk kebutuhan UI normal.

---

35. Render System

Rendering dipisahkan dari element definition.

render/
├── mod.rs
├── tree.rs
├── node.rs
├── renderer.rs
├── mount.rs
├── patch.rs
└── diff.rs

Element mendefinisikan perilakunya.

Render system menentukan bagaimana tree tersebut diwujudkan pada target.

Model:

Element
   ↓
Render Node
   ↓
Render Tree
   ↓
Renderer
   ↓
Browser

---

36. DOM Bukan Model Utama

Bahasa tidak mendesain aplikasi berdasarkan:

DOM
 ↓
HTML
 ↓
JavaScript

Model utama:

Application
 ↓
Component
 ↓
Element
 ↓
State
 ↓
Event
 ↓
Reactive Runtime
 ↓
Renderer
 ↓
Browser

DOM dapat digunakan sebagai implementasi target browser.

---

37. CLI

CLI menjadi interface pengguna untuk compiler.

Struktur:

cli/
└── src/
    ├── main.rs
    ├── command/
    │   ├── mod.rs
    │   ├── build.rs
    │   ├── dev.rs
    │   ├── run.rs
    │   ├── check.rs
    │   └── format.rs
    │
    ├── config/
    │   └── mod.rs
    │
    └── output/
        ├── mod.rs
        └── diagnostics.rs

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
├── lexer/
├── parser/
├── semantic/
├── ir/
├── codegen/
├── element/
├── state/
├── event/
├── component/
├── style/
├── layout/
├── reactive/
├── browser/
└── integration/

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
 ↓
handler
 ↓
state mutation
 ↓
reactive update

Contoh:

click
 ↓
count += 1
 ↓
Text update

---

44. Element Test

Setiap primitive element yang kompleks dapat memiliki test sendiri.

Contoh:

tests/element/button/
├── creation.rs
├── state.rs
├── event.rs
├── style.rs
└── interaction.rs

Untuk element sederhana dapat digabung secukupnya.

---

45. Integration Test

Integration test menguji pipeline penuh.

Source
  ↓
Compiler
  ↓
Generated Output
  ↓
Runtime
  ↓
Browser
  ↓
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
 ↓
compile
 ↓
build
 ↓
serve
 ↓
execute
 ↓
interaction
 ↓
verify output

Jangan menganggap:

cargo test

saja sudah cukup untuk memastikan compiler dan runtime bekerja.

---

47. Fixtures

Program contoh untuk testing disimpan pada:

fixtures/
├── valid/
└── invalid/

Contoh:

fixtures/
├── valid/
│   ├── basic_app/
│   ├── counter/
│   ├── state/
│   ├── event/
│   ├── component/
│   └── layout/
│
└── invalid/
    ├── unknown_element/
    ├── unknown_event/
    ├── invalid_state/
    └── type_error/

---

48. Examples

Examples adalah program yang dapat dibaca developer.

examples/
├── counter/
├── todo/
├── form/
├── dashboard/
├── data_table/
├── editor/
└── application/

Examples bukan pengganti integration test.

---

49. Dokumentasi

Dokumentasi dipisahkan berdasarkan domain.

docs/
├── prd.md
├── struktur.md
├── syntax.md
├── architecture.md
├── compiler.md
├── runtime.md
├── element.md
├── component.md
├── state.md
├── event.md
├── style.md
├── layout.md
├── reactive.md
└── browser.md

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
   ↓
AST
   ↓
Semantic
   ↓
IR
   ↓
Codegen

Runtime:

Element
   ↓
Layout
   ↓
Render

State
   ↓
Reactive
   ↓
Render

Event
   ↓
State
   ↓
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
├── mod.rs
├── state.rs
├── event.rs
├── style.rs
└── render.rs

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
├── mod.rs
└── render.rs

sudah cukup.

Tidak perlu:

text/
├── mod.rs
├── state.rs
├── state_manager.rs
├── event.rs
├── event_manager.rs
├── renderer.rs
├── render_pipeline.rs
├── abstraction.rs
└── factory.rs

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
      ↓
Lexer
      ↓
Parser
      ↓
AST
      ↓
Semantic
      ↓
IR
      ↓
Codegen
      ↓
Runtime
      ↓
Tests
      ↓
Documentation

---

59. Aturan Perubahan IR

IR merupakan kontrak internal compiler.

Jika IR berubah:

IR
 ↓
Verifier update
 ↓
Codegen update
 ↓
Regression test

Verifier harus selalu mencerminkan aturan IR terbaru.

---

60. Aturan Bug Compiler

Setiap bug compiler harus memiliki regression test.

Alur:

Bug ditemukan
     ↓
Buat test yang gagal
     ↓
Perbaiki compiler
     ↓
Test berhasil
     ↓
Test tetap dipertahankan

Jangan hanya memperbaiki source code tanpa test.

---

61. Aturan Runtime

Runtime harus diuji pada level:

Unit
 ↓
Subsystem
 ↓
Integration
 ↓
End-to-End

Contoh:

Button
 ↓
click
 ↓
state change
 ↓
reactive update
 ↓
render

harus dapat diuji sebagai satu alur.

---

62. Struktur Akhir yang Direkomendasikan

Struktur repository secara keseluruhan:

project/
│
├── Cargo.toml
├── Cargo.lock
├── README.md
├── LICENSE
│
├── docs/
│   ├── prd.md
│   ├── struktur.md
│   ├── syntax.md
│   ├── architecture.md
│   ├── compiler.md
│   ├── runtime.md
│   ├── element.md
│   ├── component.md
│   ├── state.md
│   ├── event.md
│   ├── style.md
│   ├── layout.md
│   ├── reactive.md
│   └── browser.md
│
├── compiler/
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── lexer/
│       ├── parser/
│       ├── ast/
│       ├── semantic/
│       ├── ir/
│       ├── codegen/
│       └── diagnostics/
│
├── runtime/
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── element/
│       │   ├── registry.rs
│       │   ├── button/
│       │   ├── input/
│       │   ├── text/
│       │   ├── image/
│       │   ├── container/
│       │   ├── row/
│       │   ├── column/
│       │   ├── stack/
│       │   ├── grid/
│       │   ├── list/
│       │   ├── table/
│       │   ├── dialog/
│       │   ├── popover/
│       │   ├── tooltip/
│       │   ├── tabs/
│       │   ├── sidebar/
│       │   ├── toolbar/
│       │   ├── canvas/
│       │   ├── editor/
│       │   └── code_editor/
│       │
│       ├── component/
│       │   ├── instance.rs
│       │   ├── tree.rs
│       │   ├── lifecycle.rs
│       │   ├── input.rs
│       │   ├── output.rs
│       │   ├── context.rs
│       │   └── registry.rs
│       │
│       ├── state/
│       │   ├── local/
│       │   ├── shared/
│       │   ├── global/
│       │   ├── derived/
│       │   ├── persistent/
│       │   ├── dependency/
│       │   ├── transaction/
│       │   └── scheduler/
│       │
│       ├── event/
│       │   ├── pointer/
│       │   ├── keyboard/
│       │   ├── input/
│       │   ├── focus/
│       │   ├── form/
│       │   ├── drag/
│       │   ├── clipboard/
│       │   ├── media/
│       │   ├── lifecycle/
│       │   ├── animation/
│       │   ├── application/
│       │   └── custom/
│       │
│       ├── style/
│       │   ├── property/
│       │   ├── state/
│       │   ├── theme/
│       │   ├── responsive/
│       │   └── resolver.rs
│       │
│       ├── layout/
│       │   ├── node.rs
│       │   ├── constraints.rs
│       │   ├── measurement.rs
│       │   ├── alignment.rs
│       │   ├── sizing.rs
│       │   ├── spacing.rs
│       │   ├── overflow.rs
│       │   ├── responsive.rs
│       │   ├── scroll.rs
│       │   ├── resize.rs
│       │   └── engine.rs
│       │
│       ├── reactive/
│       │   ├── signal.rs
│       │   ├── dependency.rs
│       │   ├── effect.rs
│       │   ├── watcher.rs
│       │   ├── scheduler.rs
│       │   ├── batch.rs
│       │   ├── update.rs
│       │   └── invalidation.rs
│       │
│       ├── animation/
│       │   ├── animation.rs
│       │   ├── transition.rs
│       │   ├── timeline.rs
│       │   └── keyframe.rs
│       │
│       ├── render/
│       │   ├── tree.rs
│       │   ├── node.rs
│       │   ├── renderer.rs
│       │   ├── mount.rs
│       │   ├── patch.rs
│       │   └── diff.rs
│       │
│       ├── browser/
│       │   ├── fetch/
│       │   ├── websocket/
│       │   ├── storage/
│       │   ├── clipboard/
│       │   ├── file/
│       │   ├── url/
│       │   ├── history/
│       │   ├── media/
│       │   ├── canvas/
│       │   ├── worker/
│       │   └── notification/
│       │
│       ├── scheduler/
│       └── error/
│
├── standard/
│   ├── Cargo.toml
│   └── src/
│
├── cli/
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs
│       ├── command/
│       ├── config/
│       └── output/
│
├── tests/
│   ├── lexer/
│   ├── parser/
│   ├── semantic/
│   ├── ir/
│   ├── codegen/
│   ├── element/
│   ├── state/
│   ├── event/
│   ├── component/
│   ├── style/
│   ├── layout/
│   ├── reactive/
│   ├── browser/
│   └── integration/
│
├── fixtures/
│   ├── valid/
│   └── invalid/
│
└── examples/
    ├── counter/
    ├── todo/
    ├── form/
    ├── dashboard/
    ├── data_table/
    ├── editor/
    └── application/

---

63. Hubungan Antarbagian

Arsitektur secara konseptual:

                         SOURCE
                           │
                           ▼
                        COMPILER
                           │
             ┌─────────────┴─────────────┐
             │                           │
           Parser                    Semantic
             │                           │
             ▼                           ▼
            AST                           IR
                                         │
                                         ▼
                                       Codegen
                                         │
                                         ▼
                                      RUNTIME
                                         │
       ┌──────────────┬──────────────┬───┴──────────────┐
       │              │              │                  │
       ▼              ▼              ▼                  ▼
    Element         State          Event              Style
       │              │              │                  │
       └──────────────┴──────┬───────┴──────────────────┘
                             ▼
                         Reactive
                             │
                             ▼
                           Layout
                             │
                             ▼
                           Render
                             │
                             ▼
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
   ↓
Component
   ↓
Element
   ↓
State + Event
   ↓
Reactive System
   ↓
Layout + Style
   ↓
Render
   ↓
Browser

Dengan struktur Rust yang mengikuti domain tersebut, setiap bagian dapat dikembangkan, diuji, dan dipelihara secara independen tanpa kehilangan hubungan antar-subsystem.