import PemutarVideo from "pemutar-video"

theme Youtube {
    background: "#ffffff"
    surface: "#ffffff"
    surfaceAlt: "#f2f2f2"
    border: "#e5e5e5"
    foreground: "#0f0f0f"
    muted: "#606060"
    accent: "#ff0000"
    radius: "12px"
}

app VideoPlayerTest {
    state volume = 80.0

    Page {
        style {
            margin: "0 auto"
            background: "#ffffff"
            color: "#0f0f0f"
        }
        Column {
            gap: 0

            Row {
                gap: 16
                style {
                    padding: "12px 24px"
                    borderBottom: "1px solid #e5e5e5"
                    alignItems: "center"
                }
                Heading "▶ YouTube" {
                    level: 2
                    style {
                        color: "#ff0000"
                    }
                }
                SearchBar {
                    placeholder: "Cari video..."
                }
            }

            Row {
                gap: 24
                style {
                    padding: 24
                    alignItems: "flex-start"
                }

                Column {
                    gap: 16
                    style {
                        flex: "1"
                    }
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
                    Heading "Video Demo Platipus - Tutorial Lengkap 2026" {
                        level: 3
                    }
                    Row {
                        gap: 16
                        style {
                            alignItems: "center"
                        }
                        Text "Channel Platipus"
                        Text "1,2 jt x ditonton • 3 jam lalu"
                    }
                    Row {
                        gap: 16
                        style {
                            alignItems: "center"
                        }
                        ToggleRow {
                            label: "Subscribe"
                            checked: false
                        }
                        VolumeSlider {
                            value: volume
                        }
                    }
                    ProgressBar {
                        label: "Memuat subtitle"
                        value: 0.8
                    }
                    AudioPlayer {
                        source: "podcast.mp3"
                    }
                }

                Column {
                    gap: 12
                    style {
                        width: "320px"
                    }
                    Heading "Rekomendasi" {
                        level: 3
                    }
                    ImageCard {
                        src: "thumb1.jpg"
                        caption: "Cara Membuat Aplikasi UI"
                    }
                    ImageCard {
                        src: "thumb2.jpg"
                        caption: "Belajar Package Manager"
                    }
                    ImageCard {
                        src: "thumb3.jpg"
                        caption: "Live Coding Platipus"
                    }
                    LoadingOverlay {
                        label: "Memuat rekomendasi..."
                    }
                }
            }
        }
    }
}
