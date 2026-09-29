# PRD — Bahasa Pemrograman UI Application Modern

**Status:** Draft Spesifikasi Produk
**Versi:** 0.1
**Jenis:** UI Application Programming Language
**Target utama:** Web Application
**Fokus:** Element, State, Event, Style, Component, Reactive UI

---

# 1. Ringkasan

Proyek ini adalah bahasa pemrograman yang dirancang khusus untuk membangun **aplikasi berbasis web modern**.

Bahasa ini tidak dirancang sebagai HTML baru, CSS baru, atau framework JavaScript.

Bahasa ini memiliki model pemrograman sendiri yang menyatukan:

```text
Element
State
Event
Style
Component
Function
Reactive System
Application
```

Tujuan utamanya adalah membuat developer dapat membangun aplikasi web kompleks tanpa harus memisahkan pemikiran antara HTML, CSS, JavaScript, dan state management.

Bahasa menyediakan **primitive element bawaan** yang dirancang untuk aplikasi modern.

Primitive tersebut dapat langsung digunakan dan dapat di-*style* ulang.

Developer tidak perlu membuat custom element hanya untuk mendapatkan variasi visual atau perilaku sederhana.

---

# 2. Visi

Membuat bahasa pemrograman yang memungkinkan aplikasi web dibangun menggunakan satu model yang konsisten:

```text
Application
    ↓
Component
    ↓
Element
    ↓
State + Event + Style
```

Bahasa harus terasa seperti bahasa pemrograman untuk membuat aplikasi, bukan seperti template HTML.

---

# 3. Masalah yang Ingin Diselesaikan

Pengembangan web modern biasanya memerlukan banyak konsep dan teknologi:

```text
HTML
CSS
JavaScript
DOM
Event
State Management
Component Framework
Routing
API Client
Build Tool
```

Untuk membuat satu aplikasi sederhana, developer harus memahami bagaimana teknologi-teknologi tersebut saling terhubung.

Masalah yang ingin dikurangi:

* syntax terlalu tersebar;
* terlalu banyak boilerplate;
* state dan UI dipisahkan;
* event menggunakan model berbeda-beda;
* styling membutuhkan sistem terpisah;
* component memiliki pola berbeda dengan element;
* developer harus memahami DOM untuk banyak pekerjaan UI;
* aplikasi besar mudah mengalami ketidakkonsistenan struktur;
* framework berbeda memiliki cara berbeda dalam menangani state dan event.

---

# 4. Solusi

Bahasa menyediakan satu model terpadu.

```text
Element
  │
  ├── Property
  ├── Style
  ├── State Binding
  └── Event
        │
        ↓
     Component
        │
        ↓
    Application
```

Reactive system menghubungkan state dengan element secara otomatis.

```text
Event
  ↓
State berubah
  ↓
Reactive system
  ↓
Element yang terdampak diperbarui
```

---

# 5. Tujuan Produk

## 5.1 Tujuan utama

Bahasa harus:

1. mudah dibaca;
2. mudah dipelajari;
3. memiliki syntax konsisten;
4. memiliki primitive element yang lengkap;
5. memiliki event system yang sangat lengkap;
6. memiliki state system native;
7. memiliki reactive UI native;
8. memiliki style system native;
9. memiliki component system native;
10. mampu membuat aplikasi kompleks;
11. mengurangi boilerplate;
12. tidak bergantung pada HTML sebagai model programming;
13. tidak mengharuskan custom element untuk customization umum.

---

# 6. Non-Goals

Bahasa ini bukan bertujuan untuk:

* menjadi HTML dengan syntax berbeda;
* meniru semua element HTML;
* mempertahankan seluruh API DOM sebagai syntax utama;
* menjadi framework JavaScript;
* menjadi component library;
* menjadi state management library;
* membuat custom element untuk setiap kebutuhan;
* menggantikan semua bahasa pemrograman umum;
* menyediakan semua browser API sejak versi pertama.

---

# 7. Prinsip Fundamental

## 7.1 Language First

Fitur inti harus menjadi bagian dari bahasa/runtime, bukan kumpulan library yang harus dipasang developer.

Contoh:

```text
state
event
component
style
element
```

merupakan konsep fundamental.

---

## 7.2 Consistency First

Syntax baru tidak boleh dibuat jika syntax lama dapat digunakan.

Contoh:

```text
state count = 0
```

harus tetap menjadi pola state.

