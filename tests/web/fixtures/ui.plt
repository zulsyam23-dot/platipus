component Stage {
    state frame = 0
    state c = null
    async fn tick() {
        if frame < 3 {
            fill(c, frame, 10, 20, 20, "tomato")
            frame = frame + 1
            await nextFrame()
            tick()
        }
    }
    Column {
        Canvas {
            id: "board"
            width: 320
            height: 200
        }
        Button "draw" {
            on click {
                c = canvas("board")
                clear(c)
                tick()
            }
        }
        Text frame
    }
}

component EditDemo {
    state body = "start"
    state sel = 0
    Column {
        Editor {
            value: body
            bind value: body
            on input {
                sel = selection().start
            }
        }
        Button "bold" {
            on click {
                exec("bold")
            }
        }
        Button "italic" {
            on click {
                exec("italic")
            }
        }
        Text body
        Text sel
    }
}

component CodeDemo {
    state doc = "fn hello() { return 1 }"
    Column {
        CodeEditor {
            value: doc
            bind value: doc
            language: "plt"
            highlight: "plt"
        }
        Button "tab" {
            on click {
                indent()
                exec("selectAll")
            }
        }
        Button "grow" {
            on click {
                doc = doc + "x"
            }
        }
        Text doc
    }
}

component DataDemo {
    state rows = ["beta", "omega", "alpha"]
    state shown = []
    fn refresh() {
        shown = page(sortBy(rows, null, "desc"), 1, 1)
    }
    on mount {
        refresh()
    }
    Column {
        for row in shown {
            Text row
        }
    }
}

component TreeDemo {
    state dropped = "none"
    state open = false
    Column {
        Button "toggle" {
            on click {
                open = !open
            }
        }
        for kid in ["a", "b"] {
            Text kid
        }
        if open {
            Text "open"
        }
        Button "leaf" {
            on drop {
                dropped = event.data
            }
        }
        Text dropped
    }
}

app DemoApp {
    Page {
        Column {
            Stage { }
            EditDemo { }
            CodeDemo { }
            DataDemo { }
            TreeDemo { }
        }
    }
}