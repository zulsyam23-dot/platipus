# Catatan pekerjaan yang belum selesai

Diperbarui: 2026-10-08

## Kondisi yang sudah diverifikasi

- `cargo clippy --workspace -- -D warnings` lulus.
- `cargo test --workspace` lulus.
- Kedua hasil ini hanya memverifikasi lint dan test yang tersedia; belum membuktikan semua fitur bahasa atau performa produksi selesai.
- Registry HTTP dan `p2lt add github.com/owner/repository` kini diimplementasikan; lihat `docs/package-registry.md`. Fitur ini belum diverifikasi terhadap layanan registry publik atau repository GitHub nyata.

## Prioritas berikutnya

### 1. Tutup bukti kemampuan komputasi murni Platipus

- [ ] Buat paket penerimaan `examples/algoritma/` dengan `p2lt.toml` dan `src/lib.plt`, tanpa `#[rust]` dan tanpa `app`.
- [ ] Tambahkan test nilai nyata untuk algoritma yang diminta: gcd/lcm, primalitas, Fibonacci, faktorial, operasi daftar, Collatz, sorting tanpa mengubah input, binary search, string palindrome, FNV-1a, dan lambda map.
- [ ] Uji alur paket dari ujung ke ujung: publish, install di aplikasi lain, impor beberapa fungsi, panggil dari `derived` dan handler, lalu build.
- [ ] Uji batas performa yang diminta: loop satu juta iterasi di harness Node.
- [ ] Audit dan lengkapi test lintas parser, semantic, IR, codegen, dan runtime untuk arity, tipe argumen, kemurnian fungsi, import fungsi, rentang angka, closure, serta operator bitwise. Jangan menganggap dukungan lengkap hanya karena parser atau satu lapisan sudah menerimanya.

### 2. Optimasi aplikasi/output web

Roadmap (bagian 4.10) mencatat hal-hal berikut belum lengkap:

- [ ] Fine-grained rendering: perubahan state masih merender ulang seluruh virtual tree komponen.
- [ ] Batching lintas dispatch: perubahan hanya digabung dalam satu dispatch; belum ada coalescing lintas event/tick. Runtime host juga belum memiliki transaksi setara.
- [ ] Virtualisasi daftar besar: `each` masih memproses seluruh daftar; `page` bukan virtualisasi transparan.
- [ ] Code splitting: output masih satu bundel JavaScript.
- [ ] Tree shaking: komponen/fungsi yang tidak terpakai belum dihapus dari output.
- [ ] Mode produksi: belum ada jalur produksi/minifikasi khusus untuk artefak web.

Sebelum menyebut optimasi “super”, ukur baseline dan hasil dengan benchmark yang dapat diulang, serta pastikan perubahan tidak mengubah perilaku aplikasi.

### 3. Fitur produk lain yang belum lengkap

- [ ] Lengkapi fitur Advanced UI yang dicatat di roadmap, termasuk virtualisasi DataGrid, fitur Canvas lanjutan, kemampuan editor, serta perilaku khusus Tree.
- [ ] LSP dan debugger belum tersedia.
- [ ] Audit kebutuhan type checking dan diagnostik berdasarkan perilaku compiler sekarang; roadmap menyebut type checking belum ada, tetapi status itu perlu diverifikasi terhadap perubahan kode terbaru.

## Catatan konsistensi dokumentasi

- `docs/roadmap.md` perlu diaudit ulang: tabel fase kemampuan komputasi masih menyebut beberapa implementasi lama sebagai belum ada, sementara tree kerja saat ini sudah berisi perubahan terkait lambda, builtin, kemurnian, dan arity.
- Bagian roadmap merujuk `fase.md`, tetapi file tersebut tidak ditemukan di root repository.
- Perbarui status hanya setelah bukti test dan path kode diperiksa; bedakan fitur yang belum ada dari fitur yang ada tetapi belum diuji end-to-end.
