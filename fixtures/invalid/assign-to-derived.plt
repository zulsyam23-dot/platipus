app Main {
    state count = 0
    derived total = count * 2
    Button "reset" {
        on click {
            total = 0
        }
    }
}
