// Lifecycle and reactive-state demos.
component LifecycleProbe {
    input tag: String = "probe"
    state created = 0
    state mounted = 0

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
        Text "update hook emits ticked"
    }
}

component CoreSection {
    input tick: Int = 0
    state local = 1
    shared state sharedCount = 10
    global state globalCount = 100
    persistent state stored = "kept"
    derived doubled = local * 2
    derived tripled = local * 3
    derived combined = local + sharedCount

    Column {
        gap: 18
        SectionIntro {
            eyebrow: "STATE & REACTIVITY"
            title: "Reactive core"
            description: "Local, shared, global, persistent, and derived state update the interface."
        }

        Grid columns: 3 {
            StatTile { value: doubled caption: "derived · 2× local" }
            StatTile { value: tripled caption: "derived · 3× local" }
            StatTile { value: combined caption: "derived · local + shared" }
            StatTile { value: sharedCount caption: "shared state" }
            StatTile { value: globalCount caption: "global state" }
            StatTile { value: stored caption: "persistent state" }
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
}
