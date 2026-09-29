// Rich text and syntax-highlighted code editing.
component EditorSection {
    state body = "type here"
    state caret = 0
    state doc = "fn main() {\n  return 1\n}"
    state mode = "plain"

    Column {
        gap: 16
        SectionIntro {
            eyebrow: "EDITORS"
            title: "Text & code"
            description: "Edit rich text, inspect the selection, and press Tab in the code editor to indent."
        }
        Grid columns: 2 {
            Card {
                padding: 14
                Column {
                    Text "Rich text editor"
                    Editor {
                        value: body
                        bind value: body
                        on input { caret = selection().start }
                        on focus { mode = "focused" }
                        on blur { mode = "blurred" }
                    }
                    KeyValue { key: "Selection start" val: caret }
                    KeyValue { key: "Focus state" val: mode }
                    Text body
                    Row {
                        Button "bold" { on click { exec("bold") } }
                        Button "select all" { on click { exec("selectAll") } }
                    }
                }
            }
            Card {
                padding: 14
                Column {
                    Text "CodeEditor · Platipus"
                    CodeEditor {
                        bind value: doc
                        language: "plt"
                        highlight: "plt"
                        on keydown {
                            if event.key == "Tab" {
                                event.preventDefault()
                                indent()
                            }
                        }
                    }
                    Text "Tab inserts two spaces while the editor is focused."
                    Button "append line" { on click { doc = doc + "\n// appended" } }
                    Text doc
                }
            }
        }
    }
}
