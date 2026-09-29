import widgets from "./widgets.plt"

component Probe2 {
    input title: String = "x"

    shared state sharedCount = 0
    global state globalCount = 0
    state local = 0
    derived doubled = local * 2
    state items = [{ name: "ann", age: 30 }, { name: "bob", age: 41 }]
    state sock = null

    async fn asyncStuff() {
        local = await wait(1)
        store("local", "k", "v")
        sock = await webSocket("ws://x")
    }

    Column {
        Heading title
        Text doubled
        Text sharedCount
        Text globalCount
        DataGrid {
            Header "Name"
            Header "Age"
            TableRow {
                Cell "ann"
                Cell "30"
            }
        }
        Tree {
            Text "root"
        }
        Editor {
            value: "hi"
        }
        CodeEditor {
            language: "js"
            bind value: local
        }
        Progress {
            value: 50
        }
        Slider {
            min: 0
            max: 10
            bind value: local
        }
        Select {
            bind value: local
            Option "one" { value: "1" }
        }
        Tabs {
            Tab "a" {
                Text "aa"
            }
        }
        List {
            Text "i1"
        }
        Table {
            Header "h"
            TableRow {
                Cell "c"
            }
        }
        for p in items {
            Text p.name
            Text p.age
        }
        StatTile { value: "1" }
    }
}

app ProbeApp {
    Page {
        Probe2 { title: "t" }
    }
}

