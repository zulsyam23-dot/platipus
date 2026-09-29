// Browser APIs and async composition, isolated from the app shell.
component AsyncSection {
    state body = ""
    state shown = ""
    state result = "not run"
    state localBack = ""
    state sessionBack = ""
    state indexedBack = ""
    state response = null
    state ok = false
    state info = null
    state socket = null
    state socketUrl = ""
    state heard = ""
    state before = ""

    async fn loadRemote() {
        result = "loading"
        response = await fetch("data:text/plain,platipus")
        result = response.status + ":" + response.text
    }

    async fn copyOut() {
        body = "copied at " + result
        ok = await writeClipboard(body)
        result = "clipboard write: " + ok
    }

    async fn copyIn() {
        body = await readClipboard()
        result = "clipboard read: " + body
    }

    async fn pick() {
        info = await openFile()
        if info.cancelled {
            result = "file picker cancelled"
        } else {
            result = "file: " + info.name
        }
    }

    async fn talk() {
        if socketUrl == "" {
            result = "enter a WebSocket URL"
        } else {
            result = "connecting"
            socket = await webSocket(socketUrl)
            if socket.status == "open" {
                socket.send("ping")
                heard = await receive(socket)
                result = "socket: " + heard
            } else {
                result = "socket " + socket.status
            }
        }
    }

    async fn readAll() {
        store("local", "showcase.k", "local-value")
        store("session", "showcase.k", "session-value")
        store("indexed", "showcase.k", "indexed-value")
        localBack = load("local", "showcase.k")
        sessionBack = load("session", "showcase.k")
        indexedBack = await load("indexed", "showcase.k")
    }

    async fn pause() {
        before = result
        await wait(30)
        shown = "waited, was " + before
    }

    Column {
        gap: 16
        SectionIntro {
            eyebrow: "BROWSER APIS"
            title: "Async runtime"
            description: "Run fetch, clipboard, file picker, WebSocket, storage, and timer demos on demand."
        }
        Card {
            padding: 14
            Column {
                gap: 8
                Field {
                    Text "WebSocket URL"
                    Input {
                        placeholder: "wss://your-echo-server"
                        bind value: socketUrl
                    }
                }
                Text "Use a reachable echo endpoint. The test shim provides its own local echo."
            }
        }
        Row {
            Button "fetch" { on click { loadRemote() } }
            Button "clipboard write" { on click { copyOut() } }
            Button "clipboard read" { on click { copyIn() } }
            Button "open file" { on click { pick() } }
            Button "socket" { on click { talk() } }
            Button "storage" { on click { readAll() } }
            Button "wait 30ms" { on click { pause() } }
        }
        Card {
            padding: 14
            Column {
                gap: 8
                KeyValue { key: "Result" val: result }
                KeyValue { key: "Local storage" val: localBack }
                KeyValue { key: "Session storage" val: sessionBack }
                KeyValue { key: "IndexedDB" val: indexedBack }
                KeyValue { key: "Clipboard" val: body }
                KeyValue { key: "Timer" val: shown }
            }
        }
    }
}
