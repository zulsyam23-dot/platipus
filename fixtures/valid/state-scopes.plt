// The five state scopes in one program, so a change to any of them is caught by
// `cargo test -p platipus-cli --test fixtures` rather than by whoever happens to
// use that scope first.
//
// `shared`, `global`, and `persistent` take a modifier before `state`; a plain
// `state` is `local`, and `derived` is computed from other state. The modifiers
// are keywords, so none of them can also be a state name.
app StateScopes {
    state count = 0
    shared state total = 1
    global state palette = 2
    persistent state settings = 3
    derived doubled = count * 2

    fn bump(by: Int) {
        count += by
        total += by
    }

    fn label() -> String {
        return "scopes"
    }

    Column {
        Text count
        Text total
        Text palette
        Text settings
        Text doubled
        Text label()
        Button "+" {
            on click {
                bump(1)
            }
        }
    }
}
