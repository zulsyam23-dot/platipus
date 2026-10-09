# Format `.libplt`

`.libplt` adalah arsip satu file yang membawa seluruh isi paket Platipus. Ini
dokumen yang menjelaskan formatnya, cara membuatnya, dan apa yang dijamin dan
tidak dijamin.

Kalau yang ingin Anda lakukan hanya memasang paket yang sudah ada, bagian
[Memakai arsip](#memakai-arsip) saja sudah cukup.

## 1. Isi arsip

`p2lt pack` membuat satu file bernama `<nama>-<versi>.libplt` di root project:

```text
pemutar-video-0.1.0.libplt
```

Di dalamnya ada:

| Bagian | Isi |
| --- | --- |
| `MAGIC` | 8 byte penanda `PLTPKG\0\0`, supaya file lain tidak salah dibaca |
| `manifest` | `p2lt.toml` yang sudah lengkap dengan `integrity`, dalam bentuk TOML |
| daftar entry | path relatif dan isi tiap file, diurutkan |
| `entry_checksum` | FNV-1a 64 atas seluruh path dan isi entry |

`p2lt.toml` tidak ikut sebagai file; ia ditulis ulang dari manifest di dalam
arsip, supaya metadata arsip dan metadata project tidak bisa berbeda.

### Yang ikut dan yang tidak

`pack` menyalin seluruh isi direktori project secara rekursif, kecuali:

| Tidak ikut | Alasan |
| --- | --- |
| `.p2lt/` | state lokal package manager |
| `target/`, `dist/` | artefak build, bisa dibangun ulang dari sumber |
| `.git/` | riwayat ada di tempatnya |

Jadi `src/`, `assets/`, `rust/`, `tests/`, dan `README.md` **ikut**. Berkas
`.libplt` hasil `pack` sebelumnya juga ikut, karena tidak dikecualikan — jalankan
`p2lt clean` lebih dulu kalau project Anda menumpuk arsip.

### Urutan entry

Entry diurutkan berdasarkan path, dengan pemisah `/` dan urutan huruf besar-kecil
seperti pada perbandingan string di Rust. Urutan ini yang membuat checksum
stabil: pack dua kali dari isi yang sama menghasilkan byte yang sama, apa pun
urutan sistem file.

## 2. Membuat arsip

```text
cd pemutar-video
p2lt pack
```

Perintah ini juga menjalankan `validate` pada hasilnya, jadi arsip yang gagal
tidak pernah sampai ke disk.

Untuk memeriksa arsip tanpa memasang:

```text
p2lt validate pemutar-video-0.1.0.libplt
```

Keluaran:

```text
valid: pemutar-video 0.1.0
```

### `integrity`

`pack` mengisi `integrity` sendiri bila belum ada, dengan bentuk
`fnv1a64:<16 digit hex>` dari seluruh isi payload. Manifest yang ditulis ulang
ke dalam arsip adalah manifest yang sudah terisi itu, sehingga `validate`
menyebutkan checksum yang sama dengan yang dihitung dari isi arsip.

## 3. Memakai arsip

Dari berkas lokal:

```text
cd my-app
p2lt install ../pemutar-video/pemutar-video-0.1.0.libplt
```

Yang terjadi:

* isi arsip diekstrak ke `<store>/packages/pemutar-video/`;
* `p2lt.toml` project menambah `pemutar-video = "0.1"` di `[dependencies]`;
* `p2lt.lock` diperbarui dengan checksum payload yang terpasang.

Store bersifat global per mesin
(`%LOCALAPPDATA%\platipus\store\packages` di Windows,
`~/.local/share/platipus/packages` di Unix) dan **tidak pernah** masuk ke dalam
project, jadi tidak ada `node_modules` yang perlu di-commit.

Untuk memasang dari registry lokal, taruh `.libplt` di direktori registry dan
panggil namanya:

```text
p2lt install pemutar-video
```

Untuk registry HTTP, lihat [`package-registry.md`](../../docs/package-registry.md).

Setelah terpasang, paket dipakai seperti biasa:

```plt
import PemutarVideo from "pemutar-video"

app Main {
    VideoPlayer {
        source: "movie.mp4"
    }
}
```

`import` dengan nama paket dibaca sebagai
`<store>/packages/<nama>/src/lib.plt`. `import X from "./file.plt"` tetap dibaca
relatif terhadap file yang mendeklarasikannya.

## 4. Pemeriksaan saat memasang

`install` menolak arsip yang:

| Situasi | Pesan |
| --- | --- |
| magic bukan `PLTPKG\0\0` | bukan arsip `.libplt` |
| habis sebelum panjang yang tertulis dibaca | arsip terpotong |
| `entry_checksum` tidak cocok | payload checksum mismatch |
| nama di manifest tidak sama dengan yang diminta | nama paket tidak cocok |
| manifest tidak punya nama atau versi | manifest tidak lengkap |
| tidak ada `src/lib.plt` atau `src/main.plt` | package source is missing |

Pemeriksaan pertama yang gagal menghentikan pemasangan; tidak ada keadaan
setengah terpasang.

## 5. Impor relatif di dalam paket

Paket boleh memecah sumbernya sendiri:

```text
computasi/
├── p2lt.toml
└── src/
    ├── lib.plt        ← mengimpor parts.plt
    └── parts.plt
```

`./parts.plt` di dalam `lib.plt` berarti "di sebelah `lib.plt`", dan itu tetap
berlaku setelah paket dipasang di store: loader melaporkan file mana yang
benar-benar dibaca, dan impor relatif di dalamnya diselesaikan terhadap file itu,
bukan terhadap file yang kebetulan mengimpor paket tersebut.

Kalau impor relatif tidak ditemukan, galatnya menunjuk ke file library-nya
sendiri:

```text
computasi/src/lib.plt:4:18: semantic[import-not-found]: cannot find `./parts.plt` imported as `Parts`
```

## 6. Yang tidak dijamin

* **Bukan tanda tangan kriptografis.** `integrity` adalah FNV-1a, yang
  mendeteksi korusi dan salah kirim, bukan bukti kepada pihak ketiga. Untuk
  distribusi publik, taruh arsip di tempat yang sudah dijaga, atau tambahkan
  tanda tangan sendiri di luar P2LT.
* **Tidak ada build script.** P2LT tidak menjalankan apa pun dari paket saat
  memasang. Isi arsip adalah data; apa pun yang berjalan di program Anda
  berasal dari kode yang Anda tulis sendiri.
* **Tidak ada resolusi versi.** `p2lt install <arsip>` memasang apa yang ada di
  arsip. `p2lt update` mengambil ulang dari registry atau GitHub sesuai yang
  tercatat di lockfile.

## 7. Alur lengkap, dari sumber ke aplikasi

Contoh nyata yang berjalan di repo ini, dari paket sampai aplikasi yang
memakainya:

```text
# 1. paket library
cd examples/computasi
p2lt pack                      # -> computasi-0.1.0.libplt
p2lt validate computasi-0.1.0.libplt

# 2. registry lokal, supaya arsip punya nama
p2lt publish                   # salin ke direktori registry

# 3. aplikasi yang memakainya
cd examples/computasi-demo
p2lt install computasi         # atau: p2lt install ../computasi/computasi-0.1.0.libplt
p2lt build
node dist/tests.mjs
```

Langkah terakhir menjalankan programnya: aplikasi memasang paket, dan setiap widget-nya
diisi angka yang dihitung library, dan test library ikut jalan di dalamnya.

Atau tanpa registry sama sekali, langsung dari GitHub:

```text
p2lt install github.com/zulsyam23-dot/computasi
```

Lihat [`package-registry.md`](../../docs/package-registry.md) untuk protokol
registry HTTP dan sumber GitHub.
