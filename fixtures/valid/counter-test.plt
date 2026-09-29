// `test` blocks run against the program itself: `click` acts on a node by the
// text a person would read, and `expect` reads the program's own state. Each
// test starts from a fresh mount, so a test has to do the clicks it depends on.
app Counter {
    state count = 0
    state step = 1
    state doubled = 0

    derived total = count * 2

    Column {
        Text count
        Text total
        Button "+" {
            on click {
                count = count + step
            }
        }
        Button "reset" {
            on click {
                count = 0
            }
        }
    }
}

test startsAtZero {
    expect count == 0
    expect total == 0
}

test clickingAddsAStep {
    click "+"
    expect count == 1
    expect total == 2
}

test clickingTwiceCountsBothSteps {
    click "+"
    click "+"
    expect count == 2
    expect total == 4
}

test resetReturnsToZero {
    click "+"
    click "reset"
    expect count == 0
}
