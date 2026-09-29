# Status Implementasi dan Roadmap Bahasa Platipus

---

## 1. Tujuan Dokumen

Dokumen ini mencatat **status implementasi yang nyata** dari setiap fase dalam
[`docs/prd.md`](prd.md) bagian 69, beserta backlog pekerjaan yang masih harus
diselesaikan.

`README.md` bagian Status merujuk ke `docs/roadmap.md` sebagai sumber kebenaran
status fase, jadi isi dokumen ini hanya boleh menyatakan fakta yang terverifikasi
pada kode. `docs/struktur.md` menjelaskan **arsitektur target**, bukan kondisi
repository saat ini, sehingga tidak dapat dipakai sebagai acuan status.

Aturan pemakaian dokumen ini:

* isi dokumen ini hanya boleh menyatakan fakta yang terverifikasi pada kode;
* setiap klaim status harus disertai bukti berupa `path:line`;
* referensi `path:line` memakai path relatif dari root repository, kecuali
  `README.md` dan `Cargo.toml` yang sudah relatif terhadap root;
* ketika suatu item berubah status, tabel pada bagian 4 dan checklist pada
  bagian 6 harus diperbarui pada perubahan yang sama;
* `docs/prd.md` tetap dokumen kebutuhan, `docs/struktur.md` tetap dokumen
  arsitektur target. Dokumen ini tidak menggantikan keduanya.

**Audit terakhir: 2026-09-29**, repo `platipus` v0.1.0, workspace 4 crate
(`compiler`, `runtime`, `cli`, `standard`). Revisi ini merangkum perbaikan
runtime sebelumnya dan menutup celah pada showcase agar klaim demo cocok dengan
perilaku browser nyata:

* `on input` + `bind` pada node yang sama rusak karena render di antara dua
  listener. Kedua jenis listener kini berbagi satu DOM listener per event, jadi
  satu dispatch adalah satu transaksi (bagian 4.3 dan 4.10);
* `event.data` untuk `on drop` tidak pernah ada, sehingga payload drag hanya
  bisa dibaca lewat `dataTransfer` (bagian 4.5);
* `clear()` membersihkan area 1000x1000 tetap, bukan ukuran canvas (bagian 4.8);
* `applyDom()` menulis `textContent` pada `contenteditable` tanpa syarat, yang
  membuang caret dan membatalkan ketikan yang baru saja masuk state (bagian 4.3).
* Fallback `openFile()` tidak menangani event `cancel` dari `<input type="file">`,
  sehingga promise tidak pernah selesai saat pengguna menutup dialog. Fallback
  kini menyelesaikan hasil dengan `cancelled: true` dan melepas listener; jalur
  cancel maupun pemilihan file diuji di `runtime.mjs` (bagian 4.7).
* `webSocket()` sebelumnya langsung mengembalikan handle berstatus `connecting`,
  sehingga `send()` dapat dipanggil sebelum browser membuka koneksi. Sekarang
  `await webSocket(url)` menunggu `open` atau status gagal/timeout, dan showcase
  meminta URL server nyata alih-alih memakai `ws://echo` yang hanya ada di shim.
* Showcase kini benar-benar menguji form submit/persist, sumber payload drag,
  Tree yang dikomposisi dari state/conditional, Tab pada CodeEditor, animasi
  canvas, API browser, dan calculator empat operasi melalui
  `tests/web/showcase.mjs`.
* Showcase monolitik dipisah menjadi shell `examples/showcase/main.plt`,
  library `widgets.plt`/`design.plt`, dan satu modul per area fitur di
  `examples/showcase/sections/`.

Semua `path:line` ke `compiler/src/codegen/web/javascript.rs` dihitung ulang pada
revisi ini: file itu tumbuh sekitar 150 baris karena pekerjaan batching, sehingga
nomor baris dari versi dokumen sebelumnya sudah tidak berlaku. Angka yang berubah
tercatat di bagian 7.

---

## 2. Legenda Status

| Status | Arti |
| --- | --- |
| **SELESAI** | Fitur bekerja dari permukaan bahasa sampai target runtime, dan ada bukti eksekusi. |
| **SEBAGIAN** | Permukaan bahasa ada, tetapi sebagian konteks tidak ada atau tidak berfungsi. |
| **BELUM** | Tidak ada implementasi, dan tidak merusak program yang sudah berjalan. |
| **RUSAK** | Kompilasi lolos tanpa error, tetapi hasil yang dihasilkan salah atau kosong. |

Status **RUSAK** sengaja dibedakan dari **BELUM** karena keduanya memerlukan
tindakan yang berbeda. Yang **RUSAK** adalah pemanggilan yang menyesatkan dan
lebih berbahaya karena developer mengira fiturnya tersedia, sedangkan yang
**BELUM** hanya gap cakupan fitur yang mudah diprediksi.

---

## 3. Ringkasan per Fase

| Fase | Judul | Status | Ringkasan |
| --- | --- | --- | --- |
| 0 | Language Specification | **SEBAGIAN** | Syntax, keywords, literals, expressions, dan scope selesai. **Types belum ada**: `TypeRegistry` adalah stub yang tidak pernah dipakai. |
| 1 | Compiler Core | **SELESAI** | Lexer, Parser, AST, Semantic, dan Codegen berjalan penuh, ditambah lapisan IR yang tidak disebut di PRD. Tidak ada type checking. |
| 2 | UI Core | **SELESAI** | 62 primitive terdaftar — seluruh 47 primitive PRD plus 15 ekstra — dan ke-62 punya properti, 46 nama properti berbeda. Lihat bagian 4.3. |
| 3 | Reactive Core | **SELESAI** | 5 jenis state, derived, dependency tracking, dan reactive update berjalan. |
| 4 | Event Core | **SELESAI** | 28 event DOM terpetakan, custom event, keempat lifecycle event, event object ternormalisasi (`event.value`/`key`/`position`/`data`), dan propagation (`stopPropagation` + delegasi capture) bekerja. Lihat bagian 4.5. |
| 5 | Layout | **SELESAI** | Semua primitive layout PRD bekerja, `responsive { }` menghasilkan `@media`, dan posisi scroll kini menjadi state reaktif lewat binding `scrollTop`/`scrollLeft` maupun `on scroll` + `event.position`. Lihat bagian 4.6. |
| 6 | Browser Runtime | **SELESAI** | DOM integration dan browser event integration selesai (Phase 2-5), lalu `fetch`, `clipboard`, `file`, dan `WebSocket` kini bekerja lewat 9 builtin, dan `storage` lengkap: `local`/`session` sinkron plus `indexed` (IndexedDB). Lihat bagian 4.7. |
| 7 | Advanced UI | **SEBAGIAN** | Kelima item punya permukaan bahasa dan dukungan runtime: `Canvas` (2D context, fill/clear/drawText, loop frame), `Editor` (contenteditable, `selection()`, `exec()`, `indent()`), `CodeEditor` (overlay token yang disinkronkan), `DataGrid` (`sortBy()` + `page()`), dan `Tree` (DnD lewat `event.data`). Yang belum ada: path/arc, undo, keymap, gutter, virtualisasi grid, dan runtime khusus `Tree`. Lihat bagian 4.8. |
| 8 | Developer Tools | **SEBAGIAN** | CLI punya 9 command termasuk `new`, `dev`, `format`, dan runner `test`. Formatter sudah ada dan `new` kini menulis template yang sudah terformat. Hot reload ada: `dev` memantau entry dan semua file yang diimportnya. LSP dan debugger tidak ada. |
| 9 | Optimization | **SEBAGIAN** | 1 dari 6 item berubah dari nol: **batching** kini nyata sebagai satu transaksi per dispatch, dengan dirty set dan flush tunggal. Fine-grained rendering, virtualisasi, code splitting, tree shaking, dan jalur produksi masih nol. |

---

## 4. Status Detail per Fase

### 4.1 Phase 0 - Language Specification

| Item PRD | Status | Bukti |
| --- | --- | --- |
| syntax | **SEBAGIAN** | Tersebar di `docs/prd.md` bagian 67-68 dan `docs/prd.md:1964-2040`, dengan contoh sintaks di `README.md:28-43`. Dokumen permukaan `docs/syntax.md` belum ada. |
| grammar | **SEBAGIAN** | Parser adalah recursive descent tulisan tangan di `compiler/src/parser/`. Tidak ada file grammar formal. |
| keywords | **SELESAI** | `compiler/src/lexer/keyword.rs` |
| literals | **SELESAI** | `compiler/src/lexer/literal.rs` |
| expressions | **SELESAI** | `compiler/src/parser/expression.rs` |
| types | **BELUM** | `compiler/src/semantic/types.rs:21-23`. `TypeRegistry` mendaftarkan 6 primitive lalu membuang nilai primitivenya di `compiler/src/semantic/types.rs:43` (`let _ = primitive;`). Metode `by_name()` di `compiler/src/semantic/types.rs:46-48` tidak pernah dipanggil. `compiler/src/semantic/checker.rs:18` menyimpan field `types`, `compiler/src/semantic/checker.rs:36` mengisinya, dan tidak ada query lain terhadapnya. **Tidak ada type inference maupun type checking.** |
| scope | **SELESAI** | `compiler/src/semantic/scope.rs` dan `compiler/src/semantic/name_resolver.rs` |
| AST specification | **SEBAGIAN** | Struktur AST ada di `compiler/src/ast/` (10 file), tetapi spesifikasi formalnya hanya prosa di `docs/prd.md`. |

**Dampak:** acceptance criteria `docs/prd.md` bagian 71 yang mensyaratkan
"compiler dapat memberikan error yang jelas" untuk kasus salah tipe belum
terpenuhi, karena compiler tidak pernah memeriksa tipe. Diagnostic yang ada
bersifat struktural, misalnya `unknown-element`, `unknown-property`, dan
`unknown-breakpoint`.

**Catatan sistem warning.** `Warning` dan `WarningKind` benar-benar dipakai:
`cli/src/output/diagnostics.rs:26-27` merendernya di dalam `render()` yang
dimulai di `:8`, sehingga `build` dan `check` keduanya menampilkannya. Tetapi
dari 11 variant di `compiler/src/diagnostics/warning.rs:6-18`, hanya
`Convention` yang pernah dibuat, dan hanya di satu tempat,
`compiler/src/semantic/checker.rs:206-215` untuk `import` tanpa ekstensi
`.plt`. `WarningKind::UnusedImport` dideklarasikan di
`compiler/src/diagnostics/warning.rs:8` tetapi tidak pernah diinstansiasi,
sehingga program yang mengimpor apa pun tanpa memakainya tidak pernah
diperingatkan. Lihat B7.

### 4.2 Phase 1 - Compiler Core

| Item PRD | Status | Bukti |
| --- | --- | --- |
| lexer | **SELESAI** | `compiler/src/lexer/scanner.rs`, `token.rs`, `keyword.rs`, `literal.rs`, `operator.rs` |
| parser | **SELESAI** | `compiler/src/parser/` dengan 10 file: `app.rs`, `component.rs`, `element.rs`, `event.rs`, `expression.rs`, `function.rs`, `mod.rs`, `state.rs`, `statement.rs`, `style.rs` |
| AST | **SELESAI** | `compiler/src/ast/` dengan 10 file |
| semantic analyzer | **SEBAGIAN** | `compiler/src/semantic/checker.rs` memvalidasi nama, element, event, handler, dan scope. Tidak memvalidasi tipe, lihat bagian 4.1. |
| basic code generation | **SELESAI** | `compiler/src/codegen/web/` menghasilkan `html.rs`, `css.rs`, `javascript.rs`, `dom.rs`, `tests.rs`, dan `bundle.rs` |