Tidak boleh muncul beberapa gaya:

```text
state count = 0
let count = 0
signal count = 0
reactive count = 0
```

untuk fungsi yang sama.

---

## 7.3 Primitive First

Bahasa menyediakan primitive element yang cukup lengkap.

Developer mengembangkan aplikasi dengan menggabungkan primitive tersebut.

---

## 7.4 Style Instead of Custom Element

Jika kebutuhan hanya perubahan visual:

```text
Button
```

tetap digunakan.

Developer cukup:

```text
Button "Save" {
    style {
        radius: 10
    }
}
```

Tidak perlu:

```text
CustomSaveButton
```

---

## 7.5 Reactive by Default

State yang digunakan oleh UI bersifat reactive.

Developer tidak perlu memanggil:

```text
render()
update()
refresh()
setState()
```

secara manual untuk kasus normal.

---

# 8. Model Bahasa

Bahasa memiliki konstruksi utama:

```text
app
component
state
derived
event
element
style
fn
if
for
```

Model ini harus menjadi fondasi bahasa.

---

# 9. Syntax Core

## 9.1 Application

```text
app MyApp {
    ...
}
```

Contoh:

```text
app CounterApp {

    state count = 0

    Text count
}
```

`app` adalah root aplikasi.

---

# 10. Component

```text
component Counter {
    ...
}
```

Contoh:

```text
component Counter {

    state count = 0

    Text count

    Button "+" {
        on click {
            count += 1
        }
    }
}
```

---

# 11. Element

Primitive element dipanggil langsung.

```text
Button
```

Dengan text:

```text
Button "Save"
```

Dengan block:

```text
Button "Save" {
    ...
}
```

Element container:

```text
Container {
    Text "Hello"
    Button "Save"
}
```

Bahasa tidak menggunakan:

```text
<div>
<span>
<button>
```

sebagai syntax inti.

---

# 12. Property

Semua property menggunakan satu bentuk:

```text
property: value
```

Contoh:

```text
Button "Save" {
    disabled: false
    loading: true
}
```

Tidak menggunakan variasi:

```text
disabled="false"
:disabled="..."
[disabled]="..."
```

---

# 13. State

Syntax state:

```text
state name = value
```

Contoh:

```text
state count = 0
state username = ""
state loading = false
```

State dapat digunakan langsung oleh UI:

```text
Text count
```

---

# 14. Derived State

State turunan:

```text
derived fullName = firstName + " " + lastName
```

Derived state harus diperbarui ketika dependency berubah.

---

# 15. Event

Semua event menggunakan:

```text
on event {
    ...
}
```

Contoh:

```text
Button "Save" {

    on click {
        save()
    }
}
```

Tidak ada variasi:

```text
onclick
onClick
@click
@onclick
```

---

# 16. Event Object

Event dapat diakses melalui:

```text
event
```

Contoh:

```text
Input {

    on input {
        username = event.value
    }
}
```

Keyboard:

```text
on keydown {

    if event.key == "Enter" {
        submit()
    }
}
```

---

# 17. Custom Event

Custom event:

```text
emit selected(user)
```

Contoh:

```text
component UserCard {

    input user

    Button "Select" {

        on click {
            emit selected(user)
        }
    }
}
```

Parent:

```text
UserCard user: currentUser {

    on selected {
        selectedUser = event.value
    }
}
```

---

# 18. Component Input

Component menerima input:

```text
input user
```

Dengan tipe:

```text
input user: User
```

Dengan default:

```text
input title: String = "Untitled"
```

---

# 19. Component Invocation

Component digunakan seperti element:

```text
UserCard user: currentUser
```

Dengan block:

```text
UserCard user: currentUser {

    on selected {
        ...
    }
}
```

Component dan primitive element memiliki cara pemanggilan yang konsisten.

---

# 20. Function

Function menggunakan:

```text
fn name(args) {
    ...
}
```

Contoh:

```text
fn add(a: Int, b: Int) -> Int {
    return a + b
}
```

Pemanggilan:

```text
result = add(10, 20)
```

---

# 21. Conditional

```text
if condition {
    ...
}
```

Dengan else:

```text
if loggedIn {
    Dashboard
} else {
    Login
}
```

---

# 22. Loop

```text
for item in items {
    ...
}
```

Contoh:

```text
for user in users {
    UserCard user: user
}
```

---

# 23. Binding

One-way property:

