import VideoPlayer from "pemutar-video"

app VideoPlayerTest {
    Column {
        Text "Video Player Test"
        VideoPlayer {
            source: "movie.mp4"
            controls: true
        }
    }
}
