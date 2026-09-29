// Collection utilities and the calculator demo.
component UtilSection {
    state names = ["delta", "alpha", "charlie", "bravo"]
    state order = "asc"
    state pageIndex = 0
    state out = ""

    fn refresh() {
        out = page(sortBy(names, null, order), pageIndex, 2)
    }

    Column {
        gap: 16
        SectionIntro {
            eyebrow: "COLLECTIONS"
            title: "Sort & paginate"
            description: "Compose sortBy() and page() to control which values are displayed."
        }
        Row {
            Button "sort asc" {
                on click {
                    order = "asc"
                    refresh()
                }
            }
            Button "sort desc" {
                on click {
                    order = "desc"
                    refresh()
                }
            }
            Button "page 0" {
                on click {
                    pageIndex = 0
                    refresh()
                }
            }
            Button "page 1" {
                on click {
                    pageIndex = 1
                    refresh()
                }
            }
        }
        Grid columns: 2 {
            Card {
                padding: 14
                Column {
                    Text "Visible page"
                    Text out
                }
            }
            Card {
                padding: 14
                Column {
                    Text "Source list"
                    for name in names { Text name }
                }
            }
        }
    }
}

component CalculatorSection {
    state display = 0
    state previous = 0
    state pending = ""
    state startNew = true
    state status = ""

    fn enterDigit(digit: Int) {
        status = ""
        if startNew {
            display = digit
            startNew = false
        } else {
            if display < 0 {
                display = display * 10 - digit
            } else {
                display = display * 10 + digit
            }
        }
    }

    fn calculate() {
        if pending == "+" { display = previous + display }
        if pending == "-" { display = previous - display }
        if pending == "*" { display = previous * display }
        if pending == "/" {
            if display == 0 {
                status = "Cannot divide by zero"
            } else {
                display = previous / display
            }
        }
        pending = ""
        startNew = true
    }

    fn chooseOperator(next: String) {
        if pending != "" && !startNew {
            calculate()
        }
        previous = display
        pending = next
        startNew = true
        status = ""
    }

    fn resetCalculator() {
        display = 0
        previous = 0
        pending = ""
        startNew = true
        status = ""
    }

    Card {
        padding: 18
        width: 360
        Column {
            gap: 10
            SectionIntro {
                eyebrow: "INTERACTIVE DEMO"
                title: "Calculator"
                description: "Four operations, percent, sign toggle, and safe division."
            }
            Card {
                padding: 14
                Column {
                    Text pending
                    Text display { size: 34 weight: 700 }
                    Text status { color: "#fb7185" }
                }
            }
            Row {
                Button "C" { on click { resetCalculator() } }
                Button "±" {
                    on click {
                        display = 0 - display
                        startNew = false
                    }
                }
                Button "%" { on click { display = display / 100 } }
                Button "÷" { on click { chooseOperator("/") } }
            }
            Row {
                Button "7" { on click { enterDigit(7) } }
                Button "8" { on click { enterDigit(8) } }
                Button "9" { on click { enterDigit(9) } }
                Button "×" { on click { chooseOperator("*") } }
            }
            Row {
                Button "4" { on click { enterDigit(4) } }
                Button "5" { on click { enterDigit(5) } }
                Button "6" { on click { enterDigit(6) } }
                Button "−" { on click { chooseOperator("-") } }
            }
            Row {
                Button "1" { on click { enterDigit(1) } }
                Button "2" { on click { enterDigit(2) } }
                Button "3" { on click { enterDigit(3) } }
                Button "+" { on click { chooseOperator("+") } }
            }
            Row {
                Button "0" { on click { enterDigit(0) } }
                Button "00" {
                    on click {
                        enterDigit(0)
                        enterDigit(0)
                    }
                }
                Button "=" { on click { calculate() } }
            }
        }
    }
}