Di luar daftar PRD, sudah ada satu lapisan **IR** yang tidak disebut di
`docs/prd.md` bagian 69: `compiler/src/ir/lower.rs` untuk AST ke IR, dan
`compiler/src/ir/verifier.rs` untuk validasi IR sebelum codegen. Verifier
memang punya teeth: ia menolak modul kosong (`:39`), nama app yang tidak cocok
(`:47`), path import kosong (`:61`), theme kosong (`:70`), test kosong (`:79`),
symbol IR duplikat (`:97`, `:107`), penulisan ke derived (`:181`), dan
pernyataan deklarasi di dalam handler (`:246`). Lapisan inilah yang membuat
pipeline di `compiler/src/pipeline.rs` dapat mendeteksi error sebelum
menghasilkan JavaScript.

**Test suite terdaftar di Cargo.toml**, 9 target yang menguji internal
compiler dan CLI:

| Suite | File | Registrasi | Test |
| --- | --- | --- | --- |
| lexer | `tests/lexer/main.rs` | `compiler/Cargo.toml:16-18` | 17 |
| parser | `tests/parser/main.rs` | `compiler/Cargo.toml:20-22` | 4 |
| semantic | `tests/semantic/main.rs` | `compiler/Cargo.toml:24-26` | 38 |
| ir | `tests/ir/main.rs` | `compiler/Cargo.toml:28-30` | 25 |
| codegen | `tests/codegen/main.rs` | `compiler/Cargo.toml:32-34` | 64 |
| loader | `tests/loader/main.rs` | `compiler/Cargo.toml:36-38` | 10 |
| cli | `tests/cli/main.rs` | `cli/Cargo.toml:22-24` | 31 |
| fixtures | `tests/fixtures/main.rs` | `cli/Cargo.toml:26-28` | 2 |
| web | `tests/web/main.rs` | `cli/Cargo.toml:30-32` | 1 |

Ditambah unit test di dalam crate: 32 di `platipus-cli`, 40 di
`platipus-compiler`, 16 di `platipus-runtime`, 25 di `platipus-standard`.

**Program `.plt` milik pengguna sekarang dijalankan, bukan hanya dikompilasi.**
`tests/web/main.rs` membangun delapan entry termasuk showcase dengan import
transitif, lalu menjalankan `run.mjs`, `templates.mjs`, `lifecycle.mjs`,
`events.mjs`, `runtime.mjs`, `ui.mjs`, `components.mjs`, dan `showcase.mjs` di
atas `dom.mjs` hasil generate. Kedelapannya terdaftar di `HARNESSES`; selain
`examples/counter.plt` dan `examples/showcase/main.plt`, enam harness memakai
fixture di `tests/web/fixtures/`. Direktori scratch dibersihkan setelah tiap
harness. Semua script menerima direktori build sebagai `argv[2]`, agar tidak
menguji artefak checkout yang basi. Selain itu
`tests/cli/main.rs` menjalankan `plt test` melalui Node dan menjalankan `plt dev`
sungguhan, menyalakan server, meminta `/`, lalu mengedit entry file ataupun file
yang diimport dan menunggu sinyal reload
(`dev_rebuilds_when_an_imported_file_changes`).

Status saat audit: `cargo test --workspace` hijau, **305 test lulus dan 0 gagal**.
Ada 19 target hasil test: 15 unit/integration target dan 4 doc-test target;
dua target binary aplikasi dan seluruh doc-test kosong. Tahap Phase 4 (bagian
4.5) menambah 10 test: 7 unit test
di `compiler/src/codegen` (5 untuk `merge_handlers`, 2 untuk `needs_capture`),
3 integrasi handler di `tests/codegen/main.rs`, dan 1 harness web baru
(`events.mjs`) yang menambah 1 test. Tahap Phase 5 (bagian 4.6) menambah
`a_scroll_top_binding_keeps_state_in_step_with_the_scroll` di
`tests/codegen/main.rs` plus 1 assertion binding scroll di `events.mjs`. Tahap
Phase 6 (bagian 4.7) menambah 4 test integrasi kodegen
(`a_fetch_call_is_lowered_to_the_plt_runtime`,
`a_web_socket_handle_keeps_its_method_calls`, `clipboard_file_and_storage_builtins_map_to_the_runtime`,
`a_handler_body_may_await`), 1 unit test shim yang diperluas
(`the_shim_offers_the_globals_the_runtime_reads`), dan 1 harness web
baru (`runtime.mjs`) yang menguji permukaan browser runtime. Tahap Phase 7
(bagian 4.8) menambah 5 test integrasi kodegen, 1 harness web baru
(`ui.mjs`, 7 test), dan revisi ini menambah 2 test lagi
(`the_scaffolded_project_is_already_formatted` di `tests/cli/main.rs` serta
assertion listener dan penulisan `contenteditable` di `tests/web/ui.mjs`).
Angka ini cocok dengan hitungan statis 305 test. Test yang butuh Node
melompat sendiri bila `node` tidak ada di `PATH` (`tests/web/main.rs:52-55`,
`tests/cli/main.rs:304-310`). Pada mesin audit Node v24.18.0 ada di `PATH`, jadi
kedelapan harness benar-benar berjalan: 10 + 9 + 11 + 5 + 8 + 7 + 9 + 9 = 68
check, nol gagal. `runtime.mjs` juga menguji pemilihan file dan pembatalan pada
fallback `<input type="file">`; `showcase.mjs` menjalankan interaksi demo
melalui module loader yang sama dengan build CLI.

### 4.3 Phase 2 - UI Core

| Item PRD | Status | Bukti |
| --- | --- | --- |
| primitive elements | **SELESAI** | 62 builtin terdaftar di `compiler/src/semantic/element.rs:169-232` dan dibangun oleh `builtins::all()` di `compiler/src/semantic/element.rs:296-458`, dipetakan satu banding satu ke tag HTML di `compiler/src/codegen/elements/mod.rs:12-76`. Seluruh 47 primitive PRD bagian 26 sekarang ada, ditambah 15 ekstra yang tidak di-PRD. |
| element tree | **SELESAI** | `compiler/src/ast/element.rs`, `compiler/src/codegen/elements/mod.rs`, dan vdom (`vnode`/`el`/`fragment`/`branch`) di `compiler/src/codegen/web/javascript.rs:578-594` plus rekonsiliasi (`keyOf`/`sameKind`/`patch`/`patchList`) di `:615-728` |
| properties | **SELESAI** | 46 nama properti berbeda dideklarasikan untuk seluruh 62 builtin. 16 properti bersama untuk box (singkatnya "layout") dari `compiler/src/semantic/element.rs:277-294`; daftar elemen berbox melampaui kategori layout lewat `BOX_ELEMENTS` di `compiler/src/semantic/element.rs:239-270`; properti spesifik per elemen di `compiler/src/semantic/element.rs:306-456` (seluruh `all()`, `compiler/src/semantic/element.rs:296-458`). |
| style | **SELESAI** | `compiler/src/codegen/style/mod.rs` dan `compiler/src/codegen/web/css.rs`, termasuk pseudo-class `:hover`, `:active`, `:focus-within`, dan `:disabled` dari `selector()` di `compiler/src/codegen/style/mod.rs:88-95` |
| component | **SELESAI** | `compiler/src/ast/component.rs`, `compiler/src/parser/component.rs`, `compiler/src/codegen/components/mod.rs` |

**Ke-47 primitive PRD bagian 26 sekarang terdaftar.** Sebelumnya hanya 27 dari
47 yang ada; audit ini menambahkan 20 yang belum ada: `Heading`, `Date`, `Time`,
`File`, `Color`, `ContextMenu`, `Command`, `Panel`, `Splitter`, `Spacer`,
`Tree`, `DataGrid`, `Dialog`, `Popover`, `Navigation`, `Sidebar`, `Toolbar`,
`Viewport`, `Editor`, `CodeEditor`. `Container` sudah ada sejak B2 ditutup.
`docs/prd.md:724-810` memuat 47 primitive: 7 di 26.1 Content, 11 di 26.2 Input,
4 di 26.3 Action, 9 di 26.4 Layout, 4 di 26.5 Data, 8 di 26.6 Application UI,
dan 4 di 26.7 Advanced UI.

**15 builtin tidak disebut di PRD.** Sebelas sudah ada sejak audit sebelumnya:
`Page`, `Card`, `Form`, `Field`, `Option`, `Sheet`, `Toast`, `MenuItem`, `Tab`,
`Loader`, `Spinner`. Tiga ditambahkan audit ini untuk melengkapi data table:
`TableRow`, `Cell`, `Header`, yang membuat `DataGrid` dapat diekspresikan.
62 = 47 PRD + 15 ekstra.

Per-element surface (input di 26.2): `Input`, `Textarea`, `Select`, `Option`,
`Checkbox`, `Radio`, `Switch`, `Slider`, `Date`, `Time`, `File`, `Color`
mendeklarasikan properti yang benar-benar dirender: `value`, `placeholder`,
`checked`, `name`, `min`, `max`, `step`, `accept`, `multiple` (boolean
attribute), dan `value` sebagai properti DOM. Media di 26.x: `Video` punya
`poster`, `controls`, `autoplay`, `loop`, `muted`; `Audio` punya `controls`,
`autoplay`, `loop`; `Canvas` punya `width` dan `height` sebagai atribut HTML
(lihat `property_target` di `compiler/src/codegen/elements/mod.rs:115-130`).
`Form` punya `action`, `method`, `novalidate`; `Heading` punya `level`, `size`,
`weight`; `Text` punya `size`, `weight`; `Link` punya `href`, `target`;
`Command` punya `shortcut` dan `disabled`.

**Property dideklarasikan selalu dirender, bukan dekoratif.** `is_style_property`
di `compiler/src/codegen/elements/mod.rs:100-117` mengarahkan properti "box"
ke CSS (elemen layout + `BOX_ELEMENTS`, digabung dengan properti bersama dari
kanal `Style`), sedangkan sisanya menjadi atribut HTML atau properti DOM. Dua
pengecualian CSS baru: `size` → `font-size` dan `weight` → `font-weight` di
`compiler/src/codegen/style/mod.rs:13-17`, jadi `Heading "T" { level: 2 size: 20
weight: 700 }` menghasilkan tag `h2` (dipilih `heading_tag()` di
`compiler/src/codegen/components/mod.rs:149-166`), class `plt-eN` dengan dua
deklarasi CSS, dan tidak ada atribut `level` yang bocor. Media tidak ikut
"box": `Video { width: 100 }` tetap menjadi atribut HTML karena semantik
`<video width>`; test `only_layout_primitives_take_style_properties` di
`compiler/src/codegen/elements/mod.rs:231-237` menjaga batas itu.

Guard jumlah: `catalog_primitives_are_registered` dan
`catalog_covers_the_documented_builtin_surface` di `tests/semantic/main.rs`
memotret 62 builtin dan 46 nama properti, jadi katalog tidak menyusut tanpa
disadari. `accepts_the_full_primitive_catalog` mengkompilasi seluruh katalog
dalam satu pohon.

### 4.4 Phase 3 - Reactive Core

| Item PRD | Status | Bukti |
| --- | --- | --- |
| state | **SELESAI** | 5 jenis state dideklarasikan di `compiler/src/ast/state.rs:5-11`: `Local`, `Shared`, `Global`, `Persistent`, `Derived`. Di IR menjadi `IrStateKind` di `compiler/src/ir/expression.rs:80`, dipetakan ke scope penyimpanan di `compiler/src/codegen/state/mod.rs:88-95`. Sisi host ada di `runtime/src/state.rs`. |
| derived state | **SELESAI** | `runtime/src/derived.rs`; dependensi dikumpulkan oleh `dependencies()` di `compiler/src/ir/lower.rs:293-300` dan dibaca helper `derived_dependencies()` di `compiler/src/codegen/state/mod.rs:114-116` |
| dependency tracking | **SEBAGIAN** | Daftar dependensi diteruskan ke `plt.computed()` di `compiler/src/codegen/web/javascript.rs:246`. Dependensi dipakai untuk menentukan kapan nilai dihitung ulang, **tidak** untuk mempersempit re-render hanya ke subtree yang berubah. Yang sudah ada adalah batas transaksi: satu dispatch DOM adalah satu render, bukan satu render per listener (lihat catatan di bawah). |
| reactive update | **SELESAI** | `refresh()` di `compiler/src/codegen/web/javascript.rs:1372-1396`, dipicu oleh setter signal di `compiler/src/codegen/web/javascript.rs:233-237` |

