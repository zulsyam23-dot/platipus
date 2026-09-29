// ============================================================================
// platipus showcase — one section per feature area
// ============================================================================
//
// This program is the manual acceptance test for the compiler and the web
// runtime. Every section is reachable from the tab bar at the top, and every
// section writes what it did into a value the reader can see on screen. The
// companion harness `showcase.mjs` drives the same program under the Node DOM
// shim and asserts the same values, so the two together check that a section
// does what this file claims it does.
//
// The browser-facing sections (clipboard, file picker, sockets) are driven by
// button clicks rather than by `on mount`, because those APIs require a user
// gesture in a real browser. Running them on mount would work in the shim and
// fail in the page, which is exactly the kind of gap this file exists to find.
// ============================================================================

import widgets from "./widgets.plt"

// Counts how many times each lifecycle hook actually fires. `update` is
// reported through an emitted event rather than a state write, because a
// state write inside `update` re-enters the render that called it and the
// counters would climb without bound.
component LifecycleProbe {
    input tag: String = "probe"
    state created = 0
    state mounted = 0
    state updated = 0

    on create {
        created = created + 1
    }

    on mount {
        mounted = mounted + 1
    }

    on update {
        emit ticked(tag)
    }

    Row {
        gap: 8
        Text created
        Text mounted
        Text updated
    }
}

// ---------------------------------------------------------------------------
// Reactive core: all five state kinds, derived values, and cascading updates.
// ---------------------------------------------------------------------------
component CoreSection {
    input tick: Int = 0
    state local = 1
    shared state sharedCount = 10
    global state globalCount = 100
    persistent state stored = "kept"
    derived doubled = local * 2
    derived tripled = local * 3
    derived combined = local + sharedCount

    Row {
        gap: 12
        StatTile { value: doubled caption: "derived doubled" }
        StatTile { value: tripled caption: "derived tripled" }
        StatTile { value: sharedCount caption: "shared state" }
        StatTile { value: globalCount caption: "global state" }
        StatTile { value: stored caption: "persistent state" }
        StatTile { value: combined caption: "derived of derived" }
    }

    Row {
        gap: 8
        Button "local +1" {
            on click {
                local = local + 1
                globalCount = globalCount + 1
            }
        }
        Button "shared +5" {
            on click {
                sharedCount = sharedCount + 5
            }
        }
        Button "persist" {
            on click {
                stored = "kept-" + tick
                store("local", "showcase.stored", stored)
            }
        }
        Button "restore" {
            on click {
                stored = load("local", "showcase.stored")
            }
        }
        Text local
    }
}

