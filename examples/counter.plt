component Counter {
    state count = 0
    derived doubled = count * 2

    fn increment(by: Int) {
        count += by
    }

    fn reset() {
        count = 0
    }

    Column {
        Text "count: "
        Text count
        Text "doubled: "
        Text doubled
        Row {
            Button "-1" {
                on click {
                    increment(-1)
                }
            }
            Button "+1" {
                on click {
                    increment(1)
                }
            }
            Button "reset" {
                on click {
                    reset()
                }
            }
        }
    }
}

app CounterApp {
    Page {
        Counter { }
    }
}

style counter {
    backgroundColor: "#101014"
    padding: 16
    hover {
        backgroundColor: "#181820"
    }
}