Catatan performa: fan-out signal bersifat sinkron, sehingga satu penulisan
state memicu satu render penuh secara langsung. Yang berubah pada revisi ini
adalah **batas dispatch**: `refresh()` di `:1374-1378` tidak lagi merender saat
`batchDepth > 0`, melainkan menandai instance dan menaruhnya di `dirtied`, lalu
`endBatch()` di `:204-212` drain set itu sampai kosong. Satu dispatch yang
menulis state beberapa kali karena itu menghasilkan satu render, bukan beberapa.
Batas ini masih sinkron dan masih per-dispatch: belum ada
`queueMicrotask`/`requestAnimationFrame` yang menggabungkan dispatch berbeda, dan
render masih mencakup seluruh virtual tree komponen, sehingga sisanya tetap
temuan Phase 9 pada bagian 4.10.

### 4.5 Phase 4 - Event Core

| Item PRD | Status | Bukti |
| --- | --- | --- |
| event dispatcher | **SELESAI** | 28 pemetaan nama event ke event DOM di `compiler/src/codegen/events/mod.rs:15-46`. Semua listener dipasang lewat `node.addEventListener` di `compiler/src/codegen/web/javascript.rs:1101` (satu listener bersama per event, lihat catatan batching di bawah), `:1208-1209` (pasangan pointer untuk `swipe`), dan `:1327` (custom event). |
| event object | **SELESAI** | Runtime `plt.event()` di `compiler/src/codegen/web/javascript.rs:831-864` membangun objek `{ value, key, data, position, target, type, originalEvent }` dengan `stopPropagation`/`preventDefault` yang diteruskan ke raw event. Codegen menyuntik `const event = plt.event(raw);` ke handler yang membaca `event` (bukan custom/lifecycle) di `compiler/src/codegen/components/mod.rs:214-217`. `value` membaca `target.value` (atau `checked` untuk checkbox/radio, dan `textContent` untuk `contenteditable`), `position` memakai `clientX`/`clientY`, dan `data` dibaca lewat `transferData()` di `:809-829`: `raw.data` bila ada, kalau tidak `getData("text/plain")` dari `dataTransfer`, dan `undefined` bila tidak ada payload maupun `getData` (yang hanya terbaca di dalam handler drop sungguhan). |
| event propagation | **SELESAI** | `event.stopPropagation()` diteruskan ke raw event (`compiler/src/codegen/web/javascript.rs:853-855`), jadi bubbling bisa dihentikan. Delegasi event: `EventCategory::is_bubbling` di `compiler/src/ast/event.rs:73-84` dipanggil oleh `needs_capture()` di `compiler/src/codegen/events/mod.rs:54-57`, yang mengarahkan event non-bubbling (form + `scroll`) menjadi descriptor `[handler, true]` di `compiler/src/codegen/components/mod.rs:219-223`; `subscribe()` di `compiler/src/codegen/web/javascript.rs:1079-1106` lalu memasang descriptor itu pada fase capture, sehingga ancestor tetap mengamati event yang tidak mencapai dirinya lewat bubbling. Listener capture dan listener bubble pada event yang sama tidak boleh berbagi listener, jadi fase capture adalah bagian dari identitas grup di `:1084` (`id = capture ? \`${key}!\` : key`). |
| custom event | **SELESAI** | Statement `emit` di `compiler/src/parser/event.rs:25-40` menjadi `plt.emit()` di `compiler/src/codegen/statement.rs:80-81`, dengan emitter komponen di `compiler/src/codegen/web/javascript.rs:775-791` dan dispatch payload di `:793-802`. Handler custom menerima payload mentah yang di-`emit`, bukan event DOM: `compiler/src/codegen/components/mod.rs:208-210`. |
| lifecycle event | **SELESAI** | Keempatnya fire dari runtime. Bukti dan semantik di catatan berikut. |

Jumlah event yang dipetakan adalah **28**, yaitu seluruh 32 nama di
`compiler/src/ast/event.rs:94` dikurangi 4 lifecycle. `focusin`, `focusout`, dan
`select` yang sebelumnya tidak punya pasangan kini terpetakan di
`compiler/src/codegen/events/mod.rs:35-37`, sehingga tidak ada lagi nama event
yang dikenal compiler tetapi jatuh ke kanal custom "plt:..." secara keliru.

**Keempat lifecycle event sekarang fire.** B3 menutup cacat yang sama: handler
yang lolos kompilasi tapi tidak pernah dijalankan. `on create` dan `on update`
dihandle oleh runtime JS, bukan lewat channel, karena keduanya adalah titik
siklus hidup, bukan event DOM. Titik pemanggilnya:

* `create` pada element di `compiler/src/codegen/web/javascript.rs:1002`, pada
  component di `:1318` dan `:1405`;
* `mount` pada element di `:1260`, pada component di `:1343`;
* `update` pada element di `:1010`, pada component di `:977` dan `:1391`;
* `destroy` pada element di `:1280`, pada component di `:1296`.

Penulisan state di dalam `mount` tidak hilang: `mountTree()` di
`compiler/src/codegen/web/javascript.rs:1338-1349` menyetel flag `busy` dan
mengantar render lanjutan lewat antrean `pending` yang diproses oleh
`refresh()` di `:1372-1396` (loop `while (instance.pending || instance.dirty)`
pada `:1392`).

Vnode yang dipakai ulang menerima `update` pada setiap render ulang meski
propertinya tidak berubah. Ini pilihan sadar, bukan kelalaian:
`docs/prd.md:1058` hanya menyatakan bahwa lifecycle mengikuti sistem event
dan tidak pernah mensyaratkan pembedaan diff, sedangkan mempersempit `update`
ke diff nyata memerlukan dirty-tracking yang sendirinya item Phase 9.

**Satu dispatch adalah satu transaksi.** Ini closure cacat yang ditemukan audit
ini, dan bentuknya berulang: sebuah `Editor` dengan `bind value:` dan
`on input` pada node yang sama. Handler `on input` menulis state, render terjadi
di akhir handler itu, `applyDom()` menulis ulang `textContent` dari state yang
baru, lalu listener `bind` yang berjalan berikutnya membaca kotak yang sudah
ditimpa: state kembali ke teks lama dan caret meloncat ke awal. Perbaikan bukan
menambah jeda, melainkan memindahkan batas render: `subscribe()` di
`compiler/src/codegen/web/javascript.rs:1079-1106` memasang **satu** listener DOM
per (event, fase) dan menjalankan seluruh callback yang terdaftar di dalamnya di
dalam satu `beginBatch()`/`endBatch()`, sehingga tidak ada render di antara dua
listener pada event yang sama.

Batasnya benar-benar satu dispatch, bukan satu listener. Menembelkan
`transactional()` pada masing-masing listener seperti percobaan pertama tidak
cukup, karena listener pertama menutup transaksinya sendiri sebelum listener
kedua dipanggil; wrapper bersama di `:1090-1097` itulah yang menutup dispatch.
Fasilitas ini sekaligus menghapus listener yang menumpuk: node hanya membawa satu
listener per event, bukan satu per render. `unsubscribe()` di `:1131-1147`
melepas listener DOM begitu tidak ada lagi callback yang memakainya, dan
`unsubscribeAll()` di `:1149-1153` dipakai `destroy()` di `:1273-1288`.

Selain itu `applyDom()` di `compiler/src/codegen/web/javascript.rs:1048-1060`
tidak lagi menulis `textContent` pada `contenteditable` tanpa syarat: nilai
hanya ditulis bila berbeda, sehingga render ulang yang tidak mengubah isi tidak
membuang caret. Bukti eksekusi: `tests/web/ui.mjs` menguji kedua hal itu, yaitu
bahwa satu dispatch tidak menulis sama sekali ke kotak yang sedang diketik, dan
bahwa `on input` beserta `bind` berbagi tepat satu listener.

Selain itu, `EventCategory::Clipboard`, `Media`, `Animation`, dan
`Application` adalah enum variant yang tidak terjangkau: `category_of()` di
`compiler/src/semantic/events.rs:27-53` tidak pernah mengembalikan keempatnya.

### 4.6 Phase 5 - Layout

Status keseluruhan fase ini adalah **SELESAI**.

| Item PRD | Status | Bukti |
| --- | --- | --- |
| Row | **SELESAI** | `compiler/src/codegen/layout/mod.rs:7` memakai `display: flex; flex-direction: row;` |
| Column | **SELESAI** | `compiler/src/codegen/layout/mod.rs:6` memakai `display: flex; flex-direction: column;` |
| Stack | **SELESAI** | `gap: 8` kini menjadi CSS `gap: 8px` lewat kanal `Style` di `compiler/src/codegen/elements/mod.rs`, bukan atribut HTML. |
| Grid | **SELESAI** | `columns: 3` dipetakan ke nama CSS `grid-template-columns` oleh `PROPERTY_EXCEPTIONS` di `compiler/src/codegen/style/mod.rs:16`, lalu
Nilainya menjadi `repeat(3, minmax(0, 1fr))` di `css_value()` pada `:71-73`, dan `gap` mengikuti jalur CSS yang sama. |
| Container | **SELESAI** | Terdaftar di `compiler/src/semantic/element.rs:171`, dipetakan ke `div` di `compiler/src/codegen/elements/mod.rs:15`, dengan base `display: block` di `compiler/src/codegen/layout/mod.rs:5`. |
| Scroll | **SELESAI** | `compiler/src/codegen/layout/mod.rs:9` memberi `overflow: auto;`. Posisi scroll menjadi state reaktif: `on scroll` memberi `event.position` (lihat 4.5), dan binding `bind scrollTop: state`/`bind scrollLeft: state` menyinkronkan posisi scroll dua arah. Binding scroll dipasang oleh `applyBinds()` di `compiler/src/codegen/web/javascript.rs:1212-1244`, yang memilih listener `scroll` (bukan `input`) untuk nama sama dengan awalan `scroll` dan menulis balik `node.scrollTop`/`scrollLeft` ke state; seed tidak melawan scroll yang sedang berjalan karena hanya menulis bila nilai DOM berbeda dari state. Listener scroll ini berbagi listener dengan `on scroll` pada node yang sama lewat
wrapper bersama di `:1090-1097`, jadi satu event scroll menghasilkan satu render. Dilindungi test `a_scroll_top_binding_keeps_state_in_step_with_the_scroll` di `tests/codegen/main.rs` dan 1 assertion di `events.mjs`. Perilaku sticky tidak di-PRD. |
| responsive layout | **SELESAI** | Satu kosakata breakpoint di parser, IR, dan emitter. Lihat catatan berikut. |

Seluruh base rule layout ada di `compiler/src/codegen/layout/mod.rs:3-12`.

**`responsive { }` sekarang menghasilkan `@media`.** Rantainya dulu terputus
karena parser menerima `mobile`/`tablet`/`desktop` sementara emitter mencari
`Small`/`Medium`/`Large`/`XLarge`, lalu melewati kelompok yang tidak ditemukan
secara diam-diam. Sekarang `BREAKPOINTS` di
`compiler/src/codegen/layout/mod.rs:31` memakai kosakata yang sama dengan
parser, `breakpoint_of()` di `:37-42` mengembalikan `None` untuk
`BASE_BREAKPOINT` (`mobile`, `:34`) sehingga `mobile` menjadi base dan tidak
memerlukan `@media`, sedangkan `tablet` (`48rem`) dan `desktop` (`64rem`)
menghasilkan rule. `render_responsive()` di
`compiler/src/codegen/web/css.rs:162-219` tidak lagi melewati kelompok tak
diketahui secara diam-diam.

