// Workloads for the repeatable performance harness in `tests/web/benchmark.mjs`.
//
// Nothing here asserts a duration. The fixture exists so the harness has
// something real to time, and so the numbers it prints can be compared between
// runs on the same machine. Correctness is asserted; speed is measured.
//
// The sizes are chosen to be large enough to dominate the noise floor and small
// enough that `cargo test` stays quick: a million counted iterations, a hundred
// thousand list elements pushed, mapped and folded, and a thousand calls into a
// recursively written function.

app Benchmark {
    state rangeTotal = 0
    state whileTotal = 0
    state listTotal = 0
    state fibTotal = 0

    /// Plain recursion, so the cost of entering a Platipus function from a loop
    /// is visible next to the loops themselves.
    fn fib(n: Int) -> Int {
        if n < 2 {
            return n
        }
        return fib(n - 1) + fib(n - 2)
    }

    /// Applies `f` to every element, so the list pass costs a closure call per
    /// item rather than a direct loop body.
    fn mapEach(values: Any, f: Any) -> Any {
        let mapped = []
        for value in values {
            mapped[len(mapped)] = f(value)
        }
        return mapped
    }

    /// A counted range: the loop bounds are computed once and the counter is a
    /// plain JavaScript integer.
    fn countWithRange() -> Int {
        let total = 0
        for i in 0..1_000_000 {
            total = total + i
        }
        return total
    }

    /// A while loop, where the bound is re-read on every pass.
    fn countWithWhile() -> Int {
        let total = 0
        let i = 0
        while i < 1_000_000 {
            total = total + i
            i = i + 1
        }
        return total
    }

    /// Growing a list by index and then folding a lambda over it, which is where
    /// per-element cost shows up if the runtime boxes anything.
    fn buildAndFold() -> Int {
        let values = []
        let i = 0
        while i < 100_000 {
            values[len(values)] = i
            i = i + 1
        }
        let doubled = mapEach(values, x => x * 2)
        let total = 0
        for value in doubled {
            total = total + value
        }
        return total
    }

    /// A thousand calls into a recursive function. The argument is taken modulo
    /// eighteen because plain Fibonacci is exponential: `fib(29)` alone is
    /// hundreds of thousands of calls, which would measure recursion depth
    /// rather than the cost of entering a function once.
    fn fibSum() -> Int {
        let total = 0
        let i = 0
        while i < 1_000 {
            total = total + fib(i % 18)
            i = i + 1
        }
        return total
    }

    Column {
        Text rangeTotal
        Text whileTotal
        Text listTotal
        Text fibTotal
        // One button per workload, so the harness can time each one on its own
        // instead of charging every measurement for all four.
        Button "range" {
            on click { rangeTotal = countWithRange() }
        }
        Button "while" {
            on click { whileTotal = countWithWhile() }
        }
        Button "list" {
            on click { listTotal = buildAndFold() }
        }
        Button "calls" {
            on click { fibTotal = fibSum() }
        }
        Button "all" {
            on click {
                rangeTotal = countWithRange()
                whileTotal = countWithWhile()
                listTotal = buildAndFold()
                fibTotal = fibSum()
            }
        }
    }
}
