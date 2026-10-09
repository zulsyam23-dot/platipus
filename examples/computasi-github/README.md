# examples/computasi-github

Aplikasi yang memasang paket `computasi` **langsung dari GitHub** dan
memanggilnya. Paketnya tidak ikut di repo ini.

## Alurnya

```text
p2lt add github.com/zulsyam23-dot/lib-uji-coba-
p2lt build
node dist/tests.mjs
```

`p2lt add` meng-clone repository itu, membaca `p2lt.toml` dari root-nya, dan
memasang paket `computasi` ke Library Store. `p2lt.lock` di folder ini mencatat
source dan revisi yang dipakai:

```text
[[package]]
name = "computasi"
version = "0.1.0"
checksum = "58e440de16f6a9a9"
source = "github.com/zulsyam23-dot/lib-uji-coba-"
revision = "118d03999d2f11d38334d2be2923bc74e008c8d0"
```

## Buktinya bukan kebetulan

Hapus store-nya, lalu build lagi:

```text
$env:PLATIPUS_STORE = "$env:TEMP\store-kosong"
p2lt build
```

Hasilnya:

```text
semantic[import-not-found]: cannot find `computasi` imported as `Komputasi`
```

Lalu pasang dari GitHub dan build sekali lagi, dan programnya jalan. Angka-angka
yang tampil di halaman itu karena repo `lib-uji-coba-` mengeluarkannya, bukan
karena ada salinan di sini.

## Yang diuji

`node dist/tests.mjs` menjalankan 17 test: 4 milik aplikasi ini dan 13 milik
paket, yang ikut terbawa ke dalam program. Semua test aplikasi membaca angka
melalui `import` yang sama dengan yang dipakai widget, sehingga setiap angka
yang tampil sudah dibuktikan di sini, bukan hanya di test library.

| Test | Yang dibuktikan |
| --- | --- |
| `theInstalledPackageComputes` | Collatz, prima, faktor, FNV-1a, median, variansi |
| `thePackageHelpersAreReachable` | `resultPairs`, `primeDensity`, `smallPrimes`, `collatzBars`, `barHeights` |
| `movingTheSeedMovesThePackageFigures` | mengubah state mengubah angka dari paket |
| `inspectingFillsTheReport` | handler memanggil `editDistance`, `wordCount`, `factorial` |

## Isi

```text
computasi-github/
├── p2lt.toml          mendaftarkan dependensi
├── p2lt.lock          hasil add: source dan revisi
└── src/main.plt       aplikasi; tidak menghitung apa pun
```

Aplikasi ini memiliki `theme` bernama dan seluruh tata letaknya. Widget-nya
datang dari paket. Itulah pemisahan yang dipakai
[`docs/library.md`](../../docs/library.md): pustaka menentukan bentuk,
aplikasi menentukan warna.
