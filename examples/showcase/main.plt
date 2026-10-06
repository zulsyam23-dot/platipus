#[rust]
#[export]
fn fib(n: i64) -> i64 {
    if n <= 1 {
        n
    } else {
        fib(n - 1) + fib(n - 2)
    }
}

import Stat from "./widgets.plt"

api ShowcaseApi {
    get snapshot from "/api/showcase"
    post reset from "/api/showcase/reset"
}

component CounterPanel {
    state count = 0

    Card {
        padding: 18
        Column {
            gap: 8
            Text "Counter"
            Text count
            Row {
                gap: 8
                Button "+" {
                    on click {
                        count = count + 1
                    }
                }
                Button "reset" {
                    on click {
                        count = 0
                    }
                }
            }
        }
    }
}

app Showcase {
    shared state section = "Reactive"
    state ticks = 0

    on mount {
        store("local", "showcase.launches", 1)
    }

    Page {
        style {
            maxWidth: "1080px"
            margin: "0 auto"
            padding: 24
        }

        Column {
            gap: 16

            Heading "Platipus Showcase" {
                level: 1
                size: 32
            }

            Row {
                gap: 16

                Column {
                    CounterPanel
                }

                Column {
                    Card {
                        padding: 18
                        Column {
                            gap: 8
                            Text "Rust wasm bridge"
                            Text fib(12)
                            Stat label: "Fib(12) =" value: fib(12)
                        }
                    }
                }
            }
        }
    }
}
