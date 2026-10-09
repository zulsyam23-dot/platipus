# Catatan pekerjaan yang belum selesai

Diperbarui: 2026-10-09

## Kondisi yang sudah diverifikasi

- `cargo clippy --workspace --all-targets -- -D warnings` lulus.
- `cargo test --workspace` lulus. Rincian per suite ada di bagian 5.
- Kedua hasil ini hanya memverifikasi lint dan test yang tersedia; belum
  membuktikan semua fitur bahasa atau performa produksi selesai.
- Registry HTTP dan `p2lt add github.com/owner/repository` tetap belum
  diverifikasi terhadap layanan registry publik atau repository GitHub nyata.
  Alur lokal (publish → install → import → build) sudah diuji ujung ke ujung,
  lihat bagian 1.

## 1. Bukti kemampuan komputasi murni Platipus — SELESAI

Semua butir pada prioritas lama nomor 1 sudah tertutup. Yang ditutup dan
dengan bukti apa:

- [x] Paket penerimaan `examples/computasi/` dengan `p2lt.toml`,
  `src/lib.plt`, dan `src/widgets.plt` yang diimpor relatif, tanpa `#[rust]`
  dan tanpa `app`.
- [x] Test nilai nyata untuk gcd/lcm, primalitas, Fibonacci, faktorial,
  operasi daftar, Collatz, sorting tanpa mengubah input, binary search,
  palindrome, FNV-1a, dan lambda map. Tiga belas blok `test`, masing-masing
  menyatakan angka yang benar-benar dihitung program.
- [x] Alur paket ujung ke ujung: publish ke registry lokal, install di
  aplikasi lain, impor beberapa fungsi, panggil dari `derived` dan dari click
  handler, build, lalu `node` menjalankan test aplikasi. Di
  `p2lt/tests/acceptance.rs`. Konsekuensinya menarik: konsumen mewarisi 13 test
  milik paket, jadi 16 test yang hijau di sana.
- [x] Alur GitHub: paket dipublish ke `github.com/zulsyam23-dot/lib-uji-coba-`
  lalu dipasang dari sana dengan `p2lt add github.com/zulsyam23-dot/lib-uji-coba-`.
  Di `examples/computasi-github/`. Dengan store kosong build gagal
  `import-not-found`, jadi angka-angkanya benar-benar dari repo tersebut.
- [x] Batas performa 1 juta iterasi di harness Node, di
  `tests/web/benchmark.mjs`. Baseline di mesin ini: `for i in 0..1_000_000`
  0.7 ms, `while` 0.7 ms, 100rb list + map + fold 6.7 ms, 1000 panggilan
  rekursif 2.0 ms.
- [x] Test lintas layer untuk arity, tipe argumen, kemurnian fungsi, impor
  fungsi, rentang angka, closure, dan operator bitwise. 25 test baru di
  `tests/semantic`, 6 di `tests/parser`, 6 di `tests/codegen`, 3 di
  `tests/loader`, 7 unit test di `libraries/standard`.

### Bug nyata yang ditemukan dan diperbaiki saat menulis test tersebut

Semuanya ditemukan karena test menulis nilai yang diharapkan, bukan karena
kode dibaca.

1. **Operand bitwise tidak diberi kurung.** `n & k == 1` berarti `(n & k) == 1`
   di Platipus, tetapi dipancarkan sebagai `n & k == 1`, dan JavaScript
   mengikat `==` lebih ketat dari `&`, jadi program menghitung `5 & (3 == 1)`
   = `0`, bukan `1`. Ini persis risiko yang ditulis di `docs/logic-design.md`
   tetapi mekanismenya tidak pernah dijalankan. Diperbaiki dengan
   `is_bitwise` di `libraries/ir/src/lower.rs`, dan test-nya ada di
   `tests/codegen/main.rs`.
2. **`test` blok library ikut terbawa ke konsumen, lalu gagal.** Impor
   menggabungkan sumber, jadi test milik library ikut masuk ke program
   konsumen — tetapi runner menulis ekspektasinya terhadap nama internal
   `f_gcd`, yang tidak ada di modul runner. Diperbaiki di
   `libraries/web/src/codegen/web/tests.rs`; runner sekarang menarik setiap
   fungsi tingkat atas lewat nama ekspornya. `__plt` juga diekspor tanpa syarat
   karena runner memakai `plt.*`.
3. **`fn` Platipus tidak implicit-return.** `fn square(x) { x * x }`
   menghasilkan `undefined` tanpa diagnostic. Bukan bug — dokumentasi yang
   belum ada; sekarang tertulis di `docs/guide.md`.
4. **Tidak ada builtin untuk pembagian bulat maupun akses karakter.**
   `binarySearch` menghasilkan `3.5` sebagai indeks, yang membuktikan `/`
   adalah pembagian sejati. Ditambahkan `idiv(a, b)` dan `codeAt(teks, i)`;
   keduanya diuji di kedua backend.

### Batas yang ditemukan dan sekarang terdokumentasi

- `Int` di backend web adalah `Number` JavaScript, jadi hanya bulat sampai
  `2^53 - 1` yang persis. `fib(79)` bernilai matematis
  `14472334024676221` tetapi dipancarkan `14472334024676220`. Diukur oleh test
  `integersLosePrecision`, bukan diasumsikan.
- `/` adalah pembagian sejati dan `//` tidak bisa dipakai untuk pembagian
  bulat karena `//` sudah menjadi komentar baris. `docs/logic-design.md`
  bagian 4 menjelaskannya.

## 2. Optimasi aplikasi/output web — BELUM

Roadmap bagian 4.10 mencatat hal-hal berikut belum lengkap. Tidak ada yang
dikerjakan pada putaran ini; yang dilakukan hanya menyiapkan benchmark supaya
baseline bisa dibandingkan nanti (bagian 1).

