// Pointer, keyboard, drag/drop, scrolling, gesture, and custom events.
component EventSection {
    state notice: String = ""
    state clicks = 0
    state lastKey = ""
    state focusState = "none"
    state dropped = "none"
    state scrollY = 0
    state pointer = "0,0"

    on mount { notice = "ready" }

    Column {
        gap: 16
        SectionIntro {
            eyebrow: "EVENTS"
            title: "Browser interactions"
            description: "Try pointer, keyboard, focus, drag/drop, scroll, swipe, and a child event."
        }

        Card {
            padding: 16
            Column {
                gap: 10
                Row {
                    Button "click me" { on click { clicks = clicks + 1 } }
                    Button "double" { on doubleclick { clicks = clicks + 100 } }
                    Button "hold" {
                        on mousedown { pointer = "down" }
                        on mouseup { pointer = "up" }
                    }
                    KeyValue { key: "Click count" val: clicks }
                    KeyValue { key: "Pointer" val: pointer }
                }

                Input {
                    placeholder: "type then press a key"
                    on keydown { lastKey = event.key }
                    on keyup { lastKey = event.key }
                    on focus { focusState = "focused" }
                    on blur { focusState = "blurred" }
                }
                KeyValue { key: "Last key" val: lastKey }
                KeyValue { key: "Focus" val: focusState }

                Row {
                    Button "drop target" {
                        on dragover { dropped = "dragging over target" }
                        on drop { dropped = event.data }
                    }
                    Link "drag this text onto the target" {
                        href: "#drag-payload"
                        on dragstart {
                            event.originalEvent.dataTransfer.setData("text/plain", "showcase payload")
                        }
                    }
                }
                Text dropped

                Scroll {
                    height: 90
                    bind scrollTop: scrollY
                    on scroll { scrollY = event.target.scrollTop }
                    Column {
                        for line in ["01", "02", "03", "04", "05", "06", "07", "08"] {
                            Text line
                        }
                    }
                }
                KeyValue { key: "scrollTop" val: scrollY }

                Button "swipe me" {
                    on swipe { notice = "swiped " + event.dir }
                }
                Text notice
                ChildTalker {
                    on talked { notice = "heard custom event: " + event }
                }
            }
        }
    }
}

component ChildTalker {
    state sent = "nothing"

    fn send() {
        sent = "ping"
        emit talked(sent)
    }

    Row {
        gap: 8
        Button "emit talked" { on click { send() } }
        Text sent
    }
}
