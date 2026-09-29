// Exercises the four lifecycle events on both a component and an element, and
// the cleanup that has to happen when a node leaves the document.
//
// `note` records into state, which is what lets the test read the log after
// mount. It cannot be used from `update`, because a state write there re-runs
// the render that called it and the log would grow without end, so `update`
// reports through an emitted event instead: emitting is not a state write, so
// the pass it happens in is the last one.
app Lifecycle {
    state n = 0
    state show = true
    state log = ""

    fn note(entry: String) {
        log = log + entry + ";"
    }

    fn trace(entry: String) {
        emit observed(entry)
    }

    on create {
        note("component-create")
    }

    on mount {
        note("component-mount")
        n = 1
    }

    on update {
        trace("component-update")
    }

    on destroy {
        note("component-destroy")
    }

    Column {
        on create {
            note("element-create")
        }
        on mount {
            note("element-mount")
        }
        on update {
            trace("element-update")
        }
        Text n
        Row {
            Button "+" {
                on click {
                    n = n + 1
                }
            }
            Button "toggle" {
                on click {
                    show = !show
                }
            }
        }
        if show {
            Button "branch" {
                on create {
                    note("branch-create")
                }
                on mount {
                    note("branch-mount")
                }
                on destroy {
                    note("branch-destroy")
                }
                on click {
                    n = n + 100
                }
            }
        }
    }
}
