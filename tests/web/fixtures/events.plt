component Field {
    state text = "start"
    state lastKey = ""

    Column {
        Input {
            placeholder: "type"
            on input {
                text = event.value
            }
            on keydown {
                lastKey = event.key
            }
        }
        Text text
        Text lastKey
    }
}

app EventApp {
    state scrollY = 0
    state depth = 0

    Page {
        Field { }
        Scroll {
            on scroll {
                scrollY = event.position.y
            }
            Text scrollY
        }
        Scroll {
            bind scrollTop: depth
            Text depth
        }
    }
}