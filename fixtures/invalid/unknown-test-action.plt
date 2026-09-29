app Main {
    state count = 0
    Text count
}

test bogus {
    hover "+"
    expect count == 0
}
