import PemutarVideo from "pemutar-video"

app VideoPlayerTest {
    state volume = 50.0

    Page {
        style {
            maxWidth: "720px"
            margin: "0 auto"
            padding: 24
            background: "#eef2f7"
            color: "#0f172a"
        }
        Column {
            gap: 16
            Heading "Demo Pemutar Video" {
                level: 1
            }

            VideoPlayer {
                source: "movie.mp4"
            }

            AudioPlayer {
                source: "podcast.mp3"
            }

            ImageCard {
                src: "poster.jpg"
                caption: "Poster Film"
            }

            SearchBar {
                placeholder: "Cari video..."
            }

            ProgressBar {
                label: "Mengunduh"
                value: 0.6
            }

            ToggleRow {
                label: "Mode gelap"
                checked: true
            }

            VolumeSlider {
                value: volume
            }

            LoadingOverlay {
                label: "Memuat video..."
            }

            AlertDialog {
                open: true
                title: "Selamat datang"
                message: "Demo library pemutar-video"
            }
        }
    }
}
