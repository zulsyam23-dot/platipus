# computasi

Komputasi dan widget presentasi, ditulis penuh dalam Platipus. Tidak ada blok
`#[rust]`, tidak ada helper dari host: setiap angka yang ditampilkan dihitung di
dalam paket ini.

## Isi paket

```text
computasi/
├── p2lt.toml
└── src/
    ├── lib.plt       fungsi-fungsi komputasi, plus test-nya
    └── widgets.plt   komponen tampilan, diimpor oleh lib.plt
```

## Yang dihitung

| Kelompok | Fungsi |
| --- | --- |
| Teori bilangan | `gcd`, `lcm`, `isPrime`, `primesUpTo`, `primeFactors`, `primeFactorPairs`, `divisors` |
| Barisan | `fib`, `factorial`, `collatzSteps`, `collatzSequence` |
| Daftar | `sum`, `product`, `maxOf`, `minOf`, `mean`, `median`, `variance`, `sortAsc`, `sortDesc`, `binarySearch`, `filterEach`, `mapEach`, `chunk`, `sliceOf`, `zipSum` |
| Teks | `isPalindrome`, `fnv1a32`, `wordCount`, `editDistance`, `isAnagram` |

## Komponen

| Komponen | Untuk apa |
| --- | --- |
| `StatTile`, `SuccessTile`, `DangerTile` | satu angka besar dengan label dan catatan kaki |
| `Meter` | proporsi 0 sampai 100 persen |
| `SparkBars` | deretan batang yang diskalakan terhadap nilai tertinggi |
| `Chip`, `AccentChip` | nilai kecil yang menonjol |
| `FindingPanel` | judul plus kisi chip |
| `ResultList`, `ResultRow` | tabel label/nilai |

## Memasang

```text
p2lt install computasi
```

Atau dari arsip langsung:

```text
p2lt pack                                  # di dalam paket ini
p2lt install ../computasi/computasi-0.1.0.libplt
```

Atau dari GitHub:

```text
p2lt install github.com/zulsyam23-dot/computasi
```

## Memakai

```plt
import Komputasi from "computasi"

theme Midnight {
    background: "#0f1117"
    surface: "#171a23"
    accent: "#7aa2f7"
    radius: "12px"
}

app Papan {
    state seed = 27

    derived steps = collatzSteps(seed)
    derived primes = primesUpTo(100)

    Page {
        Grid {
            columns: 3
            StatTile label: "Langkah Collatz" value: toText(steps) hint: "dari " + toText(seed)
            SuccessTile label: "Prima di bawah 100" value: toText(len(primes))
        }
        SparkBars values: collatzSequence(seed) height: 56
    }
}
```

Aplikasi yang lengkap ada di
[`examples/computasi-demo`](../computasi-demo). Ia tidak menghitung apa pun
sendiri; yang berubah ketika Anda menggeser slider adalah hasil pustaka yang
ikut berubah.

## Dua aturan yang perlu diketahui

Keduanya berasal dari fakta tentang CSS, bukan pilihan gaya:

1. **`style { }` hanya menerima literal.** Stylesheet tidak bisa membaca state,
   jadi checker menolak nilai yang bukan literal dengan
   `style-value-not-literal`. Yang dinamis — tinggi sebuah batang, lebar isi
   meter — ditulis sebagai **properti element**, yang compiler terbitkan inline
   pada node tersebut. Atribut warna di dalam `style { }` juga harus berupa
   literal CSS.

2. **Tidak ada operator ternary.** `if` adalah statement, bukan expression.
   Nilai yang bergantung pada sebuah pilihan datang dari sebuah fungsi.

## Test

Paket membawa test-nya sendiri, dan test itu ikut berjalan di aplikasi yang
memakainya:

```text
platipus test src/lib.plt
```

Empat belas blok test, masing-masing menyatakan angka yang benar-benar
dihitung: gcd dan lcm, prima sampai batas, faktor prima, barisan, pembagi,
aritmetika daftar, sorting yang tidak mengubah inputnya, pencarian dan
pemotongan, statistik, lambda, operasi teks, dan angka-angka yang mengisi
widget.
