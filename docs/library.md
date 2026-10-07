# Membuat dan Memakai Library Platipus

Library Platipus adalah project tanpa `app` yang mengekspor komponen, API,
style, atau test, lalu dibungkus menjadi satu file `.libplt` dan diinstal
melalui `p2lt`. Library tidak ditaruh di dalam folder project: hasil instalasi
hidup di Library Store global.

## Struktur library

```text
pemutar-video/
├── p2lt.toml
└── src/
    └── lib.plt
```

`p2lt.toml`:

```toml
[package]
name = "pemutar-video"
version = "0.1.0"
edition = "2026"

[dependencies]
```

`src/lib.plt` berisi deklarasi yang diekspor — tanpa `app`:

```plt
component VideoPlayer {
    input source = ""
    input poster = ""
    input controls = true

    Column {
        gap: 8
        Video {
            src: source
            poster: poster
            controls: true
        }
    }
}
```

## Pack menjadi .libplt

```text
cd pemutar-video
p2lt pack          # menghasilkan pemutar-video-0.1.0.libplt
p2lt validate pemutar-video-0.1.0.libplt
```

`.libplt` satu file paket: manifest tertanam, payload deterministic, dan
checksum `integrity` (FNV-1a) yang diverifikasi saat `validate`/`install`.

## Install

Dari file lokal:

```text
p2lt install ./pemutar-video-0.1.0.libplt
```

Dari registry direktori lokal (lihat bagian Registry):

```text
p2lt install pemutar-video
```

Instalasi menulis ke Library Store global:

```text
%LOCALAPPDATA%\platipus\store\packages\pemutar-video\   (Windows)
~/.local/share/platipus/packages/pemutar-video/          (Linux/macOS)
```

Folder project tetap bersih: hanya `p2lt.toml`, `p2lt.lock`, `src/`, dan
`dist/` setelah build. Override lokasi store dengan `PLATIPUS_STORE`.

## Memakai library

```plt
import VideoPlayer from "pemutar-video"

app Main {
    VideoPlayer {
        source: "movie.mp4"
    }
}
```

Import non-path dicari di Library Store: `<store>/packages/<nama>/src/lib.plt`.
Import `./file.plt` tetap dibaca relatif terhadap file yang mengimpornya.
Compiler (`platipus dev`, `platipus build`, `p2lt build`) memakai loader yang
sama, jadi import dari package ter-resolve di mana pun.

Lalu:

```text
p2lt build
```

## Registry lokal

Registry adalah direktori berisi subfolder package (masing-masing
`p2lt.toml` + `src/`). Default: `<store>/registry`, override dengan
`P2LT_REGISTRY`.

```text
cd pemutar-video
p2lt publish        # menyalin package ke registry

p2lt install pemutar-video
p2lt list
p2lt remove pemutar-video
p2lt update
p2lt search video
```

## GitHub sebagai sumber distribusi

Saat ini integrasi GitHub belum otomatis (belum ada `plt add
github.com/...`). Alur yang sudah bisa dipakai:

1. Buat library, `p2lt pack` menjadi `pemutar-video-0.1.0.libplt`.
2. Push source ke GitHub, misalnya `github.com/user/test-pemutar-video-`.
3. Unggah `.libplt` sebagai asset GitHub Release `v0.1.0`.
4. Pemakai mengunduh `.libplt` dari release lalu:
   `p2lt install ./pemutar-video-0.1.0.libplt`.

Setelah di-install, `import VideoPlayer from "pemutar-video"` dan `p2lt build`
bekerja tanpa konfigurasi tambahan.

## Manifest

Field `[package]` yang dikenal:

| Field | Wajib | Arti |
| --- | --- | --- |
| `name` | ya | nama package |
| `version` | ya | versi semver |
| `edition` | tidak | default `"2026"` |
| `package_id` | tidak | identitas stabil package |
| `platipus_version` | tidak | rentang versi Platipus yang didukung |
| `targets` | tidak | platform yang didukung, mis. `["web"]` |
| `entry_points` | tidak | entry point tambahan |
| `integrity` | tidak | diisi otomatis saat `p2lt pack` |

`[dependencies]` memetakan nama package ke versi yang kompatibel
(`nama = "0.1"`), `[rust.dependencies]` untuk crate Rust blok `#[rust]`.

## Checklist sebelum publish

1. `p2lt validate <file>.libplt` lolos.
2. Install ke store lalu `p2lt build` pada project contoh berhasil.
3. Import dari package benar-benar dipakai di build (lihat `dist/app.js`).
4. Project pemakai tetap tanpa folder dependency.