Element swap per breakpoint dihapus. `mobile: Column { }` tidak lagi parse dan
ditolak dengan `expected-expression`, yang tercakup
`a_responsive_element_swap_does_not_parse` di `tests/parser/main.rs`;
`the_three_canonical_breakpoints_are_accepted` dan
`any_other_breakpoint_name_is_rejected` menjaga kosakata dari dua sisi.

Properti responsif diarahkan ke CSS, bukan ke atribut HTML. Test yang menutup:
`a_responsive_property_override_reaches_the_stylesheet` dan
`a_responsive_named_style_reaches_the_stylesheet` di `tests/codegen/main.rs`.

Klasifikasi dipisah per instance element lewat `override_class()` di
`compiler/src/codegen/components/mod.rs:59-63`, yaitu `plt-e{element.span.start}`,
sehingga `.plt-column` yang di-override pada satu `Column` tidak bleduh ke
semua `Column` lain.

### 4.7 Phase 6 - Browser Runtime

Status keseluruhan fase ini adalah **SELESAI**.

Seluruh touchpoint browser yang dipakai JavaScript hasil generate ada di
`compiler/src/codegen/web/javascript.rs`. **20 API berbeda**, nomor baris
dihitung ulang pada revisi ini:

| API | Call site |
| --- | --- |
| `document.getElementById` | 175, 563, 566 |
| `document.createElement` | 393, 394, 758, 997, 1400 |
| `target.replaceChildren` | 170 |
| `parent.insertBefore` | 764, 934 |
| `node.remove()` | 950 |
| `node.textContent` | 771, 838, 1021, 1031, 1056 |
| `node.setAttribute` | 1044, 1060 |
| `node.removeAttribute` | 1040, 1043 |
| `node.addEventListener` | 397, 1101, 1208, 1209, 1327 |
| `node.removeEventListener` | 1134, 1135, 1142 |
| `globalThis.localStorage` | 275, 283, 480 |
| `globalThis.sessionStorage` | 479 |
| `globalThis.indexedDB` | 486, 488 |
| `JSON.parse` | 276, 329, 537 |
| `JSON.stringify` | 283, 523 |
| `globalThis.fetch` | 315 |
| `navigator.clipboard` | 350, 360 |
| `globalThis.showOpenFilePicker` | 388, 389 |
| `globalThis.WebSocket` | 443 |
| `getContext` (canvas 2D) | 575 |

Catatan versi dokumen sebelumnya tentang `innerWidth` dan `addEventListener`
masih berlaku: keduanya **tidak** dipakai untuk mengukur apa pun.
`globalThis.innerWidth = 1024` di `compiler/src/codegen/web/dom.rs:271` hanyalah
konstanta stub di shim Node, dan `addEventListener` di `:205` adalah placeholder.
Tidak ada `matchMedia`, tidak ada listener `resize`, tidak ada breakpoint yang
dievaluasi di client.

| Item PRD | Status | Bukti |
| --- | --- | --- |
| DOM integration | **SELESAI** | vdom dan rekonsiliasi penuh di `compiler/src/codegen/web/javascript.rs:866-1016` (`patchList` di `:920`, `patch` di `:962`), pemetaan tag di
`compiler/src/codegen/elements/mod.rs:12-55`, pemisahan atribut dan properti DOM di `compiler/src/codegen/elements/mod.rs:84-126` |
| browser event integration | **SELESAI** | 28 event terpetakan, keempat lifecycle event, event object ternormalisasi, dan propagation berjalan, lihat bagian 4.5. |
| storage | **SELESAI** | `store()`/`load()`/`drop()` di `compiler/src/codegen/web/javascript.rs:518-560` melayani tiga backend: `local` dan `session` sinkron lewat `storageBackend()` di `:478-483` (localStorage, sessionStorage, round-trip JSON di `:523` dan `:537`), `indexed` async lewat `idb()`/`idbRequest()` di `:485-516` (IndexedDB, store `kv`, database `platipus`). State `persistent` tetap ditangani `persistent()` di `:271-289`. Shim test menyediakan `localStorage`/`sessionStorage` memakai `memoryStorage()` di `compiler/src/codegen/web/dom.rs` dan IndexedDB deterministik `BlankIndexedDB` di bawahnya. |
| fetch | **SELESAI** | `fetch()` di `compiler/src/codegen/web/javascript.rs:305-344` membungkus `globalThis.fetch` dan selalu resolve: sukses menjadi `{ request, status, ok, type, text, json, error }`, dan gagal menjadi objek yang sama dengan `ok: false` serta `error` terisi, jadi handler yang menulis hasilnya ke state tidak perlu menangkap rejection. |
| clipboard | **SELESAI** | `clipboardWrite()`/`clipboardRead()` di `compiler/src/codegen/web/javascript.rs:346-366` memakai `navigator.clipboard` bila ada dan resolve `false`/`""` bila tidak tersedia. Shim hanya menyuntik clipboard bila `navigator` tidak memilikinya (guard getter-only di `compiler/src/codegen/web/dom.rs`). `EventCategory::Clipboard` di `compiler/src/ast/event.rs:36-49` tetap tidak terjangkau oleh `category_of()` (`compiler/src/semantic/events.rs:27-53`) dan tidak ada clipboard event di `ALL` (`compiler/src/ast/event.rs:94`); kanal event clipboard memang tidak menjadi bagian penyelesaian ini, tetapi jalur baca-tulis clipboard kini ada. |
| file | **SELESAI** | `openFile()` di `compiler/src/codegen/web/javascript.rs` memakai `showOpenFilePicker` bila browser menyediakannya, menyusul fallback `<input type="file">`, dan resolve objek `{ name, size, type, text, cancelled }`. Fallback menangani event `change` dan `cancel`, lalu melepas kedua listener; `runtime.mjs` menguji pemilihan file, pembatalan, dan cleanup listener. |
| WebSocket | **SELESAI** | `webSocket()` membuat koneksi lewat `new WebSocket(url)` yang di-share per URL; promise selesai saat koneksi terbuka atau menghasilkan status `error`, `closed`, `timeout`, atau `unavailable`. `send()` hanya mengirim saat status `open`; `receive()` mengantre pesan dan menyelesaikan `null` bila koneksi ditutup. Shim test memakai `TestSocket` yang auto-open dan echo. `runtime.mjs` memverifikasi status open serta send/receive. |

Permukaan bahasa untuk fase ini adalah **9 builtin yang dipanggil langsung**:
`fetch`, `writeClipboard`, `readClipboard`, `openFile`, `webSocket`, `receive`,
`store`, `load`, `drop`. Rantainya:

* `parse_component_body_items` di `compiler/src/parser/component.rs:84-91`
  mengenali `async fn` sebagai item component (sebelumnya hanya `fn`), dan fn
  async diemisi sebagai `async function` di `compiler/src/codegen/statement.rs:137`;
* ke-sembilan builtin di-bind di **awal** scope lewat `bind_builtins()` di
  `compiler/src/codegen/state/mod.rs:55-69` dan dipanggil pertama oleh
  `scope_for()` (`:74`), jadi state/fungsi user dengan nama sama menimpa builtin;
  semantic checker menerima nama-nama itu sebagai identifier yang boleh tanpa
  deklarasi: `BUILTIN_FUNCTIONS` di `compiler/src/semantic/checker.rs:786`,
  `is_builtin_function()` di `:798`, gate-nya di `:650`;
* setiap handler kini diemisi **async** agar `await` legal di body-nya:
  `merge_handlers` di `compiler/src/codegen/components/mod.rs:204-217`;
* builtin dilower ke panggilan `plt.*`, dan ke-sembilan nama diekspor dari IIFE
  di `compiler/src/codegen/web/javascript.rs:992-1000`.

Bukti eksekusi: 4 test integrasi di `tests/codegen/main.rs` (`a_fetch_call_is_lowered_to_the_plt_runtime`,
`a_web_socket_handle_keeps_its_method_calls`, `clipboard_file_and_storage_builtins_map_to_the_runtime`,
`a_handler_body_may_await`) plus harness Node `runtime.mjs` (lihat bagian 4.2)
yang menguji semuanya dalam satu program `tests/web/fixtures/runtime.plt`:
`fetch` data:, clipboard, file picker stub, WebSocket + `receive`, dan ketiga
backend storage, ditambah pemilihan dan pembatalan fallback file picker — 8
assertion, nol gagal.

Catatan arsitektur: crate `runtime` hanya berisi 4 file datar, yaitu
`lib.rs`, `state.rs`, `derived.rs`, dan `event.rs`, tanpa direktori anak dan
tanpa kode DOM, HTTP, async, maupun I/O. Tidak ada satu pun modul codegen yang
mengimpor `platipus-runtime`, sehingga runtime sisi host saat ini tidak
terhubung ke program yang dihasilkan. `platipus-runtime` juga bukan dependensi
`platipus-cli` lagi: `cli/Cargo.toml:34-38` hanya mendeklarasikan
`platipus-compiler`, karena
grep terhadap `cli/src/` dan `tests/` tidak menemukan satu pun penggunaan
keduanya.

### 4.8 Phase 7 - Advanced UI

Status keseluruhan fase ini adalah **SEBAGIAN**. Versi dokumen sebelumnya
menyatakan seluruhnya **BELUM** dengan bukti negatif seperti "tidak ada
`getContext` di seluruh workspace"; bukti negatif itu sudah basi, dan audit ini
memverifikasi ulang setiap item satu per satu. Yang membedakan **SEBAGIAN** dari
**BELUM** di sini adalah setiap item punya permukaan bahasa *dan* dukungan
runtime yang benar-benar dieksekusi test, tetapi belum lengkap terhadap PRD.

