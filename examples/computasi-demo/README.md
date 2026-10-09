# examples/computasi-demo

Aplikasi yang memasang pustaka `computasi` dari **arsip `.libplt`**, bukan dari
registry dan bukan dari GitHub. Yang membedakan contoh ini dari
[`computasi-github`](../computasi-github) adalah asal paketnya: satu file yang
Anda bawa sendiri, yang checksumnya dicek saat dipasang.

## Alurnya

Buat arsipnya dari pustaka:

```text
cd ../computasi
p2lt pack                                  # -> computasi-0.1.0.libplt
p2lt validate computasi-0.1.0.libplt       # -> valid: computasi 0.1.0
```

Lalu pasang dan jalankan:

```text
cd ../computasi-demo
p2lt install ../computasi/computasi-0.1.0.libplt
p2lt build
node dist/tests.mjs
```

Hasilnya **17 test hijau**: 4 milik aplikasi ini dan 13 milik pustaka, yang ikut
terbawa ke dalam program. Test pustaka ikut jalan karena impor menggabungkan
sumber, jadi blok `test` milik pustaka ikut terbawa ke program konsumen — dan
runner menariknya lewat nama yang diekspor pustaka, bukan nama internalnya.

Format arsipnya dijelaskan di
[`docs/libplt-format.md`](../../docs/libplt-format.md).

## Yang Dimiliki Aplikasi

Aplikasi tidak menghitung apa pun. Yang Dimilikinya:

* `theme Midnight` — seluruh warna, jarak, dan sudut dari token `--plt-*`, jadi
  widget pustaka ikut berubah tanpa disentuh;
* pertanyaan apa yang dilihat ke pustaka: `seed` dan `limit` lewat slider, teks lewat
  input;
* dua komponen Status yang memutuskan warnanya **di kode**, karena stylesheet
  tidak bisa membaca state.

Angka yang tampil — Collatz, prima, FNV-1a, median, variansi — semuanya dihitung
di dalam pustaka.

## Catatan tentang dua aturan bahasa

Keduanya muncul karena limitasi nyata CSS, bukan pilihan gaya:

* Nilai di dalam `style { }` harus literal. Checker menolak yang lain dengan
  `style-value-not-literal`.
* `if` adalah statement, bukan expression. Tidak ada operator ternary, jadi nilai
  yang bergantung pada pilihan datang dari fungsi.

Detailnya di [`docs/guide.md`](../../docs/guide.md).
