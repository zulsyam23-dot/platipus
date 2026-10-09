# examples

Contoh yang bisa dijalankan. Semuanya program Platipus sungguhan, bukan
potongan kode.

## Peta

| Contoh | Apa itu | Cara menjalankan |
| --- | --- | --- | --- |
| [`counter.plt`](counter.plt) | file | aplikasi paling kecil yang mungkin: satu state, satu `derived`, satu tombol | `platipus dev counter.plt` |
| [`rust/`](rust/) | file | blok `#[rust]` — FFI ke Rust yang dikompilasi ke wasm | `platipus build rust/checkout.plt` |
| [`showcase/`](showcase/) | file | galeri komponen: lifecycle, event, data, layout | `platipus dev showcase/main.plt` |
| [`computasi/`](computasi/) | paket | **pustaka**: komputasi murni + widget presentasi, tanpa `app` dan tanpa `#[rust]` | `platipus test computasi/src/lib.plt` |
| [`computasi-demo/`](computasi-demo/) | paket | aplikasi yang memasang pustaka itu dari **arsip `.libplt`** | lihat README-nya |
| [`computasi-github/`](computasi-github/) | paket | aplikasi yang memasang pustaka itu dari **GitHub** | lihat README-nya |

## Dua jalur installing paket

Kedua contoh-consumer itu sengaja berbeda, karena itu dua cara berbeda orang
mendapatkan pustaka:

```text
computasi-demo/      p2lt install ../computasi/computasi-0.1.0.libplt
computasi-github/    p2lt add github.com/zulsyam23-dot/lib-uji-coba-
```

Yang pertama arsip tunggal yang Anda bawa sendiri; checksumnya dicek saat
dipasang. Yang kedua paket diambil dari jaringan dan `p2lt.lock` mengikat revisi
yang dipakai. Keduanya berakhir sama: aplikasi memanggil fungsi dari pustaka, dan
test pustaka ikut jalan di dalamnya.

## Yang dianggap relevan

Sebuah contoh masuk daftar ini kalau salah satu dari tiga:

1. **dipakai test.** `counter.plt` dan `showcase/main.plt` jadi fixture
   `tests/web/`; `computasi/src/lib.plt` dijalankan `tests/cli/`; paket
   `computasi` jadi bahan `p2lt/tests/acceptance.rs`.
2. **memperagakan satu jalur pemasangan** yang belum diperagakan contoh lain.
   Lihat `computasi-demo` untuk arsip, `computasi-github` untuk GitHub.
3. **memperagakan satu fitur bahasa** yang belum ada tempat lain. `rust/` adalah
   satu-satunya yang menunjukkan blok `#[rust]`.

Yang tidak masuk: hasil build yang ikut ter-commit, dan contoh yang dependensinya
tidak ada di repo ini sehingga tidak bisa dibangun.

## Pranala

* Cara menulis Platipus — [`docs/guide.md`](../docs/guide.md)
* Cara membuat dan memakai pustaka — [`docs/library.md`](../docs/library.md)
* Format arsip `.libplt` — [`docs/libplt-format.md`](../docs/libplt-format.md)
* Registry HTTP dan sumber GitHub — [`docs/package-registry.md`](../docs/package-registry.md)
