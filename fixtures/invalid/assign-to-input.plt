app Main {
    input start: Int = 0

    state count = start
    Column {
        Text count
        Button "bump" {
            on click {
                start = start + 1
            }
        }
    }
}
