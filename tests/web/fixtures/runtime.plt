// Exercises the browser runtime surface: fetch, clipboard, file picking, web
// sockets, and the three storage backends. Everything runs once on mount and
// lands in a state, which is what lets the test read it back from the DOM.
app Runtime {
    state fetched = ""
    state clip = ""
    state picked = ""
    state sockText = ""
    state storedLocal = ""
    state storedSession = ""
    state storedIndexed = ""
    state response = 0
    state ok = false
    state info = 0
    state sock = null
    state cancelled = false

    async fn pickAgain() {
        info = await openFile()
        picked = info.name
        cancelled = info.cancelled
    }

    async fn run() {
        response = await fetch("data:text/plain,hi")
        fetched = response.status + ":" + response.text

        ok = await writeClipboard("draft")
        clip = await readClipboard()

        info = await openFile()
        picked = info.name

        sock = await webSocket("ws://echo")
        sock.send("ping")
        sockText = await receive(sock)

        store("local", "theme", "dark")
        storedLocal = load("local", "theme")
        store("session", "tab", "1")
        storedSession = load("session", "tab")
        store("indexed", "note", "x")
        storedIndexed = await load("indexed", "note")
    }

    on mount { run() }

    Column {
        Text fetched
        Text clip
        Text picked
        Text sockText
        Text storedLocal
        Text storedSession
        Text storedIndexed
        Button "cancel file picker" {
            on click { pickAgain() }
        }
        Button "choose fallback file" {
            on click { pickAgain() }
        }
        Text cancelled
    }
}