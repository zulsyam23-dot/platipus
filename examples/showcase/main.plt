// Showcase shell. Feature sections live in sections/; shared presentation and
// palette primitives live beside this entry point.
import widgetsModule from "./widgets.plt"
import designModule from "./design.plt"
import reactiveModule from "./sections/reactive.plt"
import formsModule from "./sections/forms.plt"
import eventsModule from "./sections/events.plt"
import dataModule from "./sections/data.plt"
import canvasModule from "./sections/canvas.plt"
import editorsModule from "./sections/editors.plt"
import asyncModule from "./sections/async.plt"
import collectionsModule from "./sections/utilities.plt"

app Showcase {
    global state launches = 0
    shared state section = "Reactive"
    state tabs = [
        "Reactive",
        "Forms",
        "Events",
        "Data",
        "Canvas",
        "Editors",
        "Browser APIs",
        "Collections",
        "Calculator"
    ]
    state tick = 1
    state lifecycleStatus = ""

    on mount {
        launches = launches + 1
        store("local", "showcase.launches", launches)
    }

    Page {
        style {
            width: "100%"
            maxWidth: "1280px"
            margin: "0 auto"
            padding: 24
        }

        Column {
            gap: 18

            Card {
                padding: 22
                Row {
                    gap: 20
                    Column {
                        gap: 5
                        Text "PLATIPUS · WEB RUNTIME" {
                            size: 11
                            weight: 700
                            color: "#38bdf8"
                        }
                        Heading "Showcase Studio" {
                            level: 1
                            size: 32
                            weight: 750
                        }
                        Text "A hands-on tour of the compiler, runtime, and browser APIs." {
                            color: "#94a3b8"
                        }
                    }
                    Spacer { }
                    Card {
                        padding: 14
                        Column {
                            Text "APP LAUNCHES" {
                                size: 11
                                weight: 700
                                color: "#94a3b8"
                            }
                            Text launches {
                                size: 24
                                weight: 700
                            }
                        }
                    }
                }
            }

            Row {
                gap: 8
                for tab in tabs {
                    Button tab {
                        style {
                            backgroundColor: "#17243b"
                            color: "#dbe7f5"
                            borderRadius: 9
                            padding: 9
                            hover {
                                backgroundColor: "#233654"
                            }
                        }
                        on click {
                            section = tab
                            tick = tick + 1
                        }
                    }
                }
            }

            Card {
                padding: 22
                if section == "Reactive" {
                    CoreSection { tick: tick }
                }
                if section == "Forms" {
                    FormSection { }
                }
                if section == "Events" {
                    EventSection { }
                }
                if section == "Data" {
                    DataSection { }
                }
                if section == "Canvas" {
                    CanvasSection { }
                }
                if section == "Editors" {
                    EditorSection { }
                }
                if section == "Browser APIs" {
                    AsyncSection { }
                }
                if section == "Collections" {
                    UtilSection { }
                }
                if section == "Calculator" {
                    CalculatorSection { }
                }
            }

            Row {
                gap: 8
                Text "Platipus 0.1 · Web target" {
                    size: 12
                    color: "#94a3b8"
                }
                Spacer { }
                Text "ACTIVE SECTION" {
                    size: 11
                    weight: 700
                    color: "#94a3b8"
                }
                Text section {
                    weight: 600
                }
            }

            LifecycleProbe {
                tag: "showcase"
                on ticked {
                    lifecycleStatus = "update event · " + event + " · render " + tick
                }
            }
            Text lifecycleStatus {
                size: 12
                color: "#94a3b8"
            }
        }
    }
}