| Item PRD | Status | Bukti |
| --- | --- | --- |
| Canvas | **SEBAGIAN** | `width`/`height` sebagai atribut HTML (lihat 4.3), lalu 5 builtin: `canvas()` di `compiler/src/codegen/web/javascript.rs:562-569` mencari node berdasarkan `id`, `fill()` di `:580-591`, `clear()` di `:593-616`, `drawText()` di `:618-633`, dan `nextFrame()` di `:635-642` untuk loop per frame. `canvas2d()` di `:571-578` memanggil `getContext("2d")` dan menyimpan context di node. Shim menyediakan `getContext` yang merekam operasi di `compiler/src/codegen/web/dom.rs:175-202` dan `globalThis.requestAnimationFrame` di `:282-286`. Bukti eksekusi: `canvas drawing records operations under a resolved id` di `tests/web/ui.mjs`. **Yang belum ada:** path, `arc`, `bezier`, transform, `measureText`, dan gambar. `clear()` juga sudah diperbaiki: area pembersihan diambil dari `node.width`/`node.height` (`:601-602`), bukan konstanta 1000x1000 yang membuat canvas besar tidak pernah bersih dan canvas kecil terisi berlebihan. |
| Editor | **SEBAGIAN** | Dirender sebagai `div` dengan `contenteditable` (lihat 4.3), lalu 3 builtin: `exec()` di `compiler/src/codegen/web/javascript.rs:651-662` menjalankan perintah editor pada node yang fokus, `selection()` di `:664-683` melaporkan `start`/`end` dari `window.getSelection()`, dan `indent()` di `:685-706`. `applyDom()` di `:1048-1060` memperlakukan `contenteditable` sebagai `textContent` dan hanya menulis bila berbeda, sehingga render tidak membuang caret. Bukti eksekusi: `a contenteditable editor binds text and reports the caret` dan `exec runs editing commands on the focused editor` di `tests/web/ui.mjs`. **Yang belum ada:** undo/redo sendiri, peta clipboard di luar `exec`, dan keymap. |
| CodeEditor | **SEBAGIAN** | Dirender sebagai `textarea` dengan `language`/`value`/`placeholder` (lihat 4.3), lalu overlay token: `highlightHtml()` di `compiler/src/codegen/web/javascript.rs:738-752` membagi teks menjadi span `plt-tok`, dan `ensureCodeOverlay()` di `:754-766` memasang overlay yang duduk di samping textarea, dengan `syncCodeOverlay()` di `:768-773` yang menyinkronkannya setiap render. Bukti eksekusi: `a highlighted code editor keeps its token overlay in sync` di `tests/web/ui.mjs`. **Yang belum ada:** gutter nomor baris, font monospace per bahasa, completion, dan LSP. |
| DataGrid | **SEBAGIAN** | Element `DataGrid` terdaftar dan dipetakan ke `table` bersama `TableRow`, `Cell`, dan `Header` (lihat 4.3). Logikanya datang dari 2 builtin: `sortBy()` di `compiler/src/codegen/web/javascript.rs:708-726` (kunci opsional, arah `asc`/`desc`) dan `page()` di `:728-736` (jendela halaman, bukan indeks item). Bukti eksekusi: `sortBy and page window the grid rows` di `tests/web/ui.mjs` dan `data_grid_builtins_sort_and_page_lists` di `tests/codegen/main.rs`. **Yang belum ada:** sorting lewat klik kolom, editing sel, virtualisasi baris, dancolumnwidth. |
| Tree | **SEBAGIAN** | Element `Tree` terdaftar dan dipetakan ke `ul` (lihat 4.3). Expand/collapse **tidak** punya runtime khusus: itu idiom komponen biasa, yaitu
state boolean plus `if`, dan itulah cara `TreeDemo` di
`tests/web/fixtures/ui.plt:95-117` mengujinya. Yang ada di runtime adalah DnD: payload drag kini terbaca sebagai `event.data` lewat `transferData()` di `compiler/src/codegen/web/javascript.rs:809-829`, yang dipakai `on drop`. Bukti eksekusi: `a tree column expands and drops data onto a node` di `tests/web/ui.mjs`. **Yang belum ada:** indentasi guide, lazy load, dan state node tree. |
| advanced interaction | **SEBAGIAN** | `swipe` diturunkan dari pasangan `pointerdown`/`pointerup` oleh `listenForSwipe()` di `compiler/src/codegen/web/javascript.rs:1186-1210` dengan ambang 24px dan payload `{ dx, dy, dir }`; `nextFrame()` di `:635-642` memberi loop per frame; properti `transition` terdaftar di `compiler/src/semantic/element.rs:294` dan dirender sebagai CSS. Bukti: `swipe_is_composed_from_pointer_events` di `tests/codegen/main.rs` yang juga asserting `transition: 0.3s;` benar-benar sampai ke stylesheet. **Yang belum ada:** pinch, spring, dan ambang swipe yang bisa diatur. |

Sepuluh builtin Phase 7 (`canvas`, `fill`, `clear`, `drawText`, `nextFrame`,
`exec`, `selection`, `indent`, `sortBy`, `page`) dipetakan ke scope komponen
lewat `bind_builtins()` di `compiler/src/codegen/state/mod.rs` dengan cara yang
sama seperti sembilan builtin Phase 6 (lihat 4.7), sehingga nama yang sama dipakai
user tetap menimpanya.

Yang **tidak** berubah: `EventCategory::Clipboard`, `Media`, `Animation`, dan
`Application` tetap tidak terjangkau oleh `category_of()` (lihat 4.5), dan tidak
ada listener `resize`/`matchMedia` di mana pun (lihat 4.7). `requestAnimationFrame`
yang ada adalah loop yang diminta program, bukan penjadwalan render framework.

### 4.9 Phase 8 - Developer Tools

| Item PRD | Status | Bukti |
| --- | --- | --- |
| CLI | **SELESAI** | 9 command di `cli/src/command/mod.rs:17-27`: `new`, `build`, `check`, `run`, `test`, `dev`, `format`, `help`, `version`. Flag di `cli/src/config/mod.rs:86-122`: `-h`, `--help`, `-v`, `--version`, `-q`, `--quiet`, `--check`, `-o`, `--out`, `-p`, `--port`. |
| formatter | **SELESAI** | `cli/src/command/format.rs`. Layout-only: setiap token ditulis ulang dari byte aslinya memakai span-nya, jadi hanya whitespace *antar* token yang diganti dan komentar, isi string, serta teks token selamat. BOM disimpan dan dikembalikan, file yang tidak lex ditolak, dan unit test di modul itu menjaga semuanya, termasuk `formatting_is_idempotent`, `string_contents_are_untouched`, `a_file_that_does_not_lex_is_reported`, `a_byte_order_mark_survives_and_does_not_shift_the_source`, dan `a_blank_line_between_statements_is_kept`. Aturannya: satu baris kosong di sebelah kurung kurawal selalu dibuang karena tata letak baris brace ditentukan formatter, sedangkan baris kosong antar pernyataan biasa dipertahankan (`format.rs:112-119` dan `:125-131`). Karena itu template `cli/src/command/new.rs:7` tidak boleh memuat baris kosong di sebelah brace: `new` sekarang menulis persis apa yang akan ditulis `format` atas program yang sama, ditutup `the_scaffolded_project_is_already_formatted` di `tests/cli/main.rs`. |
| LSP | **BELUM** | Tidak ada crate LSP, tidak ada JSON-RPC, tidak ada loop stdio, tidak ada completion, hover, rename, atau definition. `compiler/Cargo.toml:14` memiliki bagian `[dependencies]` yang kosong. |
| debugger | **BELUM** | Tidak ada debugger maupun inspector. Satu-satunya yang berdekatan adalah `plt.bind(target, instance)` di `compiler/src/codegen/web/javascript.rs:978` (dipanggil lewat `plt.bind(target, instance)` di `:168`) yang menempelkan instance ke `target.__plt`, tanpa konsumen selain harness di `tests/web/`. |
| hot reload | **SELESAI** | `dev` memantau lebih dari satu file: snapshot `(len, mtime)` per file (`Stamp` di `cli/src/command/dev.rs:102`, `Stamp::of()` di `:108`) diambil untuk entry dan semua dependensinya di `:86-93`, lalu `snapshots_change()` di `:65-69` dipanggil tiap `POLL` 50 ms (`:15`, `:59`). SSE di `:13`, `notify()` di `:142-149`, `subscribe()` di `:176-184`; client disuntik ke `index.html` di `:125-140`, bukan oleh compiler, sehingga build produksi tidak membawa kode dev. Dependensi datang dari `build::compile_entry` dan memuat entry plus semua modul `import` transitif, jadi mengedit file yang diimpor memicu rebuild. |
| testing | **SEBAGIAN** | Permukaan bahasa dan runner ada. Lihat catatan berikut. |

**Command `run` bukan menjalankan aplikasi.** `run` jatuh ke jalur `build`
(`cli/src/command/run.rs:10`), lalu hanya mencetak pesan untuk disajikan dengan
static file server. Yang menyajikan adalah `dev`.

**Runner `test` ada.** Rantainya dulu lengkap sampai IR lalu datanya dibuang.
Sekarang `Command::Test` di `cli/src/command/mod.rs:66` memanggil `test::run()`
di `cli/src/command/test.rs:12`, yang membangun program seperti `build`, lalu
menjalankan `tests.mjs` hasil generate melalui Node (`run_node()` di `:20`) dan
meneruskan exit code-nya, sehingga program yang gagal membuat build tetap hijau.

| Bagian | Bukti |
| --- | --- |
| emit runner | `tests::render()` di `compiler/src/codegen/web/tests.rs:12` dan `dom::render()` di `compiler/src/codegen/web/dom.rs:197` menghasilkan isi `tests.mjs` dan `dom.mjs`, yang diberi nama di `compiler/src/codegen/web/mod.rs:19-20`, sehingga runner tidak butuh dependensi luar |
| manifest | `compiler/src/codegen/web/bundle.rs:5` menulis `module.tests`, sehingga `program.json` mencantumkan nama test dan action-nya |
| validasi | `TEST_ACTIONS` di `compiler/src/semantic/events.rs:25`; action lain ditolak dengan `unknown-test-action` sebelum apa pun dijalankan (`compiler/src/semantic/checker.rs:220-239`) |
| CLI | `Command::Test` didispatch di `cli/src/command/mod.rs:66` ke `test::run()` di `cli/src/command/test.rs:12`, lalu menjalankan `tests.mjs` hasil generate melalui Node (`run_node()` di `:20`) dan meneruskan exit code-nya, sehingga program yang gagal membuat build tetap hijau. |
| test | `fixtures/valid/counter-test.plt`, serta test di `tests/cli/main.rs` yang mencakup artifact, manifest, action tak dikenal, exit code, isolasi antar test, scaffold `new` yang lulus tesnya sendiri, `format`, dan `dev` |

Setiap test di-mount ulang di atas target baru, sehingga satu test tidak
mewarisi state test sebelumnya. Action `click` mencocokkan node pada teks yang
dibaca orang (`compiler/src/codegen/web/tests.rs:118-122`) dan memilih match
terdalam, karena sebuah wrapper memiliki teks yang sama dengan isi di
dalamnya. Expectation ditulis ulang lewat `scope_for()` yang diimpor di
`compiler/src/codegen/web/tests.rs:9` dan dipakai di `:40`, lalu dieksekusi
sebagai thunk, bukan lewat `eval`.

Runner memakai `process.exitCode` di `compiler/src/codegen/web/tests.rs:111`,
bukan `process.exit`, supaya output tidak terpotong saat stdout berupa pipe.

**`tests/web/` sekarang berjalan di dalam `cargo test`.** Kedelapan harness
mengimpor `installDom` dari `dom.mjs` di direktori build masing-masing, sehingga
tidak ada lagi salinan shim di `tests/web/`. Kedelapannya tidak perlu dijalankan
manual setelah `plt build`; `tests/web/main.rs` melakukan build (termasuk import
transitif untuk showcase) dan menjalankannya. Yang menguji program `.plt` milik
pengguna lewat permukaannya sendiri adalah `plt test`.

### 4.10 Phase 9 - Optimization

Satu dari enam item berubah dari nol: **batching**.

| Item PRD | Status | Bukti |
| --- | --- | --- |
| fine-grained rendering | **BELUM** | `refresh()` di `compiler/src/codegen/web/javascript.rs:1372-1396` memanggil `instance.render()` yang membangun ulang seluruh virtual tree komponen, lalu mendiff seluruhnya. Informasi dependensi terkumpul (`plt.computed()` di `:246`), tetapi tidak pernah dipakai untuk membatasi render ke subnode yang berubah. Flag `busy` (`:1379`) dan `pending` (`:1380`) adalah penjaga reentrancy dan antrean lanjutan, bukan dirty-tracking per subtree. |
| batching | **SEBAGIAN** | Satu dispatch DOM adalah satu transaksi. `beginBatch()`/`endBatch()`/`transactional()` di `compiler/src/codegen/web/javascript.rs:194-224`, dirty set `dirtied` di `:192`, dan `refresh()` yang menunda render saat `batchDepth > 0` di `:1374-1378` lalu drain set sampai kosong di `:207-211`. Batasnya benar-benar per-dispatch karena seluruh callback pada satu event berbagi satu listener DOM di `subscribe()` (`:1079-1106`). Bukti eksekusi: `tests/web/ui.mjs` asserting bahwa satu `input` tidak menulis apa pun ke `contenteditable` yang sedang diketik. **Yang belum ada:** belum ada `queueMicrotask` atau `requestAnimationFrame` yang menggabungkan beberapa dispatch berbeda menjadi satu render, jadi dua `click` dalam satu tick masih dua render; dan `endBatch()` tetap sinkron, yang tidak apa-apa untuk runner tetapi tidak sesuai dengan coalescing berbasis frame. Sisi host `runtime/src/state.rs` masih memakai pola fan-out sinkron yang sama tanpa transaksi. |
| virtualization | **BELUM** | `plt.each()` di `compiler/src/codegen/web/javascript.rs:885-893` melakukan `Array.from(list ?? [])` lalu `.map()`, yaitu mematerialisasi seluruh item tanpa windowing, tanpa `IntersectionObserver`, tanpa overscan. `page()` di `:728-736` memangkas daftar sebelum dirender, tetapi itu windowing yang diminta program, bukan virtualisasi transparan. |
| code splitting | **BELUM** | `compiler/src/codegen/web/mod.rs:11-23` menghasilkan 6 artefak dengan nama tetap dari satu `IrModule`, dan `compiler/src/codegen/web/html.rs:15` menulis satu tag script. `module.imports` diturunkan di `compiler/src/ir/lower.rs` dan diverifikasi di `compiler/src/ir/verifier.rs`, tetapi tidak dibaca modul codegen mana pun: loader `import` (B7) bekerja di level AST, sebelum IR. |
| tree shaking | **BELUM** | `render_component()` di `compiler/src/codegen/web/javascript.rs` menulis seluruh komponen ke objek global secara tanpa syarat, tanpa analisis keterjangkauan. |
| production optimization | **BELUM** | Tidak ada jalur minify, release, atau produksi di mana pun. `compiler/src/codegen/web/html.rs:9` menulis meta viewport secara tanpa syarat tanpa pemisahan dev dan produksi. Satu-satunya konfigurasi optimasi di repository adalah `[profile.release]` di `Cargo.toml`, yang mengoptimasi binary compiler, bukan output yang dihasilkan. |

