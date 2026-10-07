import PemutarVideo from "pemutar-video"

theme Youtube {
    background: "#f9f9f9"
    surface: "#ffffff"
    surfaceAlt: "#f2f2f2"
    border: "#e5e5e5"
    foreground: "#0f0f0f"
    muted: "#606060"
    accent: "#ff0000"
    radius: "12px"
    fontSize: "15px"
    gap: "16px"
    pad: "16px"
    controlPad: "8px 16px"
    inputPad: "8px 12px"
}

app VideoPlayerTest {
    state volume = 80.0

    Page {
        style {
            margin: "0 auto"
            background: "var(--plt-bg)"
            color: "var(--plt-text)"
        }
        Column {
            gap: 0

            Row {
                gap: 16
                style {
                    padding: "12px 24px"
                    borderBottom: "1px solid var(--plt-border)"
                    background: "var(--plt-surface)"
                    alignItems: "center"
                }
                Heading "▶ YouTube" {
                    level: 2
                    style {
                        color: "var(--plt-accent)"
                        weight: "700"
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
                    maxWidth: "1280px"
                    margin: "0 auto"
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
                        style {
                            weight: "600"
                        }
                    }
                    Row {
                        gap: 16
                        style {
                            alignItems: "center"
                            color: "var(--plt-muted)"
                        }
                        Text "Channel Platipus"
                        Text "•"
                        Text "1,2 jt x ditonton • 3 jam lalu"
                    }
                    Row {
                        gap: 16
                        style {
                            alignItems: "center"
                        }
                        Card {
                            padding: 12
                            style {
                                background: "var(--plt-surface-2)"
                                borderRadius: "var(--plt-radius)"
                            }
                            ToggleRow {
                                label: "Subscribe"
                                checked: false
                            }
                        }
                        Card {
                            padding: 12
                            style {
                                background: "var(--plt-surface-2)"
                                borderRadius: "var(--plt-radius)"
                            }
                            VolumeSlider {
                                value: volume
                            }
                        }
                    }
                    Card {
                        padding: 16
                        style {
                            background: "var(--plt-surface)"
                            borderRadius: "var(--plt-radius)"
                            border: "1px solid var(--plt-border)"
                        }
                        ProgressBar {
                            label: "Memuat subtitle"
                            value: 0.8
                        }
                    }
                    Card {
                        padding: 16
                        style {
                            background: "var(--plt-surface)"
                            borderRadius: "var(--plt-radius)"
                            boxShadow: "var(--plt-shadow)"
                        }
                        AudioPlayer {
                            source: "podcast.mp3"
                        }
                    }
                }

                Column {
                    gap: 12
                    style {
                        width: "340px"
                    }
                    Heading "Rekomendasi" {
                        level: 3
                        style {
                            weight: "600"
                        }
                    }
                    Card {
                        padding: 12
                        style {
                            background: "var(--plt-surface)"
                            borderRadius: "var(--plt-radius)"
                            border: "1px solid var(--plt-border)"
                        }
                        ImageCard {
                            src: "thumb1.jpg"
                            caption: "Cara Membuat Aplikasi UI"
                        }
                    }
                    Card {
                        padding: 12
                        style {
                            background: "var(--plt-surface)"
                            borderRadius: "var(--plt-radius)"
                            border: "1px solid var(--plt-border)"
                        }
                        ImageCard {
                            src: "thumb2.jpg"
                            caption: "Belajar Package Manager"
                        }
                    }
                    Card {
                        padding: 12
                        style {
                            background: "var(--plt-surface)"
                            borderRadius: "var(--plt-radius)"
                            border: "1px solid var(--plt-border)"
                        }
                        ImageCard {
                            src: "thumb3.jpg"
                            caption: "Live Coding Platipus"
                        }
                    }
                    Column {
                        gap: 8
                        style {
                            padding: 24
                            alignItems: "center"
                            background: "var(--plt-surface)"
                            borderRadius: "var(--plt-radius)"
                            border: "1px solid var(--plt-border)"
                        }
                        LoadingOverlay {
                            label: "Memuat rekomendasi..."
                        }
                    }
                }
            }
        }
    }
}