```text
Input {
    value: username
}
```

Two-way binding:

```text
Input {
    bind value: username
}
```

Keyword `bind` menjadi satu-satunya syntax two-way binding.

---

# 24. Style

Style menggunakan:

```text
style {
    property: value
}
```

Contoh:

```text
Button "Save" {

    style {
        width: 120
        height: 40
        padding: 10
        radius: 8
    }
}
```

---

# 25. Style State

Style dapat memiliki keadaan:

```text
style {

    normal {
        background: blue
    }

    hover {
        background: darkblue
    }

    pressed {
        scale: 0.98
    }

    disabled {
        opacity: 0.5
    }
}
```

---

# 26. Primitive Elements

Primitive element bukan salinan satu-per-satu dari HTML.

Primitive dirancang berdasarkan kebutuhan aplikasi.

## 26.1 Content

```text
Text
Heading
Icon
Image
Video
Audio
Link
```

## 26.2 Input

```text
Input
Textarea
Select
Checkbox
Radio
Switch
Slider
Date
Time
File
Color
```

## 26.3 Action

```text
Button
Menu
ContextMenu
Command
```

## 26.4 Layout

```text
Container
Row
Column
Stack
Grid
Panel
Splitter
Spacer
Scroll
```

## 26.5 Data

```text
List
Table
Tree
DataGrid
```

## 26.6 Application UI

```text
Dialog
Modal
Popover
Tooltip
Tabs
Navigation
Sidebar
Toolbar
```

## 26.7 Advanced UI

```text
Canvas
Viewport
Editor
CodeEditor
```

Daftar primitive dapat berkembang, tetapi penambahan primitive harus mempertimbangkan konsistensi dan kebutuhan nyata.

---

# 27. Element Capability

Setiap primitive element dapat memiliki:

```text
property
style
event
children
state binding
lifecycle
```

Tidak semua element harus memiliki seluruh capability.

Contoh:

```text
Button
├── properties
├── style
├── events
└── children/text
```

---

# 28. Event System

Event system adalah salah satu fitur utama bahasa.

Event harus mencakup kategori luas.

## Pointer

```text
click
doubleclick
mousedown
mouseup
mousemove
mouseenter
mouseleave
mouseover
mouseout
pointerdown
pointerup
pointermove
pointerenter
pointerleave
wheel
```

## Keyboard

```text
keydown
keyup
keypress
```

## Input

```text
input
change
beforeinput
compositionstart
compositionupdate
compositionend
```

## Focus

```text
focus
blur
focusin
focusout
```

## Form

```text
submit
reset
invalid
validate
```

## Drag

```text
dragstart
drag
dragenter
dragover
dragleave
drop
dragend
```

## Clipboard

```text
copy
cut
paste
```

## Media

```text
play
pause
ended
timeupdate
volumechange
loaded
error
```

## Lifecycle

```text
create
mount
update
destroy
```

## Animation

```text
animationstart
animationend
transitionstart
transitionend
```

## Application

```text
state_change
component_mount
component_unmount
route_change
```

Event system harus dapat diperluas tanpa mengubah syntax `on event`.

---

# 29. State System

State harus mendukung:

```text
local
shared
global
derived
persistent
```

Contoh local:

```text
state count = 0
```

Shared:

```text
shared state user
```

Global:

```text
global state theme = "dark"
```

Derived:

```text
derived fullName = firstName + " " + lastName
```

Persistent:

```text
persistent state settings
```

Detail implementasi dapat berkembang, tetapi syntax harus tetap konsisten.

---

# 30. Reactive System

Reactive dependency harus diketahui runtime/compiler.

Contoh:

```text
state count = 0

Text count
```

Ketika:

```text
count += 1
```

hanya UI yang bergantung pada `count` yang perlu diperbarui.

Runtime tidak boleh melakukan full application rerender untuk perubahan state lokal jika tidak diperlukan.

---

# 31. Lifecycle

Component dan element dapat memiliki lifecycle:

```text
create
mount
update
destroy
```

Contoh:

```text
on mount {
    loadData()
}
```

Lifecycle mengikuti sistem event:

```text
on mount
on update
on destroy
```

Tidak perlu syntax lifecycle khusus lain.

---

# 32. Layout System

Layout merupakan primitive penting.

Bahasa harus menyediakan:

```text
Row
Column
Stack
Grid
Container
Panel
Splitter
Scroll
```

Layout harus mendukung:

* alignment;
* spacing;
* padding;
* sizing;
* min/max size;
* responsive behavior;
* wrapping;
* overflow;
* scrolling;
* nested layout;
* resizing.

---

# 33. Responsive Layout

Bahasa harus menyediakan model responsive.

Contoh target syntax:

```text
Row {

    responsive {
        mobile: Column
        tablet: Row
        desktop: Row
    }
}
```

Syntax responsive harus tetap menjadi bagian dari model style/layout dan tidak memperkenalkan sistem syntax terpisah.

---

# 34. Animation

Animation harus tersedia sebagai capability element/style.

Contoh:

```text
Dialog {

    animation: fade
    duration: 200
}
```

Event animation menggunakan sistem event biasa:

```text
on animationend {
    ...
}
```

---

# 35. Data Rendering

Rendering data:

```text
for user in users {

    UserCard user: user
}
```

Reactive update harus dapat mendeteksi:

* item ditambah;
* item dihapus;
* item diubah;
* item dipindahkan.

Untuk list besar, runtime harus menyediakan mekanisme optimasi/virtualization.

---

# 36. Application State

Application dapat memiliki state global:

```text
app MyApp {

    global state theme = "dark"

    ...
}
```

Component dapat menggunakan state yang tersedia dalam scope-nya.

Scope harus jelas dan dapat diverifikasi compiler.

---

# 37. Scope

Bahasa harus memiliki aturan scope yang eksplisit.

Prioritas:

```text
local component state
↓
parent/shared state
↓
application state
↓
global state
```

Tidak boleh ada implicit global mutation tanpa deklarasi.

---

# 38. Type System

Bahasa harus memiliki tipe dasar.

Minimal:

```text
Int
Float
Bool
String
Null
Array
Object
Function
```

Kemudian:

```text
Map
Set
Date
URL
Binary
```

dapat ditambahkan.

---

# 39. Object

Object menggunakan struktur:

```text
user = {
    name: "Budi",
    age: 20
}
```

Property object:

```text
user.name
```

Object harus dapat digunakan sebagai state.

---

# 40. Array

Array:

```text
users = [
    "Budi",
    "Andi",
    "Siti"
]
```

Loop:

```text
for user in users {
    Text user
}
```

---

# 41. Expression

Expression harus menggunakan model expression umum.

Contoh:

```text
count + 1
```

```text
user.name
```

```text
loading == false
```

```text
items.length
```

```text
firstName + " " + lastName
```

Expression harus dapat digunakan pada:

```text
property
state
derived
condition
function
binding
style
```

---

# 42. Async

Bahasa harus mendukung operasi asynchronous.

Syntax:

```text
async fn loadUsers() {

    users = await api.get("/users")
}
```

`async` dan `await` mengikuti model function yang sama.

---

# 43. Error Handling

Target syntax:

```text
try {
    ...
} catch error {
    ...
}
```

Contoh:

```text
try {
    user = await loadUser()
} catch error {
    errorMessage = error.message
}
```

Error dapat dihubungkan ke state dan UI.

---

# 44. API Integration

API client bukan bagian paling awal dari core language, tetapi harus dirancang agar kompatibel dengan model bahasa.

Target:

```text
api UserAPI {

    get users from "/users"

    get user from "/users/:id"

    post user to "/users"

    delete user from "/users/:id"
}
```

API response dapat menjadi state.

---

# 45. Browser Integration

Runtime harus menyediakan akses terstruktur terhadap:

```text
Fetch
WebSocket
Storage
Clipboard
File
URL
History
Media
Canvas
Web Worker
Notifications
```

Developer tidak perlu menggunakan DOM API mentah untuk operasi UI umum.

---

# 46. DOM

DOM bukan abstraction utama bahasa.

Browser DOM digunakan sebagai implementation target/runtime.

Developer bekerja dengan:

```text
Element
Property
Style
State
Event
Component
```

bukan:

```text
document.querySelector()
element.addEventListener()
element.innerHTML
```

untuk operasi UI normal.

---

# 47. Accessibility

Primitive element harus memiliki accessibility semantics bawaan.

Contoh:

```text
Button "Save"
```

harus diperlakukan sebagai button oleh browser.

Bahasa menyediakan property seperti:

```text
label
description
role
hidden
disabled
```

jika dibutuhkan.

---

# 48. Security

Runtime harus melindungi:

* input;
* output;
* HTML injection;
* script injection;
* unsafe URL;
* storage;
* API request;
* file access.

Default API harus menggunakan pendekatan aman.

---

# 49. Performance

Runtime harus mengutamakan:

* fine-grained reactive update;
* minimal DOM mutation;
* batching;
* event delegation bila sesuai;
* lazy rendering;
* virtualization;
* memoization;
* efficient list update;
* efficient layout update.

Performance harus menjadi bagian dari desain runtime, bukan optimasi terakhir.

---

# 50. Component Composition

Component dapat terdiri dari:

```text
primitive element
component lain
state
derived state
event
function
conditional
loop
style
```

Contoh:

```text
component UserCard {

    input user

    state selected = false

    Card {

        Image user.avatar

        Text user.name

        Button "Select" {

            on click {
                selected = true
                emit selected(user)
            }
        }
    }
}
```

---

# 51. Styling Architecture

Style dapat diberikan langsung:

```text
Button "Save" {
    style {
        radius: 8
    }
}
```

Kemudian sistem style reusable dapat ditambahkan.

Contoh target:

```text
style ButtonPrimary {

    background: blue
    radius: 8
}
```

Penggunaan:

```text
Button "Save" style: ButtonPrimary
```

Detail syntax reusable style akan ditetapkan setelah core style stabil.

---

# 52. Theme

Bahasa harus mendukung theme.

Target:

```text
theme Dark {

    color.primary: "#..."
    color.background: "#..."
    color.text: "#..."
}
```

Element dapat menggunakan token theme:

```text
Button "Save" {

    style {
        background: theme.color.primary
    }
}
```

Theme bukan bagian dari primitive element dan tidak boleh mengubah syntax core.

---

# 53. File Structure

Project aplikasi dapat memiliki struktur:

```text
my-app/
├── app.lang
├── components/
│   ├── UserCard.lang
│   └── Sidebar.lang
├── styles/
│   └── theme.lang
├── assets/
└── package/
```

Nama extension final bahasa ditentukan kemudian.

---

# 54. Module System

Module system harus menggunakan satu mekanisme import.

Target:

```text
import UserCard from "./components/UserCard"
```

Tidak boleh ada beberapa sistem import untuk kebutuhan yang sama.

---

# 55. Compiler Architecture

Compiler secara konseptual:

```text
Source
   ↓
Lexer
   ↓
Parser
   ↓
AST
   ↓
Semantic Analysis
   ↓
Reactive Analysis
   ↓
Component Analysis
   ↓
Code Generation
   ↓
Web Runtime
```

Compiler harus menjaga semantics bahasa tetap konsisten.

---

# 56. Runtime Architecture

Runtime:

```text
Application Runtime
├── Element Runtime
├── State Runtime
├── Reactive Runtime
├── Event Runtime
├── Component Runtime
├── Style Runtime
├── Layout Runtime
├── Animation Runtime
└── Browser Runtime
```

---

# 57. Compiler Optimization

Compiler harus dapat:

* menghilangkan code yang tidak digunakan;
* mengoptimalkan reactive dependency;
* mengoptimalkan event;
* mengoptimalkan rendering;
* melakukan static analysis;
* melakukan tree shaking;
* melakukan code splitting.

Namun optimasi tidak boleh mengubah behavior aplikasi.

---

# 58. Development Mode

Development mode harus mendukung:

```text
hot reload
error overlay
source mapping
state inspection
event inspection
component inspection
performance information
```

---

# 59. CLI

Target CLI:

```text
lang new my-app
lang dev
lang build
lang run
lang check
lang format
lang test
```

Nama CLI mengikuti nama final bahasa.

---

# 60. Formatter

Formatter resmi harus tersedia.

Tujuannya memastikan:

* indentasi konsisten;
* spacing konsisten;
* block konsisten;
* syntax tidak memiliki banyak gaya formatting.

Developer tidak perlu memilih style formatting sendiri untuk syntax dasar.

---

# 61. Language Server

Language Server harus mendukung:

* autocomplete;
* diagnostics;
* go to definition;
* rename;
* hover information;
* symbol search;
* component discovery;
* event discovery;
* state discovery;
* primitive element discovery.

---

# 62. Testing

Bahasa harus memiliki testing system.

Minimal:

```text
unit test
component test
event test
state test
render test
integration test
```

Contoh target:

```text
test Counter {

    click "+"
    expect count == 1
}
```