Catatan: `compiler/src/codegen/web/bundle.rs` bukan bundler. Satu-satunya
fungsi publiknya, `manifest()`, menulis JSON metadata dengan pemformatan string.
File tersebut tidak menggabungkan, tidak memampatkan.

---

## 5. Blocker

Tujuh blocker ditutup, B1 sampai B7.

### B1 - `responsive { }` tidak pernah menghasilkan `@media`

**SELESAI.** Emerson dan parser kini memakai satu kosakata breakpoint,
`mobile`/`tablet`/`desktop`, `breakpoint_of()` menjadikan `mobile` sebagai
base, dan `tablet` (`48rem`) serta `desktop` (`64rem`) menghasilkan `@media`.
Kelompok yang tidak dikenal tidak lagi dilewati diam-diam. Element swap per
breakpoint dihapus dan `mobile: Column { }` ditolak parser, tertutup
`tests/parser/main.rs`.

### B2 - `Container` tidak ada di permukaan bahasa

**SELESAI.** Terdaftar di registry builtin, dipetakan ke `div`, mendapat base
`display: block`, dan class instance seperti primitive layout lain.

### B3 - Keempat lifecycle event tidak pernah fire

**SELESAI.** Keempatnya dipanggil dari runtime JS pada titik siklus hidup yang
sesuai, dengan call site yang tercatat di bagian 4.5. Penulisan state di dalam
`mount` tidak hilang karena `mountTree()` mengantar render lanjutan lewat
antrean `pending`.

### B4 - `gap` dan `columns` jadi atribut HTML, bukan CSS

**SELESAI.** Keduanya dipetakan ke kanal CSS.
`compiler/src/semantic/element.rs:216-233` mendaftarkan 16 properti bersama
untuk 8 primitive layout, `PROPERTY_EXCEPTIONS` di
`compiler/src/codegen/style/mod.rs:13-17` menamai `columns` sebagai
`grid-template-columns`, dan `css_value()` di `:66-73` mengembangkannya jadi
`repeat(3, minmax(0, 1fr))`. Panjang polos mendapat satuan `px`, dan class
dipisah per instance element sehingga override tidak bleduh antar element.

### B5 - Empat dokumen yang dirujuk README belum ada

**BELUM.** Dua revisi README sebelumnya menutup ERROR yang paling menyesatkan,
yaitu link yang menunjuk ke file yang tidak ada. `README.md:110-116` kini hanya
merujuk tiga dokumen yang benar-benar ada, yaitu `docs/prd.md`,
`docs/struktur.md`, dan `docs/roadmap.md`, dan ketiganya memang ada di
`docs/`. Bagian Build Output juga sudah diganti dengan enam artefak yang
memang dihasilkan (`README.md:68-74` cocok dengan
`compiler/src/codegen/web/mod.rs:13-21`), bukan nama `bootstrap.js`,
`runtime.wasm`, dan `program.bin` seperti yang pernah diklaim.

Yang tersisa hanyalah tiga dokumen yang tidak pernah ada dan sekarang tidak
Dirujuk dari mana pun: `docs/syntax.md`, `docs/architecture.md`, dan
`docs/traceability.md`. Direktori `docs/` hanya berisi `prd.md`,
`struktur.md`, dan `roadmap.md`. Ditunda eksplisit: ketiganya adalah pekerjaan
dokumentasi, bukan blocker kode, dan tidak ada yang rusak sekarang.

### B6 - Runner `test` tidak ada

**SELESAI.** `Command::Test` membangun lalu menjalankan `tests.mjs` hasil
generate melalui Node, meneruskan exit code, dan melompat sendiri bila Node
tidak ada saat `cargo test`. `module.tests` masuk ke `program.json`, action
divalidasi oleh `TEST_ACTIONS`, dan setiap test di-mount ulang agar tidak
mewarisi state test sebelumnya.

### B7 - Module loader `import`

**SELESAI.** `import Name from "./file.plt"` sekarang memuat file, bukan
sekadar dikompilasi lalu dibuang. Semantiknya adalah *library module*: file
yang diimpor tidak boleh mendeklarasikan `app`, dan deklarasi komponen, style,
theme, api, dan test-nya di-inline ke scope global program, dirujuk dengan nama
telanjang seperti deklarasi lokal.

Rantai loader ada di `compiler/src/loader.rs`:

1. `resolve_and_combine()` di `compiler/src/loader.rs:76` membaca entry lalu
   setiap `import` secara DFS, dengan resolusi path relatif terhadap direktori
   file yang mendeklarasikannya, dan identitas berbasis path canonical;
2. tiap modul diverifikasi sambil dimuat: modul tidak boleh punya `app`
   (`import-module-has-app`), label impor unik (`duplicate-import`), satu path
   tidak dimuat dua kali (`duplicate-module`), nama deklarasi level atas tidak
   boleh bentrok dengan apa yang sudah dimuat (`import-collision`), dan cycle
   ditolak (`import-cycle`);
3. sumber semua file digabung ke satu buffer (entry + `"\n"` + tiap modul dalam
   urutan DFS), lalu pipeline normal (`analyse` di `pipeline.rs:196` +
   `finish` di `:136`) mengompilasi buffer itu sekali, sehingga span diagnostic
   menunjuk ke `SourceFile` gabungan yang sumber aslinya tetap terbaca;
4. kompilasi di-inject: compiler tetap murni. `pipeline::build_file`
   (`pipeline.rs:61`) menolak `import` yang tidak dimuat dengan
   `import-not-loaded`, sedangkan `pipeline::build_entry` (`pipeline.rs:101`)
   menerima `&dyn Loader` dan mengembalikan
   `Loaded { compilation, file, dependencies }` (`pipeline.rs:76`);
5. `FsLoader` (`loader.rs:37`) adalah satu-satunya implementasi
   `Loader` (`loader.rs:31`); `ReadError` di `loader.rs:19` membedakan file
   hilang (`import-not-found`) dari error I/O lain (`import-read-error`).

`cli/src/command/build.rs:27` memakai `build_entry` dengan `FsLoader`, sehingga
`build`, `run`, `test`, dan `dev` semuanya memuat `import`. `dev` memakai
`Loaded.dependencies` sebagai daftar yang dipantau (`cli/src/command/dev.rs:86-93`),
jadi mengedit file yang diimpor memicu rebuild.

Path import kosong tetap ditangani verifier IR (`empty-import-path` di
`compiler/src/ir/verifier.rs:61`), sehingga `fixtures/invalid/empty-import-path.plt`
tetap lolos. `WarningKind::UnusedImport` di `compiler/src/diagnostics/warning.rs:8`
masih belum diinstansiasi: impor yang tidak pernah dipakai tidak diperingatkan.

Bukti eksekusi pada audit ini:

```text
$ plt check import.plt            # app Main { Card { } } + ui.plt mendeklarasikan component Card
checked import.plt: 2 components, 0 apis      # exit 0; nama Card terselesaikan lewat import

$ plt build import.plt -o out
# app.js berisi "function Card(" dan app.css berisi style dari ui.plt

$ plt check broken.plt            # ui.plt tidak ada
error: semantic[import-not-found]: cannot find `./ui.plt` imported as `Card`
  --> app.plt:1:1
   |
   import Card from "./ui.plt"
   ^~~~~~~~~~~~~~~~~~~~~~~~~~~
```

Semantic checker memperingatkan `import` tanpa ekstensi `.plt` di
`compiler/src/semantic/checker.rs:203-218` seperti sebelumnya, dan loader tetap
memuat file itu; warnanya memang hanya konvensi, bukan larangan.

---

## 6. Backlog Prioritas

### P0 - Blocker

- [x] B1 samakan kosakata breakpoint dan hentikan `continue` diam-diam
- [x] B2 daftarkan primitive `Container`
- [x] B3 implementasikan keempat lifecycle event
- [x] B4 petakan `gap` dan `columns` ke CSS, bukan atribut HTML
- [ ] B5 tulis `docs/syntax.md`, `docs/architecture.md`, dan `docs/traceability.md` (ditunda)
- [x] B6 tambahkan runner `test` beserta `Command::Test`
- [x] B7 module loader untuk `import` (library module: di-inline, tanpa `app`, nama = label unik)

### P1 - Cakupan MVP dan kebenaran dokumentasi

- [x] `platipus new <nama>` untuk scaffold proyek
- [x] `platipus dev` dengan file watcher dan hot reload (memantau entry dan semua file `import`)
- [x] `platipus format`
- [x] `platipus new` menulis template yang sudah terformat: hasil `new` sama persis dengan
      hasil `format` atas file yang sama. Perbaikannya ada di template
      `cli/src/command/new.rs:7`, bukan di formatter, karena aturan formatter
      memang membuang baris kosong di sebelah brace; ditutup
      `the_scaffolded_project_is_already_formatted`
- [x] Isi `fixtures/valid/` dan `fixtures/invalid/` — sekarang 2 file valid dan 19 file invalid
- [x] Sambungkan `tests/web/*.mjs` ke target `[[test]]` agar pengujian perilaku berjalan di CI
- [x] Tambahkan test yang menutup B1 sampai B4 dan B6
- [x] Hapus `cli/dist/` dan artefak hasil build yang basi di `tests/web/`; sekarang `tests/web/` berisi 8 harness, `main.rs`, dan 6 fixture `.plt`
- [x] Perbaiki klaim `README.md`
- [x] `cargo clippy --workspace --all-targets` bersih, nol warning
- [ ] Jalankan `cargo fmt --all` untuk 19 file di `cli/`, `runtime/`,
      `standard/`, dan `tests/` yang masih menyimpang dari `rustfmt`
      (`compiler/` sudah bersih). Ditunda eksplisit: tidak ada version control
      di repository ini, jadi diff sebesar itu tidak dapat ditinjau atau
      dibalikkan, dan pekerjaan ini tidak mengubah perilaku apa pun. Perlu
      keputusan eksplisit.

### P2 - Fase berikutnya

- [x] Phase 6: `fetch`, clipboard, file, WebSocket, serta storage yang lengkap
- [x] Phase 7: `Canvas`, `Editor`, `CodeEditor`, `DataGrid`, dan `Tree` (lih. 4.8).
      Sisa yang jelas: path/arc pada canvas, undo dan keymap editor, gutter dan
      completion `CodeEditor`, sorting klik-kolom dan virtualisasi `DataGrid`,
      indentasi guide dan lazy load `Tree`, serta pinch/spring
