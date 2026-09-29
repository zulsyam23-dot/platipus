// Exercises how components take their place in the document: an app whose only
// root is a component, a component with several sibling roots, a component whose
// inputs arrive from the parent and change while it is alive, and a branch that
// swaps one section for another.
//
// Every case here is something the runtime got wrong at least once, so the
// harness in components.mjs can tell the difference between a tree that mounted
// and a tree that mounted *and* cleaned up after itself.

// Written out in full so the default and the value the parent passes are not the
// same value: a component that cannot tell them apart proves nothing.
//
// `n` is the child's own state and `echo` is an input, so the two can be told
// apart on screen. Seeding happens on mount only, which is what makes the
// difference between an input arriving at a live child and a fresh one visible.
component Counter {
    input start = 0
    input label = "count"
    input echo = "none"
    state n = 0

    on mount {
        n = start
    }

    fn bump() {
        n = n + 1
    }

    Row {
        gap: 4
        Button label {
            on click {
                bump()
            }
        }
        Text n
        Text echo
    }
}

// Two roots with no parent of their own, so the runtime has no single node to
// put in the document and has to keep both.
component Pair {
    input left = ""
    input right = ""

    Row {
        Text left
        Text right
    }
    Text "pair-footer"
}

app Components {
    // The app's only root is a component, so nothing builds a tree unless the
    // runtime instantiates a child before looking for its nodes.
    state n = 0
    state section = "counter"
    state echo = "echo-0"

    Column {
        gap: 4
        Row {
            gap: 4
            Button "bump" {
                on click {
                    n = n + 1
                }
            }
            Button "retag" {
                on click {
                    echo = "echo-1"
                }
            }
            Button "counter" {
                on click {
                    section = "counter"
                }
            }
            Button "pair" {
                on click {
                    section = "pair"
                }
            }
            Button "none" {
                on click {
                    section = "none"
                }
            }
        }

        if section == "counter" {
            Counter {
                start: n
                label: "outer"
                echo: echo
            }
        }

        if section == "pair" {
            Pair {
                left: "left-slot"
                right: "right-slot"
            }
        }

        if section == "none" {
            Text "nothing here"
        }
    }
}
