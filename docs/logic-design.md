# Logic Design — Keputusan Sintaks & Teknis Komputasi

Dokumen ini adalah rujukan final untuk keputusan sintaks dan keputusan teknis
yang dibuat selama pengimplementasian paket "kemampuan komputasi". Riwayat dan
bukti per fase ada di `catatan.md` dan tabel fase di `docs/roadmap.md` bagian 10.
Keputusan di bawah bersifat **final** — jika ada konflik dengan dokumen lain,
halaman ini yang menang.

## 1. Presedensi operator (rendah → tinggi)

```
penugasan   = += -= *= /= %= &= |= ^= <<= >>=   (right-associative, bukan ekspresi infix)
rentang     .. ..=
boolean     ||
boolean     &&
kesamaan    == !=
perbandingan < <= > >=
bitwise     |
bitwise     ^
bitwise     &
geser       << >>
penjumlahan + -
perkalian   * / %
unary       - ! ~
call/member ( )
```

Konsekuensi yang dites:

- `n & 1 == 0` → `(n & 1) == 0`
- `1 << 2 + 3` → `1 << (2 + 3)`
- `1 + 2 * 3` → `1 + (2 * 3)`
- `true || false && false` → `true || (false && false)`
- `a - b - c` → `(a - b) - c` (left-associative)

Enum `Precedence` (`libraries/language/src/lexer/operator.rs`) menyimpan urutan
ini dari rendah ke tinggi:
`None < Range < Or < And < Equality < Comparison < BitOr < BitXor < BitAnd <
Shift < Sum < Product < Unary < Call`. Urutan relatif operator lama persis
dipertahankan; hanya blok `Range` dan bitwise/Shift yang disisipkan.

### Keputusan teknis T-0: perbaikan bug presedensi parser

Temuan: `binary_info` (`libraries/language/src/parser/expression.rs`)
sebelumnya mengembalikan `(own, next_higher)` dan `parse_precedence` memakai
`next_higher` sebagai batas operand kanan. Karena batas itu *lebih ketat*
daripada operator itu sendiri, operator yang lebih ketat justru **tidak**
diserap oleh operand kanan, lalu diterapkan di level luar — artinya grup berubah:

- `1 + 2 * 3` dihasilkan sebagai `(1 + 2) * 3` (salah)
- `true || false && false` dihasilkan sebagai `(true || false) && false` (salah)

Keputusan: `binary_info` sekarang mengembalikan `(P, P)` untuk semua operator
infix (semuanya left-associative). Batas operand kanan = presedensi operator
itu sendiri, sehingga operator yang lebih ketat tetap diserap, sedangkan yang
sama presedensi berhenti → asosiativitas kiri terjaga.

Ini adalah **perbaikan bug**, bukan perubahan desain: tabel presedensi yang
dideklarasikan tidak berubah, perilaku hanya diselaraskan dengannya. Tanpa
perbaikan ini `1 << 2 + 3` akan salah kelompok dan seluruh Fase 5 bergantung
padanya.

## 2. Token dan literal (Fase 0)

Token baru: `..` `..=` `=>` `&` `|` `^` `~` `<<` `>>` `&=` `|=` `^=` `<<=`
`>>=` — semuanya **penambahan**; tidak ada token lama yang diubah artinya.

- **Maximal munch**: `scan_symbol` (`libraries/language/src/lexer/scanner.rs`)
  mencoba kandidat 3, 2, lalu 1 byte terhadap tabel `OPERATORS` (hanya kandidat
  ASCII murni). Tabel `is_pair_candidate` lama dihapus karena sudah tidak cukup
  untuk simbol 3 karakter (`<<=`, `..=`, `>>=`).
  - `&` vs `&&`, `|` vs `||`, `<<` vs `<=`, `..` vs `.`, `=>` vs `=`
    terpecah benar karena pencocokan terpanjang menang.
  - `xs.length` tetap `.` tunggal; `a -> b` tetap `Arrow`.
