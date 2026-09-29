component Counter {
    state count = 0
    derived doubled = count * 2

    fn increment(by: Int) {
        count += by
    }

    Column {
        Text "count: "
        Text count
        if count > 0 {
            Text "positive"
        } else {
            Text "not positive"
        }
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
        }
        for label in ["a", "b"] {
            Text label
        }
        Input {
            placeholder: "type"
            bind value: count
        }
    }
}

app CounterApp {
    Page {
        Counter { }
    }
}

style card {
    backgroundColor: "#101014"
    padding: 16
    hover {
        backgroundColor: "#181820"
    }
}