// ---------------------------------------------------------------------------
// Forms: every input primitive, two-way binding, validation.
// ---------------------------------------------------------------------------
component FormSection {
    state notice: String = ""
    state name = "ada"
    state bio = ""
    state plan = "pro"
    state volume = 5
    state agreed = false
    state notify = true
    state when = "2026-09-29"
    state clock = "09:30"
    state accent = "#3399ff"
    state attachment = ""

    on mount {
        name = load("local", "showcase.name")
    }

    Column {
        gap: 10

        Field {
            Text "name"
            Input {
                placeholder: "your name"
                bind value: name
                on keydown {
                    notice = "key:" + event.key
                }
            }
            Text notice
        }

        Field {
            Text "bio"
            Textarea {
                placeholder: "say something"
                bind value: bio
            }
        }

        Field {
            Text "plan"
            Select {
                bind value: plan
                Option "free" { value: "free" }
                Option "pro" { value: "pro" }
                Option "team" { value: "team" }
            }
        }

        Field {
            Text "volume"
            Slider {
                min: 0
                max: 10
                step: 1
                bind value: volume
            }
            Text volume
            Button "volume up" {
                on click {
                    volume = volume + 1
                }
            }
        }

        Row {
            gap: 12
            Checkbox {
                bind checked: agreed
            }
            Text "agreed"
            Switch {
                bind checked: notify
            }
            Text "notify"
        }

        Row {
            gap: 12
            Date {
                bind value: when
            }
            Time {
                bind value: clock
            }
            Color {
                bind value: accent
            }
        }

        Field {
            Text "attachment"
            File {
                accept: ".txt"
                on change {
                    attachment = event.value
                }
            }
            Text attachment
        }

        Form {
            novalidate: true
            on submit {
                name = "ada-" + volume
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Events: pointer, keyboard, focus, form submission, drag and drop, scrolling,
// and a custom event travelling from a child up to the app.
// ---------------------------------------------------------------------------
component EventSection {
    state notice: String = ""
    state clicks = 0
    state lastKey = ""
    state focusState = "none"
    state caretAt = 0
    state dropped = "none"
    state scrollY = 0
    state note = ""
    state pointer = "0,0"

    on mount {
        notice = "ready"
    }

    Column {
        gap: 10

        Row {
            gap: 8
            Button "click me" {
                on click {
                    clicks = clicks + 1
                }
            }
            Button "double" {
                on doubleclick {
                    clicks = clicks + 100
                }
            }
            Button "hold" {
                on mousedown {
                    pointer = "down"
                }
                on mouseup {
                    pointer = "up"
                }
            }
            Text clicks
            Text pointer
        }

        Input {
            placeholder: "type then press a key"
            on keydown {
                lastKey = event.key
            }
            on keyup {
                lastKey = event.key
            }
            on focus {
                focusState = "focused"
            }
            on blur {
                focusState = "blurred"
            }
        }
        Text lastKey
        Text focusState

        Button "drop target" {
            on dragover {
                dropped = "over"
            }
            on drop {
                dropped = event.data
            }
        }
        Text dropped

        Scroll {
            height: 90
            bind scrollTop: scrollY
            on scroll {
                // A scroll event carries no pointer, so the position comes from
                // the control itself. `scroll` is bound in the capture phase
                // because it does not bubble.
                scrollY = event.target.scrollTop
            }
            Column {
                Text "1"
                Text "2"
                Text "3"
                Text "4"
                Text "5"
                Text "6"
                Text "7"
                Text "8"
            }
        }
        Text scrollY

        Button "swipe me" {
            on swipe {
                // The gesture is derived from pointerdown/pointerup, so the
                // payload is `{ dx, dy, dir }` rather than a platform event.
                notice = "swiped " + event.dir
            }
        }
        Text notice

        // The custom event is declared here, on the element that emits it, and
        // its payload arrives as `event`. Reaching this handler at all is the
        // proof that an emit crosses the component boundary.
        ChildTalker {
            on talked {
                notice = "heard custom event: " + event
            }
        }
    }
}

// Emits a custom event so the app can prove that `emit` crosses a component
// boundary and that the app's own handler runs.
component ChildTalker {
    state sent = "nothing"

    fn send() {
        sent = "ping"
        emit talked(sent)
    }

    Row {
        gap: 8
        Button "emit talked" {
            on click {
                send()
            }
        }
        Text sent
    }
}

// ---------------------------------------------------------------------------
// Data: list, table, grid, tree, plus the sort and paginate builtins.
// ---------------------------------------------------------------------------
component DataSection {
    state rows = [
        { name: "ann", team: "core", score: 91 },
        { name: "bob", team: "web", score: 78 },
        { name: "cyd", team: "core", score: 84 },
        { name: "dee", team: "docs", score: 66 }
    ]
    state order = "desc"
    state pageIndex = 0
    state shown = []
    state picked = "none"

    fn refresh() {
        shown = page(sortBy(rows, "score", order), pageIndex, 2)
    }

    on mount {
        refresh()
    }

    Column {
        gap: 10

        Row {
            gap: 8
            Button "sort desc" {
                on click {
                    order = "desc"
                    refresh()
                }
            }
            Button "sort asc" {
                on click {
                    order = "asc"
                    refresh()
                }
            }
            Button "next page" {
                on click {
                    pageIndex = pageIndex + 1
                    refresh()
                }
            }
            Button "first page" {
                on click {
                    pageIndex = 0
                    refresh()
                }
            }
        }

        Table {
            Header "name"
            Header "team"
            Header "score"
            for row in shown {
                TableRow {
                    Cell row.name
                    Cell row.team
                    Cell row.score
                }
            }
        }

        DataGrid {
            for row in rows {
                TableRow {
                    Cell row.name
                    Cell row.team
                    on click {
                        picked = row.name
                    }
                }
            }
        }
        Text picked

        List {
            for row in rows {
                Text row.name
            }
        }

        Tree {
            for row in rows {
                Text row.team
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Canvas: clearing, filling, and text, driven a frame at a time.
// ---------------------------------------------------------------------------
component CanvasSection {
    state frames = 0
    state handle = null
    state last = ""
    state bars = [0, 1, 2, 3, 4]

    async fn animate() {
        if frames < 6 {
            clear(handle)
            for i in bars {
                fill(handle, i * 40, 10, 32, 32, "teal")
            }
            drawText(handle, "frame " + frames, 12, 170)
            frames = frames + 1
            last = "drew frame " + frames
            await nextFrame()
            animate()
        }
    }

    Column {
        gap: 8
        Canvas {
            id: "showcase-board"
            width: 320
            height: 200
        }
        Row {
            gap: 8
            Button "paint" {
                on click {
                    handle = canvas("showcase-board")
                    frames = 0
                    animate()
                }
            }
            Button "clear only" {
                on click {
                    clear(handle)
                    last = "cleared"
                }
            }
            Button "tint" {
                on click {
                    clear(handle, "#101014")
                    last = "tinted"
                }
            }
        }
        Text last
    }
}

// ---------------------------------------------------------------------------
// Editors: rich text with a live selection readout, and a code editor whose
// round trip is the thing the earlier caret bug used to break.
// ---------------------------------------------------------------------------
component EditorSection {
    state body = "type here"
    state caret = 0
    state doc = "fn main() {\n  return 1\n}"
    state mode = "plain"

    Column {
        gap: 8
        Heading "Editor"
        Editor {
            value: body
            bind value: body
            on input {
                caret = selection().start
            }
            on focus {
                mode = "focused"
            }
            on blur {
                mode = "blurred"
            }
        }
        Text caret
        Text mode
        Text body

        Row {
            gap: 8
            Button "bold" {
                on click {
                    exec("bold")
                }
            }
            Button "selectAll" {
                on click {
                    exec("selectAll")
                }
            }
        }

        Heading "CodeEditor"
        CodeEditor {
            bind value: doc
            language: "plt"
            highlight: "plt"
        }
        Row {
            gap: 8
            Button "indent" {
                on click {
                    indent()
                }
            }
            Button "append line" {
                on click {
                    doc = doc + "\n// appended"
                }
            }
        }
        Text doc
    }
}

// ---------------------------------------------------------------------------
// Async and the browser APIs. Each one is behind its own button so that a
// failure shows up as a line in the log instead of an exception that stops
// everything after it.
// ---------------------------------------------------------------------------
component AsyncSection {
    state body = ""
    state shown = ""
    state result = "not run"
    state localBack = ""
    state sessionBack = ""
    state indexedBack = ""
    // Temporaries are component state, not block locals: the language has no
    // `let`, so anything an async body needs to read back has to be declared
    // up here and then written.
    state response = null
    state ok = false
    state info = null
    state socket = null
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
        result = "file: " + info.name
    }

    async fn talk() {
        result = "connecting"
        socket = await webSocket("ws://echo")
        socket.send("ping")
        heard = await receive(socket)
        result = "socket: " + heard
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
        // Reads the value the other handlers left behind, waits, then reports
        // what it was: the point is that the read happened before the wait.
        before = result
        await wait(30)
        shown = "waited, was " + before
    }

    Column {
        gap: 8
        Row {
            gap: 8
            Button "fetch" {
                on click {
                    loadRemote()
                }
            }
            Button "clipboard write" {
                on click {
                    copyOut()
                }
            }
            Button "clipboard read" {
                on click {
                    copyIn()
                }
            }
            Button "open file" {
                on click {
                    pick()
                }
            }
            Button "socket" {
                on click {
                    talk()
                }
            }
            Button "storage" {
                on click {
                    readAll()
                }
            }
            Button "wait 30ms" {
                on click {
                    pause()
                }
            }
        }
        Text result
        Text localBack
        Text sessionBack
        Text indexedBack
        Text body
        Text shown
    }
}

// ---------------------------------------------------------------------------
// Utilities that take a list and return a list.
// ---------------------------------------------------------------------------
component UtilSection {
    state names = ["delta", "alpha", "charlie", "bravo"]
    state order = "asc"
    state pageIndex = 0
    state out = ""

    // `sortBy` and `page` are separate builtins, so the two steps live in one
    // place that every button calls after it has changed a state. Paging the
    // sorted list rather than the source is what makes the order visible.
    fn refresh() {
        out = page(sortBy(names, null, order), pageIndex, 2)
    }

    Column {
        gap: 8
        Row {
            gap: 8
            Button "sort asc" {
                on click {
                    order = "asc"
                    refresh()
                }
            }
            Button "sort desc" {
                on click {
                    order = "desc"
                    refresh()
                }
            }
            Button "page 0" {
                on click {
                    pageIndex = 0
                    refresh()
                }
            }
            Button "page 1" {
                on click {
                    pageIndex = 1
                    refresh()
                }
            }
        }
        Field {
            Text "sorted"
            Text out
        }
        Text "source"
        for name in names {
            Text name
        }
    }
}

// ---------------------------------------------------------------------------
// The app itself.
// ---------------------------------------------------------------------------
app Showcase {
    global state launches = 0
    shared state section = "core"
    state tabs = ["core", "forms", "events", "data", "canvas", "editors", "async", "utils"]
    state talkLog = ""
    state lifeLog = ""
    // Starts at 1, not 0, so the value the app passes into `CoreSection` is
    // distinguishable from that component's own default for the same input.
    state tick = 1
    state note = ""

    on mount {
        launches = launches + 1
        store("local", "showcase.launches", launches)
    }

    fn report(entry: String) {
        talkLog = talkLog + entry + ";"
    }

    Page {
        Column {
            gap: 12

            Row {
                gap: 10
                Heading "platipus showcase"
                Text "launches:"
                Text launches
            }

            // Tab bar. `section` is shared state, so every component that
            // reads it moves together.
            Row {
                gap: 6
                for tab in tabs {
                    Button tab {
                        on click {
                            section = tab
                            tick = tick + 1
                        }
                    }
                }
            }

            if section == "core" {
                CoreSection { tick: tick }
            }
            if section == "forms" {
                FormSection { }
            }
            if section == "events" {
                EventSection { }
            }
            if section == "data" {
                DataSection { }
            }
            if section == "canvas" {
                CanvasSection { }
            }
            if section == "editors" {
                EditorSection { }
            }
            if section == "async" {
                AsyncSection { }
            }
            if section == "utils" {
                UtilSection { }
            }

            // `ticked` is a custom event too: the probe emits it from `update`,
            // and the app listens on the element that emitted it.
            LifecycleProbe {
                tag: "app"
                on ticked {
                    lifeLog = "ticked at " + tick
                }
            }

            Text talkLog
            Text lifeLog
        }
    }
}

// ---------------------------------------------------------------------------
// Style and theme, which are emitted into app.css rather than into the tree.
// ---------------------------------------------------------------------------
style showcasePanel {
    backgroundColor: "#14141a"
    padding: 16
    hover {
        backgroundColor: "#1c1c26"
    }
}

theme dark {
    background: "#0d0d11"
    foreground: "#e8e8ef"
    accent: "#3399ff"
}

// Declared so the manifest exercises the api block; there is no server behind
// it, and nothing in this program calls it.
api Showcase {
    get snapshot from "/api/showcase"
    post reset from "/api/showcase/reset"
}