- **Literal basis**: `0x…` / `0b…` dideteksi di `scan_number` hanya bila
  literal diawali tepat `0` dan langsung diikuti `x|X|b|B` tanpa spasi.
  Seluruh karakter alfanumerik setelah penanda dikonsumsi supaya digit salah
  tidak bocor jadi identifier. `parse_int_literal` (`lexer/literal.rs`)
  mem-parsing dengan radix (16/2/10) dan mempertahankan pesan galat lama untuk
  basis 10.
  - `0b2`, `0xG`, `0x`, `0b`, dan overflow → `invalid-int`.
  - `0 b 0 x` (berspasi) tetap `0` lalu identifier — literal lama tak berubah.
  - `1_000_000` tetap valid; `_` dibuang sebelum parsing.
  - Keputusan: hex/biner adalah **sugar untuk Int biasa**, bukan tipe baru;
    hasilnya i64 sehingga `0xFFFFFFFFFFFFFFFF` (2^64-1) ditolak sebagai
    "out of range for Int".
- **Formatter**: `..` dan `..=` tanpa spasi di kedua sisi
  (`cli/src/command/format.rs::separator`) sehingga `0..n` tetap `0..n`
  setelah `plt format`.
- **Compound assignment bitwise** (`&=` dll.) ditambahkan ke tabel operator
  tetapi **belum** dimasukkan ke `TokenKind::is_assignment()`. Alasannya:
  `is_assignment` dipakai `try_parse_assignment`, dan `AssignOp` AST belum
  punya varian bitwise sampai Fase 5. Jika dimasukkan lebih awal, `x &= 1`
  akan terserap sebagai `x = 1` dan kehilangan `&`. Sampai Fase 5, `&=`
  menghasilkan galat `expected-expression` yang jelas.
- **Kata kunci baru**: hanya `let` dan `while` (Fase 1). `for`, `fn`,
  `import` sudah ada. Tidak ada kata kunci baru setelahnya.

## 3. Putusan lanjutan (diisi per fase)

- **Fase 1** — `let` hanya di scope blok; shadowing dilarang dengan galat
  `shadow-local`; `for i in a..b` dievaluasi dengan batas yang dihitung
  sekali; `for x in <angka>` dibungkus guard runtime `plt.iter`.
- **Fase 2** — param lambda di-rename unik (`l<n>_<nama>`) sebelum diikat di
  `Scope` codegen agar `rewrite` tidak salah memindai teks.
- **Fase 5** — IR menyimpan teks JS dan preseden JS berbeda (`&` di JS lebih
  rendah dari `==`), jadi setiap ekspresi bitwise di-emit dengan kurung
  eksplisit penuh: `(a & b)`. Keputusan ini baru benar-benar dijalankan pada
  2026-10-09; sebelumnya `operand_at_least` hanya menambahkan kurung saat
  presedensi operand lebih longgar dari yang dibutuhkan, sehingga
  `n & k == 1` dipancarkan sebagai `n & k == 1` dan JavaScript membacanya
  sebagai `n & (k == 1)`. Angka yang benar adalah `1`, yang dipancarkan `0`.
  Sekarang `is_bitwise` di `libraries/ir/src/lower.rs:996` memaksa kurung pada
  setiap operand bitwise.

## 4. Pembagian `/` dan pembagian bulat

`/` adalah pembagian sejati. `1 / 2` adalah `0.5`, apa pun tipe operandnya,
karena inilah yang diemittekan ke JavaScript. Konsekuensinya: `Int` di backend
web adalah `Number` JavaScript, dan hanya bilangan bulat sampai `2^53 - 1` yang
tersimpan persis. `fib(79)` bernilai `14472334024676221` secara matematis tetapi
dipancarkan `14472334024676220`; `factorial(20)` yang lebih besar justru masih
persis karena faktor duacheckmark lebihanyak. Batas ini diukur, bukan
diasumsikan: test `integersLosePrecision` di `examples/algoritma/src/lib.plt`.

Pembagian bulat tidak bisa ditulis `//` karena `//` sudah menjadi komentar
baris di Platipus — setiap file pada repo ini dimulai dengannya. Sisa pilihan
yang tidak ambigu adalah pemanggilan, jadi ada builtin `idiv(a, b)` yang
memotong ke arah nol. Nol sebagai pembagi tidak punya jawaban bulat, jadi
`idiv` mengembalikan "tidak ada" (`null` di web, `None` di runtime Rust)
bukan `Infinity` atau `NaN`. Kedua backend menolak operand `Float` dengan
alasan yang sama, supaya program yang sama tidak menjawab berbeda tergantung
backend mana yang menjalankannya.