Detail syntax testing dapat ditentukan setelah core language stabil.

---

# 63. Debugging

Debugger harus dapat memperlihatkan:

```text
Component
State
Derived State
Event
Element
Property
```

Contoh:

```text
Counter
 ├── state count = 4
 ├── Button "+"
 └── Text count
```

---

# 64. Primitive Element Rules

Primitive baru hanya boleh ditambahkan jika:

1. memiliki fungsi yang jelas;
2. digunakan pada aplikasi nyata;
3. tidak sekadar variasi style;
4. tidak dapat digantikan dengan composition sederhana;
5. memiliki API yang konsisten;
6. memiliki event yang jelas;
7. memiliki accessibility semantics jika relevan.

Contoh:

`ButtonPrimary` tidak perlu menjadi primitive.

`DataGrid` dapat menjadi primitive karena memiliki behavior dan rendering model yang kompleks.

---

# 65. Custom Element Policy

Bahasa **tidak mendorong custom element sebagai mekanisme styling**.

Gunakan:

```text
style
property
state
event
component
```

sesuai kebutuhan.

Custom component digunakan ketika ada behavior atau struktur yang memang reusable.

---

# 66. Contoh Aplikasi Lengkap

```text
app TodoApp {

    state todos = []
    state text = ""
    state loading = false
    state error = null

    Column {

        Heading "Todo List"

        Row {

            Input {
                bind value: text
                placeholder: "Tambah todo"

                on keydown {

                    if event.key == "Enter" {
                        addTodo()
                    }
                }
            }

            Button "Tambah" {

                disabled: text == ""

                on click {
                    addTodo()
                }
            }
        }

        if loading {
            Loader
        }

        if error {
            Text error
        }

        for todo in todos {

            Row {

                Checkbox {
                    bind value: todo.done
                }

                Text todo.title

                Button "Hapus" {

                    on click {
                        removeTodo(todo)
                    }
                }
            }
        }
    }

    fn addTodo() {

        todos.push({
            title: text,
            done: false
        })

        text = ""
    }

    fn removeTodo(todo) {
        todos.remove(todo)
    }
}
```

Kode tersebut harus sudah menggambarkan hampir seluruh filosofi bahasa:

```text
Element
State
Binding
Event
Condition
Loop
Function
Style
Component
Reactive UI
```

---

# 67. Syntax Rules yang Dikunci

Syntax inti berikut dianggap sebagai **kontrak v0.1**.

| Konsep       | Syntax                  |
| ------------ | ----------------------- |
| Application  | `app Name { }`          |
| Component    | `component Name { }`    |
| Element      | `Element`               |
| Element text | `Element "text"`        |
| Property     | `property: value`       |
| State        | `state name = value`    |
| Derived      | `derived name = value`  |
| Event        | `on event { }`          |
| Emit         | `emit event(value)`     |
| Function     | `fn name() { }`         |
| Conditional  | `if condition { }`      |
| Else         | `else { }`              |
| Loop         | `for item in items { }` |
| Binding      | `bind property: state`  |
| Style        | `style { }`             |
| Async        | `async fn`              |
| Await        | `await expression`      |
| Try          | `try { }`               |
| Catch        | `catch error { }`       |

Prinsip penting:

> **Satu konsep = satu syntax utama.**

---

# 68. Syntax yang Tidak Digunakan

Core language tidak menggunakan:

```text
<div>
<span>
<button>
```

untuk element.

Tidak menggunakan:

```text
@click
@input
onclick
onClick
```

untuk event.

Tidak menggunakan:

```text
:property
[property]
{{value}}
```

sebagai sistem binding utama.

Tidak menggunakan:

```text
setState()
render()
updateDOM()
```

sebagai mekanisme reactive UI normal.

---

# 69. Roadmap

## Phase 0 — Language Specification

* syntax;
* grammar;
* keywords;
* literals;
* expressions;
* types;
* scope;
* AST specification.

## Phase 1 — Compiler Core

* lexer;
* parser;
* AST;
* semantic analyzer;
* basic code generation.

## Phase 2 — UI Core

* primitive elements;
* element tree;
* properties;
* style;
* component.

## Phase 3 — Reactive Core

* state;
* derived state;
* dependency tracking;
* reactive update.

## Phase 4 — Event Core

* event dispatcher;
* event object;
* event propagation;
* custom event;
* lifecycle event.

## Phase 5 — Layout