- [x] Phase 2: lengkapi 20 primitive yang hilang dan properti bagi elemen yang kosong (lih. 4.3)
- [x] Phase 4: event propagation dan event object yang kaya (lih. 4.5)
- [x] Phase 5: posisi scroll sebagai state reaktif (binding `scrollTop`/`scrollLeft` + `on scroll`/`event.position`, lih. 4.6)
- [ ] Phase 0: type checking, karena saat ini nol
- [x] Phase 9: batching, yaitu satu dispatch = satu transaksi (lih. 4.10)
- [ ] Phase 9: fine-grained rendering, virtualisasi, code splitting, tree shaking,
      dan jalur produksi, semuanya masih nol
- [ ] Runtime: coalescing lintas dispatch dengan `queueMicrotask` atau
      `requestAnimationFrame`. Batching yang ada berhenti di satu dispatch, jadi
      dua event dalam satu tick masih dua render (lih. 4.10)
- [ ] Runtime sisi host `runtime/src/state.rs` memakai pola fan-out sinkron tanpa
      transaksi, jadi tidak konsisten dengan runtime web yang sudah memakai
      `beginBatch()`/`endBatch()`

---

## 7. Deviasi Dokumentasi

### 7.0 Angka dan status yang berubah pada revisi 2026-09-29

| Klaim lama | Kenyataan sekarang |
| --- | --- |
| 281 test lulus | **305** lulus, 0 gagal. Snapshot sekarang: `codegen` 64, `cli` 31, unit test `platipus-compiler` 40. |
| 5 harness web, 41 assertion | **8** harness, **68** check. `showcase.mjs` menguji aplikasi showcase beserta import dan interaksinya. |
| Phase 7 **BELUM**, "tidak ada `getContext` di seluruh workspace" | Phase 7 **SEBAGIAN**. `getContext` ada di `compiler/src/codegen/web/dom.rs:175`, dan sepuluh builtin Phase 7 ada di `compiler/src/codegen/web/javascript.rs`. |
| Phase 9 "6 dari 6 item bernilai nol" | **1 dari 6** berubah: batching **SEBAGIAN**. |
| "Flag `busy` adalah penjaga reentrancy, bukan batching" | Benar untuk `busy`, tetapi tidak lagi untuk fase: kini ada dirty set dan batas dispatch (lihat 4.10). |
| `event object` = `{ value, key, position, ... }` | Sekarang juga `data`, untuk `on drop` (lihat 4.5). |
| `applyEvents()` memasang satu listener per handler | Sekarang satu listener DOM per (event, fase), dipakai bersama oleh semua handler dan bind pada node itu (lihat 4.5). |
| warning `clippy::module_inception` di `compiler/src/codegen/web/tests.rs` | Ditutup; clippy bersih total. |
| `cli/dist/` dibuat ulang oleh `cargo test` | Ditutup; test memakai `-o` ke direktori scratch. |
| `platipus new` menulis template belum terformat | Ditutup; template dan hasil `format` identik. |
| Direktori `docs/` hanya berisi tiga dokumen | Tetap benar (`prd.md`, `struktur.md`, `roadmap.md`). |

Satu klaim README ikut dikoreksi pada revisi ini, jadi tabel di atas tidak lagi
boleh dibaca sebagai "tidak ada yang tersisa": bagian Status README menyatakan
bahwa "Phase 6 browser APIs" dan "most of the Phase 7 elements" belum nyata.
Keduanya sudah basi. `README.md:120-129` kini menyebut keduanya sebagai yang
sudah bekerja, dan menggantinya dengan kekurangan yang memang masih ada, yaitu
type checking, LSP, debugger, fine-grained rendering, dan output produksi.

Semua `path:line` ke `compiler/src/codegen/web/javascript.rs` dihitung ulang
karena file itu tumbuh sekitar 150 baris. Yang tidak dihitung ulang dan masih
mewarisi nomor baris versi sebelumnya adalah referensi ke
`compiler/src/parser/`, `compiler/src/semantic/`, `compiler/src/ir/`,
`compiler/src/ast/`, `compiler/src/diagnostics/`, `cli/`, `tests/`, dan
`standard/`: file-file itu tidak disentuh revisi ini. `compiler/src/semantic/element.rs`
juga tidak disentuh, kecuali dicatat bila berbeda.

### 7.1 README diverifikasi ulang pada 2026-09-28

Versi dokumen sebelumnya mencatat empat klaim README yang tidak cocok dengan
kode. Keempatnya sudah diperbaiki, dan audit ini memverifikasi ulang hasil
perbaikannya:

| Klaim lama | Kenyataan sekarang |
| --- | --- |
| README menyebut 7 command; ada 6 | Salah hitung di kedua arah. `cli/src/command/mod.rs:17-27` punya **9** command, yaitu `new`, `build`, `check`, `run`, `test`, `dev`, `format`, `help`, dan `version`. `README.md:93-101` mendokumentasikan 7 yang utama; `help` dan `version` memang absen dari daftar itu, dan itu tidak salah karena keduanya sudah tercakup di `README.md:104-105`. |
| README menyebut output `bootstrap.js`, `runtime.wasm`, `program.bin` | Benar. `compiler/src/codegen/web/mod.rs:13-21` menghasilkan `index.html`, `app.css`, `app.js`, `dom.mjs`, `tests.mjs`, `program.json`, persis seperti pohon di `README.md:68-74`. Tidak ada target WASM. |
| README/runtime bertanggung jawab atas Element, State, Event, Reactive, Style, Layout, Render, Browser | Benar. `runtime/src/` berisi 4 file datar tanpa kode DOM, HTTP, maupun browser, dan `README.md:59-63` sekarang menyatakan itu secara eksplisit. |
| README merujuk 4 dokumen | Benar. `README.md:110-116` hanya merujuk tiga dokumen yang ada. |

Tidak ada klaim tersisa di README yang perlu dikoreksi pada audit ini.

### 7.2 `docs/struktur.md` adalah arsitektur target

Dokumen itu ditulis dalam suara normatif, misalnya "Struktur yang diinginkan"
dan "Contoh yang harus dihindari", sehingga **bukan** deskripsi repository saat
ini. Deviasi yang terverifikasi:

* `runtime/src/` dideskripsikan dengan 8 direktori anak di
  `docs/struktur.md:62-70`, sedangkan realitasnya 4 file datar, yaitu
  `lib.rs`, `state.rs`, `derived.rs`, dan `event.rs`;
* `compiler/src/codegen/` memang 7 direktori di `docs/struktur.md:304-310`,
  dan itu cocok, tetapi setiap direktori hanya punya satu `mod.rs`, kecuali
  `web/` yang punya 7 file. Pola "satu modul per fitur" yang tidak ada di
  mana pun;
* `compiler/src/semantic/` dideskripsikan berbeda di `docs/struktur.md:280-290`:
  6 file yang disebut tidak ada, yaitu `symbol.rs`, `type_check.rs`,
  `component.rs`, `state.rs`, `expression.rs`, dan `diagnostics.rs`, sementara
  4 file yang ada tidak disebut, yaitu `checker.rs`, `events.rs`,
  `name_resolver.rs`, dan `types.rs`;
* `docs/struktur.md:126-138` menyebut 12 dokumen di `docs/`, sedangkan yang ada
  hanya `prd.md` dan `struktur.md`, ditambah `roadmap.md` yang justru tidak
  disebut di sana. `roadmap.md` maupun `traceability.md` tidak disebut sama
  sekali di `docs/struktur.md`;
* `examples/` disebut berisi 5 subdirektori di `docs/struktur.md:170-175`,
  sedangkan isinya `examples/counter.plt` dan `examples/showcase/` dengan
  shell, library, dan modul-modul section;
* `tests/` disebut berisi 12 subdirektori di `docs/struktur.md:156-168`,
  sedangkan realitasnya 9 file `main.rs` datar.

Satu bagian justru cocok persis: `compiler/src/diagnostics/` di
`docs/struktur.md:312-316` menyebut 4 file, dan memang ada 4.

Deviasi yang tercatat versi dokumen sebelumnya sudah tidak berlaku lagi:
`cli/src/command/`, `cli/src/config/`, dan `cli/src/output/` kini ada, dengan
`command/` memuat 8 file, dan `tests/` kini punya target `[[test]]` yang
benar-benar menjalankan program `.plt`. Struktur ini sengaja dibiarkan apa
adanya dan dicatat di sini, agar `docs/struktur.md` dibaca sebagai
*"kemakuraan tujuan"* dan bukan sebagai laporan kondisi saat ini.

### 7.3 Artefak hasil build di working tree

``cli/dist/` tidak pernah ada di commit karena repository ini tidak berada di
dalam git, dan `.gitignore` sudah mengabaikan `dist/` di setiap level. Yang
perlu diluruskan dari versi dokumen sebelumnya adalah asal-usulnya, karena
penyebabnya bukan build manual:

* Klaim lama bahwa `cli/dist/app.js` masih memakai
  `for (const vnode of newList)` sudah basi. Artefak itu sudah memakai iterasi
  mundur `for (let index = newList.length - 1; ...)` di baris 161, sama dengan
  `patchList()` di `compiler/src/codegen/web/javascript.rs:920`.
* Klaim bahwa `cli/dist/` "ikut ter-commit" tidak dapat dipertahankan.
* **`cli/dist/` dibuat ulang oleh `cargo test` sendiri.** Penyebabnya
  `quiet_mode_prints_nothing_on_success` di `tests/cli/main.rs`. Helper `run()`
  hanya memasang `current_dir` bila argumen keenamnya `Some`, dan test itu
  memanggilnya tanpa `-o`, sehingga `plt build` mewarisi direktori kerja proses
  test, yaitu root crate `cli`, lalu menulis ke `out_dir` default `dist`.

**Closure ini selesai pada revisi ini.** Test tersebut kini diberi `-o` yang
menunjuk direktori scratch, jadi ia menulis ke sana dan tidak lagi menyentuh
working tree; sekaligus ia sekarang ikut assert bahwa output-nya benar-benar
dibuat, sehingga `-o` yang diabaikan tidak akan lolos. `cli/dist/` dari run
sebelum revisi ini sudah dihapus.

Empat test lain yang dulu memanggil `run()` tanpa `cwd` tidak pernah menulis:
`--check` tidak menulis, kasus yang gagal dikompilasi tidak menulis, dan kasus
yang file-nya tidak ditemukan tidak menulis. Test yang lain memakai
`run_in(Some(&dir), ...)` dan menulis ke direktori sementara.

Catatan untuk audit berikutnya: direktori `dist` di **root** repository masih
ada dan berisi program `Main`/`Card`. Asalnya tidak dapat dikaitkan dengan test
yang terdaftar di bagian 4.2, jadi ia diperlakukan sebagai artefak manual, bukan
artefak `cargo test`.

### 7.4 Keterbatasan lain

* `compiler/Cargo.toml` memiliki `[dependencies]` kosong, sehingga compiler
  tidak punya satu pun dependensi pihak ketiga;
* **`cargo clippy --workspace --all-targets` sekarang benar-benar bersih**, nol
  error dan nol warning. Warning `clippy::module_inception` yang tercatat di
  versi dokumen sebelumnya sudah ditutup: `mod tests` di dalam
  `compiler/src/codegen/web/tests.rs` dihapus, dan test-nya dipindah ke level
  modul itu sendiri dengan `#[cfg(test)]` pada helper `render_source`, karena
  `render()` di `:12` adalah kode produksi yang tetap harus dikompilasi di luar
  test. `mod tests` di `compiler/src/codegen/web/dom.rs` tetap ada dan tidak
  warned, karena ia berada di dalam modul `dom`, bukan `tests`;
