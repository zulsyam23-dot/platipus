// Lambdas are plain arrows once compiled, so a click that runs one should
// update state exactly like ordinary code would.
app Lambda {
    state r1 = 0
    state r2 = 0
    state r3 = 0
    state joined = ""

    fn run() {
        let double = x => x * 2
        let add = (a, b) => a + b
        let triple = n => {
            let step = n + n
            return step + n
        }
        let xs = [1, 2, 3]
        let mapped = xs.map(x => x + 10)
        r1 = double(4)
        r2 = add(2, 3)
        r3 = triple(3)
        joined = mapped.join("-")
    }

    Column {
        Text r1
        Text r2
        Text r3
        Text joined
        Button "go" {
            on click { run() }
        }
    }
}
