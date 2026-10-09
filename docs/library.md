# Membuat dan Memakai Library Platipus

Dokumen ini menjelaskan langkah aktual saat ini. Semua perintah diuji
terhadap tree repo.

## Konsep

* **Package** — satu direktori berisi `p2lt.toml` dan kode Platipus.
* **`.libplt`** — arsip satu file hasil `p2lt pack`, dibawa ke mesin lain.
* **Library Store** — tempat hasil install, global per mesin
  (`%LOCALAPPDATA%\platipus\store\packages` di Windows,
  `~/.local/share/platipus/packages` di Unix). Bukan di folder project.
* **Theme** — cara aplikasi mengubah tampilan library: semua warna, jarak,
  sudut, dan shadow komponen dibaca dari token CSS `--plt-*`.

## 1. Buat project library

Library adalah package tanpa `app` di dalam modul sumbernya.

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

[rust.dependencies]
```

`src/lib.plt` berisi komponen, fungsi, atau API. Contoh saat ini:

```plt
component VideoPlayer {
    input source = ""
    input poster = ""
    input autoplay = false
    input loop = false
    input muted = false

    Column {
        gap: 8
        Video {
            src: source
            poster: poster
            controls: true
            autoplay: autoplay
            loop: loop
            muted: muted
        }
    }
}
```

Komponen library dibuat **headless**: tidak ada `style {}` bawaan di dalam
komponen. Tata letak visual sepenuhnya milik aplikasi pemakai.

## 2. Pack dan validate

```text
cd pemutar-video
p2lt pack      # menghasilkan pemutar-video-0.1.0.libplt
p2lt validate pemutar-video-0.1.0.libplt
```

Manifest lupa diisi `integrity`, `p2lt pack` mengisinya otomatis saat pack.

## 3. Install ke project aplikasi

Dari file lokal:

```text
cd my-app
p2lt install ../pemutar-video/pemutar-video-0.1.0.libplt
```

Atau dari registry direktori lokal (folder berisi paket sumber,
default `<store>/registry` atau `$P2LT_REGISTRY`):

```text
p2lt publish        # dari dalam folder library
p2lt install pemutar-video
```

Yang terjadi:

* kode disalin ke `<store>/packages/pemutar-video/`
* `p2lt.toml` project menambah `pemutar-video = "0.1"` di `[dependencies]`
* `p2lt.lock` diperbarui dengan checksum
* folder project tetap bersih — tidak ada `node_modules`

Perintah lain:

```text
p2lt list
p2lt remove pemutar-video
p2lt update
p2lt search video
```

## 4. Memakai di aplikasi

```plt
import PemutarVideo from "pemutar-video"

app Main {
    VideoPlayer {
        source: "movie.mp4"
    }
}
```

`import X from "nama-paket"` dicari ke `<store>/packages/nama-paket/src/lib.plt`.
Import `./file.plt` tetap dibaca relatif terhadap file pengimpor. `p2lt build`,
`platipus build`, dan `platipus dev` memakai resolver yang sama.

## 5. Menata dari aplikasi

Karena komponen headless, tampilannya dirakit di aplikasi:

```plt
Card {
    padding: 16
    style {
        background: "var(--plt-surface)"
        borderRadius: "var(--plt-radius)"
        boxShadow: "var(--plt-shadow)"
    }
    VideoPlayer {
        source: "movie.mp4"
    }
}
```

Dan warna/jarak global diatur sekali lewat `theme`:

```plt
theme Youtube {
    background: "#f9f9f9"
    surface: "#ffffff"
    surfaceAlt: "#f2f2f2"
    border: "#e5e5e5"
    foreground: "#0f0f0f"
    muted: "#606060"
    accent: "#ff0000"
    radius: "12px"
}
```

Semua komponen library mengikuti token ini karena elemen dasarnya
(`Card`, `Button`, `Input`, `Video`, ...) ditata dari token yang sama di
`libraries/web/src/codegen/style/sheet.rs`.

Token standar yang bisa di-override dari `theme`:

| Token theme | CSS var | Default |
| --- | --- | --- |
| `background` | `--plt-bg` | `#f6f6f8` |
| `surface` | `--plt-surface` | `#ffffff` |
| `surfaceAlt` | `--plt-surface-2` | `#f0f0f4` |
| `border` | `--plt-border` | `#d9d9e0` |
| `foreground` | `--plt-text` | `#16161c` |
| `muted` | `--plt-muted` | `#6a6a78` |
| `accent` | `--plt-accent` | `#2f6feb` |
| `accentText` | `--plt-accent-text` | `#ffffff` |
| `danger` | `--plt-danger` | `#c0392b` |
| `success` | `--plt-success` | `#1e7a48` |
| `radius` | `--plt-radius` | `8px` |
| `gap` | `--plt-gap` | `12px` |
| `pad` | `--plt-pad` | `12px` |
| `fontSize` | `--plt-font-size` | `15px` |
| `lineHeight` | `--plt-line-height` | `1.5` |
| `controlPad` | `--plt-control-pad` | `7px 14px` |
| `controlGap` | `--plt-control-gap` | `6px` |
| `inputPad` | `--plt-input-pad` | `7px 10px` |
| `fieldGap` | `--plt-field-gap` | `5px` |
| `checkSize` | `--plt-check-size` | `16px` |
| `cellPad` | `--plt-cell-pad` | `8px 12px` |
| `tabPad` | `--plt-tab-pad` | `8px 14px` |
| `itemPad` | `--plt-item-pad` | `6px 10px` |
| `menuPad` | `--plt-menu-pad` | `6px` |
| `overlayPad` | `--plt-overlay-pad` | `10px 14px` |
| `barPad` | `--plt-bar-pad` | `8px` |

Perhatikan: nama token memakai camelCase di `theme`, dan compiler mengubahnya
menjadi kebab-case CSS var (`controlPad` → `--plt-control-pad`).

## 6. Distribusi

Yang sudah bisa dipakai hari ini:

1. `p2lt pack` → `.libplt`
2. Upload source + `.libplt` ke GitHub Release
3. Pemakai mengunduh `.libplt`, lalu `p2lt install ./nama-versi.libplt`

Untuk registry HTTP dan paket GitHub, lihat
[protokol registry](package-registry.md). `p2lt add github.com/user/repo`
mengkloning repository dan mencatat sumber serta revision di lockfile.

## Checklist sebelum publish

1. `p2lt validate <file>.libplt` lolos.
2. Komponen tidak membawa `style {}` sendiri (headless).
3. Install ke project contoh, `p2lt build` berhasil.
4. Tampilan komponen mengikuti `theme` di aplikasi contoh.
