# Panduan Coding Platipus

Dokumen ini menjelaskan cara menulis program Platipus sesuai implementasi
saat ini. Untuk daftar element, event, dan properti yang tersedia, lihat
`libraries/language/src/element.rs` dan [roadmap.md](roadmap.md).

## Struktur minimal

```plt
app Main {
    Column {
        Text "Hello"
    }
}
```

## State

```plt
app Counter {
    state count = 0

    Column {
        Text count
        Button "+" {
            on click {
                count = count + 1
            }
        }
    }
}
```

- `state` nilai reaktif per-instance.
- `shared state` dibagi antar instance.
- `derived nama = ekspresi` dihitung ulang saat dependensinya berubah.
- `input nama = default` menandai input komponen dari luar.

## Component

Satu file aplikasi boleh mendefinisikan beberapa `component`:

```plt
component Stat {
    input label = ""
    input value = 0

    Row {
        gap: 8
        Text label
        Text value
    }
}

app Main {
    Stat label: "Total" value: 42
}
```

Komponen dari file lain dipanggil dengan `import`:

```plt
import Stat from "./widgets.plt"
```

Import non-path diambil dari Library Store (lihat
[library.md](library.md)):

```plt
import VideoPlayer from "pemutar-video"
```

## Event

```plt
Button "Save" {
    on click {
        saved = true
    }
}
```

Event juga bisa dideklarasikan di root komponen: `on mount { ... }`.

## Style

```plt
Page {
    style {
        maxWidth: "1080px"
        margin: "0 auto"
    }
}
```

Nama properti di dalam `style` ditulis camelCase dan menjadi kebab-case di CSS.
Nilai **harus literal**: stylesheet tidak bisa membaca state, jadi compiler
menolak nilai lain dengan `style-value-not-literal`. Yang nilainya berubah
sesuai data — tinggi batang, lebar isi meter — ditulis sebagai **properti
element**, yang compiler terbitkan sebagai deklarasi inline pada node itu:

```plt
component Bar {
    input value = 0.0

    Row {
        width: "6px"
        height: value          // dinamis: jadi properti element
        style {
            background: "var(--plt-accent)"   // statis: literal
        }
    }
}
```

Properti element yang tata letak — `width`, `height`, `maxWidth`, `minHeight`,
`padding`, `gap`, `margin` — menjadi CSS, bukan atribut HTML. Properti lain
menjadi atribut atau properti DOM sesuai elemennya: `Slider { value: n }` mengisi
nilai kontrol, bukan CSS.

## Blok Rust

File `.plt` dapat menyematkan Rust tepercaya; compiler mengekstraknya,
membuat crate Cargo kecil, dan mengompilasinya dengan toolchain Rust:

```plt
#[rust]
#[export]
fn fib(n: i64) -> i64 {
    if n <= 1 { n } else { fib(n - 1) + fib(n - 2) }
}

app Main {
    Text fib(12)
}
```

## Test

Blok `test` menjalankan program itu sendiri; `click` menargetkan teks yang
terbaca pengguna, `expect` membaca state program:

```plt
test startsAtZero {
    expect count == 0
}

test clickingAdds {
    click "+"
    expect count == 1
}
```

Jalankan: `platipus test app.plt` atau dari project: `plt test`.

Blok `test` hanya boleh berisi action (`click`) dan `expect` — tidak ada `let`.
Test yang butuh nilai antara memanggil fungsi yang mengembalikannya, seperti yang
`examples/computasi/src/lib.plt` lakukan untuk `sortReport()`.

## Fungsi dan pembagian

`fn` selalu mengembalikan lewat `return`. Tidak ada nilai terakhir yang
dipakai otomatis:

```plt
fn square(x: Int) -> Int {
    return x * x
}
```

Badan `x * x` tanpa `return` berakhir dengan `undefined`, dan pemanggilnya
menerima `undefined` tanpa diagnostic apa pun.

`/` adalah pembagian sejati, jadi `1 / 2` adalah `0.5`. Pembagian bulat
memakai builtin `idiv`, yang memotong ke arah nol:

```plt
fn halving(n: Int) -> Int {
    return idiv(n, 2)   // 7 -> 3, bukan 3.5
}
```

Operasi teks yang sering dipakai: `codeAt(teks, i)` memberi kode karakter pada
indeks ke-`i`, dihitung per karakter (bukan per byte) supaya konsisten dengan
`len`, dan `isPalindrome` di
[`examples/computasi/src/lib.plt`](../examples/computasi/src/lib.plt)
dibangun di atasnya.

## Perintah CLI

| Perintah | Fungsi |
| --- | --- |
| `platipus new my-app` | scaffold project |
| `platipus dev app.plt` | serve `dist/` + rebuild saat file berubah |
| `platipus build app.plt` | compile ke `dist/` |
| `platipus check app.plt` | analisis tanpa menulis output |
| `platipus test app.plt` | jalankan blok `test` |
| `platipus format app.plt` | normalisasi indentasi |
| `p2lt init` | buat `p2lt.toml` + `src/main.plt` |
| `p2lt build` / `p2lt run` | build project per `p2lt.toml` |
