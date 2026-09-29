// Canvas 2D drawing and animation.
component CanvasSection {
    state frames = 0
    state handle = null
    state last = ""
    state bars = [0, 1, 2, 3, 4]
    state running = false

    async fn animate() {
        if frames < 6 {
            clear(handle)
            for i in bars {
                fill(handle, i * 40, 10, 32, 32, "#38bdf8")
            }
            drawText(handle, "frame " + frames, 12, 170)
            frames = frames + 1
            last = "drew frame " + frames
            await nextFrame()
            animate()
        } else {
            running = false
        }
    }

    Column {
        gap: 16
        SectionIntro {
            eyebrow: "CANVAS"
            title: "2D drawing loop"
            description: "Draw rectangles and text, then advance six animation frames."
        }
        Card {
            padding: 14
            Canvas {
                id: "showcase-board"
                width: 640
                height: 360
            }
        }
        Row {
            Button "paint" {
                disabled: running
                on click {
                    if !running {
                        handle = canvas("showcase-board")
                        frames = 0
                        running = true
                        animate()
                    }
                }
            }
            Button "clear" {
                disabled: handle == null || running
                on click {
                    clear(handle)
                    last = "canvas cleared"
                }
            }
            Button "tint" {
                disabled: handle == null || running
                on click {
                    clear(handle, "#111a2e")
                    last = "canvas tinted"
                }
            }
            Text last
        }
    }
}