- [ ] Fine-grained rendering: perubahan state masih merender ulang seluruh
  virtual tree komponen.
- [ ] Batching lintas dispatch: perubahan hanya digabung dalam satu dispatch;
  belum ada coalescing lintas event/tick. Runtime host juga belum memiliki
  transaksi setara.
- [ ] Virtualisasi daftar besar: `each` masih memproses seluruh daftar;
  `page` bukan virtualisasi transparan.
- [ ] Code splitting: output masih satu bundel JavaScript.
- [ ] Tree shaking: komponen/fungsi yang tidak terpakai belum dihapus dari
  output.
- [ ] Mode produksi: belum ada jalur produksi/minifikasi khusus untuk artefak web.

Sebelum menyebut optimasi "super", ukur baseline dan hasil dengan
`tests/web/benchmark.mjs`, dan pastikan perubahan tidak mengubah perilaku
aplikasi.

## 3. Fitur produk lain yang belum lengkap — BELUM

Tidak ada yang dikerjakan pada putaran ini.

- [ ] Lengkapi fitur Advanced UI yang dicatat di roadmap: virtualisasi
  DataGrid, Canvas lanjutan, kemampuan editor, dan perilaku khusus Tree.
  Audit 2026-10-09 mengonfirmasi keempat kapabilitas itu **tidak ada sama
  sekali** di luar daftar element; yang ada hanya `width`/`height`/`id` untuk
  Canvas, `value` untuk Editor, dan layout box untuk DataGrid dan Tree.
- [ ] LSP dan debugger. Audit 2026-10-09: nol baris kode, hanya prosa di
  dokumen.
- [x] Audit type checking dan diagnostik terhadap perilaku compiler sekarang.
  Hasilnya:type checking **ada dan berjalan** — `infer_expr_type`
  (`libraries/semantic/src/checker.rs:887`), pengecekan operand aritmetika dan
  bitwise (`:1087`), pencocokan anotasi (`:936`), arity fungsi dan builtin
  (`:1179`). Roadmap yang menyebut "type checking belum ada" sudah usang.
  Yang belum: tipe balik `fn`, dan apa pun yang disimpulkan dari non-literal.

## 4. Catatan konsistensi dokumentasi — SELESAI

- [x] `docs/roadmap.md` diaudit ulang. Tabel fase A ditulis ulang dengan status
  terverifikasi beserta `path:line`; baris 2, 3, dan 4 yang menyatakan
  `unimplemented!` atau "belum ditegakkan" sudah tidak benar dan sekarang
  berstatus SELESAI. Baris 5 tetap SEBAGIAN karena compound assign memang
  belum ada, dan itu ditulis terbuka di bagian 10.1.
- [x] Klaim roadmap bahwa `~` masih terpetakan ke negasi sudah dihapus;
  `libraries/language/src/parser/expression.rs:122` memetakannya ke
  `UnaryOp::BitNot`.
- [x] Referensi ke `fase.md` yang tidak ada dihapus dari `docs/roadmap.md` dan
  `docs/logic-design.md`. Status fase sekarang hidup di `docs/roadmap.md`
  bagian 10 dan di dokumen ini.
- [x] Klaim "type checking belum ada" di `README.md` diganti dengan yang
  benar: ada tapi sebagian.
- [x] `docs/logic-design.md` bagian 4 dan 5 diperbarui: mekanisme kurung
  bitwise yang sebelumnya hanya dijelaskan kini sudah dijalankan, dan pembagian
  sejati versus `idiv` didokumentasikan.

### Masih tidak dikerjakan

- [ ] Path lama di dalam `docs/roadmap.md` masih ada di banyak tempat:
  `libraries/semantic/src/element.rs` (sekarang di crate `language`),
  `libraries/semantic/src/events.rs`, dan
  `libraries/web/src/codegen/web/javascript.rs` (sudah dipecah jadi
  `javascript/{mod,component,api,bootstrap,rust}.rs` sekitar 40 kali dikutip).
- [ ] Angka lama di roadmap sudah diberi peringatan di bagian 1 dokumen itu
  sendiri, tetapi teks aslinya masih menyebut "4 crate" (sekarang 14) dan
  "305 test" (sekarang 441).

## 5. Angka test per suite

`cargo test --workspace`, 2026-10-09, seluruhnya hijau.

| Suite | Jumlah |
| --- | --- |
| `tests/parser` | 25 |
| `tests/lexer` | 31 |
| `tests/semantic` | 67 |
| `tests/ir` | 30 |
| `tests/codegen` | 74 |
| `tests/loader` | 13 |
| `tests/cli` | 32 |
| `tests/fixtures` | 2 |
| `tests/web` | 1 (menjalankan sepuluh harness Node) |
| `tests/rust` | 5 |
| `p2lt` package + acceptance | 4 |
| unit test crate | 157 |
| **total** | **441** |

## 6. Prioritas berikutnya

1. Optimasi output web (bagian 2), mulai dari yang paling murah diukur:
   tree shaking lalu mode produksi, karena keduanya bisa dinilai lewat ukuran
   artefak tanpa mengubah perilaku.
2. Tutup `docs/roadmap.md` bagian 4 — perbaikan path dan angka.
3. Type checking: periksa tipe balik `fn`, karena `infer_expr_type` sudah
   cukup baik untuk membuat aturan itu bisa ditulis.
4. Deteksi bentrok impor fungsi: `declarations()` di
   `compiler/src/loader.rs:307` tidak mengumpulkan `fn` tingkat atas, jadi dua
   modul yang sama-sama mendeklarasikan `fn square` bertabrakan dengan span
   membingungkan, bukan dengan `import-collision`.
