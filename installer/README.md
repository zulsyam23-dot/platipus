# installer

Skrip Inno Setup untuk toolchain Platipus. Yang dipaketkan hanya tiga binary
yang dibangun workspace ini — `platipus`, `plt`, dan `p2lt` — karena tidak ada
runtime yang perlu dipasang: program Platipus adalah HTML, CSS, dan JavaScript
di satu direktori, dan compiler hanya butuh Node di mesin untuk
`platipus test` dan `platipus dev`.

## Membangun

Binary dulu, lalu skripnya:

```text
cargo build --release -p platipus-cli -p platipus-p2lt
iscc installer\platipus.iss
```

Hasilnya di `installer\Platipus-0.1.0-setup.exe`, sekitar 2.5 MB.

`ISCC.exe` ada di `C:\Users\PC\AppData\Local\Programs\Inno Setup 6\` untuk
install per-user. Kalau terpasang per-machine, lokasinya
`C:\Program Files (x86)\Inno Setup 6\`.

## Yang dikemas

| Berkas | Dari |
| --- | --- |
| `platipus.exe`, `plt.exe`, `p2lt.exe` | `target\release\` |
| `README.md` | root repo, ditandai "read more" |
| `docs\guide.md`, `library.md`, `libplt-format.md`, `package-registry.md` | `docs\` |

Dokumen yang lain tidak ikut: `prd.md`, `roadmap.md`, `logic-design.md`, dan
`struktur.md` adalah catatan desain dan sejarah, bukan sesuatu yang perlu ada di
Program Files. Yang satu-satunya dokumentasi yang perlu saat memasang adalah cara
menulis program dan cara memakai pustaka.

## Ikon

`platipus.ico` dan `wizard.bmp` dibuat dari `platipus.png` di root repo:

```text
python tools\make-installer-assets.py
```

Skripnya ada di sana supaya ikon tidak harus di-regenerate setiap kali logo
berubah, dan supaya langkahnya terlihat: ikon dibuat dari logo yang sama, sehingga tidak ada
file terpisah yang bisa berbeda diam-diam.

## Keputusan yang butuh alasan

**Per-user secara bawaan.** `PrivilegesRequired=lowest`, dengan halaman
pilihan untuk mengubahnya jadi admin. Menaruh tiga `.exe` bernama kata Inggris
biasa di direktori system path perlu dipertimbangkan, dan `plt` khususnya cukup
pendek untuk bertabrakan dengan nama lain. PATH ditulis ke `HKCU`, supaya milik
orang yang memilih task tersebut, bukan milik mesin.

**Tanpa broadcast `WM_SETTINGCHANGE`.** Installer hanya menulis
`HKCU\Environment\Path`. Terminal yang dibuka sesudahnya membacanya dari registry
dengan atau tanpa pesan, jadi broadcast hanya menolong program yang membaca
ulang environment karena pesan itu — dan panggilan ke user32 harus tepat persis
di kedua bitness untuk hasil yang sangat sedikit.

**Task PATH disembunyikan kalau sudah ada.** `NeedsAddPath` di bagian `[Code]`
membandingkan dengan pemisah di depan dan di belakang, jadi direktori yang
sekadar berbagi awalan tidak dianggap sudah ada.

**Tanpa prasyarat Rust.** Blok `#[rust]` memang dikompilasi ke wasm saat build,
tetapi binary yang dihasilkan sudah berisi hasilnya. Pengguna akhir yang
menjalankan program Platipus tidak butuh toolchain Rust; yang butuh Rust hanya
yang menulis blok itu.

## Menghapus

Uninstaller dibuat Inno Setup sendiri (`unins000.exe`), jadi uninstall berjalan
seperti installer. Itu sudah diuji: direktori instalasi dan entri Start Menu
ikut hilang.