* **`cargo fmt --all --check` tidak bersih, dan itu kondisi lama.** 57 diff
  spread across 19 file, semuanya di `cli/`, `runtime/`, `standard/`, dan
  `tests/`. Repository ini karena itu tidak boleh dianggap sudah `rustfmt`-clean.
  Revisi ini **tidak** menjalankan `cargo fmt --all`, karena di repository tanpa
  version control diff sebesar itu tidak dapat ditinjau atau dibalikkan dan
  tidak mengubah perilaku apa pun; sisanya dicatat sebagai utang di bagian 6.
  Efek samping yang perlu diketahui: `compiler/` kini sudah `rustfmt`-clean,
  karena `cargo fmt -p platipus-compiler` yang dimaksudkan untuk satu file
  ternyata memformat seluruh package, bukan hanya file tersebut. `rustfmt`
  menjamin semantik tidak berubah, dan `cargo test --workspace` (305 lulus)
  serta `cargo clippy --workspace --all-targets` (bersih) tetap hijau
  sesudahnya, tetapi diff itu tidak pernah diminta dan tidak pernah ditinjau;
* `cli/Cargo.toml` hanya dependensi `platipus-compiler`. `platipus-runtime` dan
  `platipus-standard` dicoret karena tidak ada satu pun `use` di `cli/src/` atau
  `tests/`. Keduanya tetap anggota workspace, jadi tetap dibangun dan diuji;
* `fixtures/valid/` berisi 2 file, `fixtures/invalid/` berisi 19 file. Nama
  file pada `fixtures/invalid/` adalah harapan diagnostic-nya, jadi
  `tests/fixtures/main.rs` bisa memverifikasi setiap file sebagai
  self-describing test;
* `examples/` berisi `examples/counter.plt` dan showcase modular di
  `examples/showcase/` (`main.plt`, `widgets.plt`, `design.plt`, serta
  `sections/*.plt`).

---

## 8. Acceptance Criteria PRD

Status tiap acceptance criteria pada `docs/prd.md` bagian 71:

| Area | Status | Catatan |
| --- | --- | --- |
| Syntax | **SEBAGIAN** | syntax core diparse dan error struktural jelas, tetapi error salah tipe tidak ada karena tidak ada type checking |
| Element | **SELESAI** | primitive dirender, property, children, dan style bekerja; seluruh 47 primitive PRD plus 15 ekstra terdaftar dengan 46 nama properti (lihat 4.3) |
| State | **SELESAI** | state dibuat, berubah, UI bereaksi, dependensi terlacak |
| Event | **SELESAI** | event object ternormalisasi (`event.value`/`key`/`position`/`data`), custom event, keempat lifecycle event, dan propagation (`stopPropagation` + delegasi capture) bekerja (lihat 4.5) |
| Component | **SELESAI** | input, state, emit, dan reusable component bekerja |
| Layout | **SELESAI** | layout dasar, `responsive { }` ke `@media`, seluruh primitive layout PRD (termasuk `Panel`, `Splitter`, `Spacer`, `Viewport`) bekerja, dan posisi scroll menjadi state reaktif lewat `on scroll`/`event.position` maupun bind `scrollTop`/`scrollLeft` (lihat 4.6) |
| Runtime | **SEBAGIAN** | aplikasi dapat dijalankan di browser, dan satu dispatch DOM kini menghasilkan satu render, tetapi render masih mencakup seluruh virtual tree komponen sehingga kriteria "tidak melakukan rerender global yang tidak diperlukan" belum sepenuhnya terpenuhi (lihat 4.10) |

Dari 7 area, 5 terpenuhi, 2 sebagian, dan 0 belum. Element naik ke
**SELESAI** pada audit sebelumnya setelah 20 primitive yang hilang dan properti
yang kosong diimplementasikan (lihat 4.3); Event naik ke **SELESAI** pada audit
sebelumnya setelah event object dan propagation diimplementasikan (lihat 4.5);
Layout naik ke **SELESAI** pada audit sebelumnya setelah posisi scroll menjadi
state reaktif (lihat 4.6). Runtime tetap **SEBAGIAN**: revisi ini menutup
batas dispatch sehingga satu `input` tidak lagi memicu dua render, tetapi
render masih penuh dan belum ada coalescing lintas dispatch, jadi kriteria itu
belum terpenuhi.

---

## 9. Cara Memverifikasi Ulang

Status pada dokumen ini perlu disegarkan setiap beberapa waktu. Langkah audit
yang dipakai:

1. **Daftar subtree tanpa ikut `target/`** untuk melihat file yang benar-benar
   ada, bukan yang dijanjikan dokumen:

   ```powershell
   Get-ChildItem -Recurse -File -Include *.rs,*.mjs,*.plt |
     Where-Object { $_.FullName -notmatch '\\target\\' }
   ```

2. **Cari marker pekerjaan tertunda:**

   ```powershell
   Get-ChildItem -Recurse -File -Include *.rs |
     Where-Object { $_.FullName -notmatch '\\target\\' } |
     Select-String -Pattern 'TODO|FIXME|XXX|HACK|unimplemented!|todo!\('
   ```

   Audit terakhir: nol hasil.

3. **Periksa klaim negatif**, yaitu item bernilai nol. Klaim ini paling mudah
   menjadi salah karena ketiadaan sulit dilihat mata. Scan negatif untuk Phase 6
   sudah tidak berlaku sejak `fetch\s*\(|WebSocket` benar-benar ada di
   `compiler/src/codegen/web/javascript.rs`; scan untuk Phase 7 juga sudah tidak
   berlaku, karena `getContext` dan `requestAnimationFrame` kini ada di shim
   `compiler/src/codegen/web/dom.rs:175` dan `:282`. Scan yang masih berlaku
   hanya untuk sisa yang memang nol:

   ```powershell
   Get-ChildItem -Recurse -File -Include *.rs,*.mjs,*.plt |
     Where-Object { $_.FullName -notmatch '\\target\\' } |
     Select-String -Pattern 'matchMedia|requestIdleCallback|IntersectionObserver|minify'
   ```

   Audit terakhir: nol hasil untuk keempat marker itu. `globalThis.innerWidth`
   di `compiler/src/codegen/web/dom.rs:271` tetap konstanta stub yang tidak
   mengukur apa pun.

4. **Hitung klaim angka**, jangan percaya angka yang sudah tertulis di
   dokumen ini. Tiga klaim pernah salah di versi sebelumnya: jumlah primitive
   PRD, jumlah builtin di kode, dan jumlah elemen yang punya properti. Audit ini
   juga memasang guard otomatis yang memotret 62 builtin dan 46 nama properti:
   `catalog_primitives_are_registered` dan
   `catalog_covers_the_documented_builtin_surface` di `tests/semantic/main.rs`:

   ```powershell
   # primitive PRD: 47
   $p = Get-Content docs\prd.md
   $s = ($p | Select-String '^# 26\. ').LineNumber
   $e = ($p | Select-String '^# 27\. ').LineNumber
   ($p[($s-1)..($e-2)] | Where-Object { $_ -match '^[A-Z][A-Za-z]+$' }).Count

   # builtin di kode: 62
   $c = Get-Content compiler\src\semantic\element.rs
   $a = ($c | Select-String 'pub const ALL').LineNumber
   $b = ($c | Select-String '^\s*\];').LineNumber | Where-Object { $_ -gt $a } | Select-Object -First 1
   ($c[($a-1)..($b-1)] | Where-Object { $_ -match '^\s*\("' }).Count
   ```

   Untuk elemen yang punya properti, hitung langsung dari registry, jangan
   mendaftarnya ulang dengan tangan: 62 builtin semuanya membawa properti, dan
   46 nama properti berbeda, dijamin oleh
   `catalog_covers_the_documented_builtin_surface` di `tests/semantic/main.rs`.
   Enam belas properti bersama untuk elemen berbox (kategori layout plus
   `BOX_ELEMENTS`) berasal dari `LAYOUT_PROPERTIES` di
   `compiler/src/semantic/element.rs:277-294`; nama yang sama (misalnya `width`
   pada `Canvas`) tidak dihitung dua kali oleh guard karena ia mengumpulkan ke
   `BTreeSet`.

5. **Jalankan test suite yang terdaftar** untuk memastikan tidak ada regresi,
   lalu cocokkan jumlahnya dengan bagian 4.2:

   ```powershell
   cargo test --workspace
   ```

  Audit terakhir: 305 lulus, 0 gagal, 19 test binary.

6. **Buktikan harness web benar-benar jalan, bukan skip.** `cargo test`
   membuang output harness, jadi jalankan manual kalau butuh bukti:

   ```powershell
   cargo run -p platipus-cli --bin plt -- build examples\counter.plt -o out
   node tests\web\run.mjs out
   ```

  Kedelapan harness juga menolak berjalan tanpa direktori build, exit 2, supaya
  tidak pernah membaca artefak yang tertinggal. Audit terakhir: 10, 9, 11, 5,
  8, 7, 9, dan 9 check lulus. `events.mjs` menguji event object dan binding
  `scrollTop`; `runtime.mjs` menguji fetch, clipboard, file, WebSocket, storage,
  serta jalur file picker fallback; `ui.mjs` menguji canvas, editor, code editor,
  data grid, dan tree; `components.mjs` menguji input, roots, branch, swap, dan
  disposal; `showcase.mjs` menguji form, drag/drop, Tree, editor, canvas,
  calculator, serta API browser dengan import showcase yang sesungguhnya.

7. **Periksa klaim README** setiap kali README berubah, karena bagian Build
   Output dan bagian CLI paling cepat menjadi basi. Bandingkan dengan
   `compiler/src/codegen/web/mod.rs:13-21` dan `cli/src/command/mod.rs:17-27`.

8. **Uji binary yang benar-benar didistribusikan.** `cargo test` memakai
   binary debug dan tidak menyentuh jalur `dev`. Audit terakhir memakai
   `cargo build --release` lalu:

   ```powershell
   target\release\platipus.exe new demo
   target\release\platipus.exe build demo\app.plt -o out
   target\release\platipus.exe check demo\app.plt        # "1 components, 0 apis"
   target\release\platipus.exe test  demo\app.plt        # "2 passed, 0 failed"
   target\release\platipus.exe run   demo\app.plt        # hanya mencetak cara menyajikan
   target\release\platipus.exe build nope.plt            # "cannot find", exit 1
   ```

   Hasil audit terakhir: keenam perintah berperilaku seperti yang didokumentasikan,
   `build` menulis 6 artefak, `check -q` diam, dan `build` atas file yang tidak
   ada keluar dengan kode 1 serta pesan yang jelas. Audit B7 menambah smoke:
   `build`/`check` atas `app.plt` yang mengimpor `ui.plt` dan mendapati
   `function Card(` di `app.js` serta style dari `ui.plt` di `app.css`.

9. **`dev` diuji manual**, karena tidak ada test yang menjalankan
   `plt test` terhadap build produksi. Sejak B7 ada `dev_rebuilds_when_an_imported_file_changes`
   di `tests/cli/main.rs` yang mengedit file yang diimpor dan menunggu sinyal
   reload, jadi bagian "perubahan memicu rebuild" sudah otomatis. Audit terakhir
   menyalakan `dev app.plt -p 8138`, memastikan `/` dan `/app.js` sama-sama 200,
   `/app.plt` ditolak, lalu menyentuh entry file dan melihat `rebuilt app.plt`
   dicetak. Perilaku `run` yang tidak menyajikan apa pun ikut terverifikasi
   di titik yang sama.

10. **Bersihkan artefak terakhir.** `cargo test` tidak lagi menulis direktori
    mana pun ke working tree: `quiet_mode_prints_nothing_on_success` sudah
    diberi `-o` ke direktori scratch (lihat bagian 7.3), jadi `cli/dist` tidak
    muncul lagi. Audit terakhir mengonfirmasi `cli/dist` tidak ada. Direktori
    `dist` di root repository masih ada dan berisi program `Main`/`Card` yang
    tidak berasal dari test mana pun yang terdaftar; perlakukan sebagai artefak
    manual dan hapus hanya jika memang tidak dikehendaki.