* Row;
* Column;
* Stack;
* Grid;
* Container;
* Scroll;
* responsive layout.

## Phase 6 — Browser Runtime

* DOM integration;
* browser event integration;
* fetch;
* storage;
* clipboard;
* file;
* WebSocket.

## Phase 7 — Advanced UI

* Canvas;
* Editor;
* CodeEditor;
* DataGrid;
* Tree;
* advanced interaction.

## Phase 8 — Developer Tools

* CLI;
* formatter;
* LSP;
* debugger;
* hot reload;
* testing.

## Phase 9 — Optimization

* fine-grained rendering;
* batching;
* virtualization;
* code splitting;
* tree shaking;
* production optimization.

---

# 70. MVP

MVP tidak perlu memiliki semua primitive dan semua browser API.

MVP harus membuktikan bahwa model berikut bekerja dengan baik:

```text
app
component
element
property
style
state
derived
event
binding
if
for
fn
reactivity
```

MVP minimal harus dapat membuat:

```text
Counter
Todo
Login Form
Dashboard
Modal
List
Table
Navigation
Responsive Layout
```

---

# 71. Acceptance Criteria

Bahasa dianggap mencapai MVP apabila:

### Syntax

* syntax core dapat diparse;
* syntax konsisten;
* compiler dapat memberikan error yang jelas.

### Element

* primitive element dapat dirender;
* property dapat digunakan;
* children dapat digunakan;
* element dapat di-style.

### State

* state dapat dibuat;
* state dapat berubah;
* UI bereaksi terhadap perubahan;
* dependency state dapat dilacak.

### Event

* event dasar bekerja;
* event object tersedia;
* custom event bekerja;
* event dapat mengubah state.

### Component

* component dapat dibuat;
* component dapat menerima input;
* component dapat memiliki state;
* component dapat emit event;
* component dapat digunakan kembali.

### Layout

* layout dasar bekerja;
* responsive layout dasar bekerja;
* scrolling bekerja;
* nested layout bekerja.

### Runtime

* aplikasi dapat dijalankan di browser;
* reactive update tidak melakukan rerender global yang tidak diperlukan;
* event dan state tetap konsisten setelah update.

---

# 72. Contoh Mental Model Developer

Developer tidak perlu berpikir:

```text
HTML
    ↓
DOM
    ↓
CSS
    ↓
JavaScript
    ↓
State Library
    ↓
Framework
```

Developer cukup berpikir:

```text
Application
    ↓
Component
    ↓
Element
    ├── Style
    ├── State
    └── Event
```

Dan runtime mengurus detail implementasinya.

---

# 73. Identitas Produk

Bahasa ini dikategorikan sebagai:

**UI Application Programming Language**

Bukan:

```text
HTML replacement
CSS framework
JavaScript framework
Component library
State library
```

Fokus utamanya adalah:

```text
Modern Web Application Development
```

---

# 74. Prinsip Akhir

Seluruh pengembangan bahasa harus mempertahankan prinsip berikut:

```text
1. Element harus mudah digunakan.
2. Primitive harus cukup lengkap.
3. Primitive tidak harus menyerupai HTML.
4. Element dapat di-style ulang.
5. Custom component digunakan untuk composition, bukan sekadar styling.
6. State adalah fitur native.
7. Event adalah fitur native.
8. Event harus sangat lengkap.
9. Reactive UI adalah behavior default.
10. Component adalah unit reusable.
11. Syntax harus konsisten.
12. Satu konsep tidak boleh memiliki banyak syntax tanpa alasan kuat.
13. Browser DOM adalah target runtime, bukan model programming utama.
14. Performance harus diperhatikan sejak desain runtime.
15. Bahasa harus mampu berkembang dari UI sederhana menjadi aplikasi web kompleks.
```

**Core model:**

```text
                 APPLICATION
                      │
                  COMPONENT
                      │
          ┌───────────┼───────────┐
          │           │           │
       ELEMENT      STATE       EVENT
          │           │           │
          └───────────┼───────────┘
                      │
                    STYLE
                      │
                      ↓
               REACTIVE RUNTIME
                      │
                      ↓
                    WEB
```

Tujuan akhir bahasa adalah memungkinkan developer membangun aplikasi seperti **IDE browser, editor desain, aplikasi dokumen, dashboard, SaaS, dan aplikasi produktivitas** menggunakan satu bahasa dengan model UI yang konsisten.
