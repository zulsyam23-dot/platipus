// The acceptance package for pure Platipus computation.
//
// This file is the evidence that the language can carry real algorithms on its
// own: no `#[rust]` block, no `app`, no host helper. Every function below is
// written in Platipus and runs in the generated JavaScript, and every `test`
// block at the bottom states the value the program actually produced -- not that
// it merely compiled.

// -- number theory -----------------------------------------------------------

/// The greatest common divisor by Euclid's algorithm. An argument of zero is
/// defined the usual way, so `gcd(0, n)` is `abs(n)` and `gcd(0, 0)` is zero.
fn gcd(a: Int, b: Int) -> Int {
    let left = abs(a)
    let right = abs(b)
    while right != 0 {
        let remainder = left % right
        left = right
        right = remainder
    }
    return left
}

/// The least common multiple. A zero argument has no meaningful multiple, so it
/// reports zero instead of dividing by zero.
fn lcm(a: Int, b: Int) -> Int {
    if a == 0 || b == 0 {
        return 0
    }
    return idiv(abs(a * b), gcd(a, b))
}

/// Whether `n` is prime. Numbers below two are not, and only odd divisors up
/// to the square root are tried, which is what keeps it from walking to `n`.
fn isPrime(n: Int) -> Bool {
    if n < 2 {
        return false
    }
    if n < 4 {
        return true
    }
    if n % 2 == 0 {
        return false
    }
    let divisor = 3
    while divisor * divisor <= n {
        if n % divisor == 0 {
            return false
        }
        divisor = divisor + 2
    }
    return true
}

/// The `n`th Fibonacci number, counted from `fib(0) == 0`. It is the iterative
/// form, so the cost is linear rather than exponential in the argument.
fn fib(n: Int) -> Int {
    if n < 0 {
        return 0
    }
    let previous = 0
    let current = 1
    let step = 0
    while step < n {
        let next = previous + current
        previous = current
        current = next
        step = step + 1
    }
    return previous
}

/// `n` factorial, with `factorial(0) == 1`. Beyond the point where the product
/// passes the safe integer range the answer stops being exact; see the
/// `integersLosePrecision` test for where that boundary is.
fn factorial(n: Int) -> Int {
    if n < 0 {
        return 0
    }
    let product = 1
    let factor = 2
    while factor <= n {
        product = product * factor
        factor = factor + 1
    }
    return product
}

/// How many steps `n` needs to reach one under Collatz: halve an even number,
/// or send an odd one to `3n + 1`. One needs no steps, and the `seen` guard
/// stops a non-terminating input instead of hanging the program.
fn collatzSteps(n: Int) -> Int {
    if n < 1 {
        return -1
    }
    let current = n
    let steps = 0
    let seen = []
    while current != 1 {
        if contains(seen, current) {
            return -1
        }
        seen[len(seen)] = current
        if current % 2 == 0 {
            current = idiv(current, 2)
        } else {
            current = current * 3 + 1
        }
        steps = steps + 1
    }
    return steps
}

// -- lists -------------------------------------------------------------------

/// The total of a list. An empty list totals zero.
fn sum(values: Any) -> Int {
    let total = 0
    for value in values {
        total = total + value
    }
    return total
}

/// The product of a list, which is one for an empty list.
fn product(values: Any) -> Int {
    let result = 1
    for value in values {
        result = result * value
    }
    return result
}

/// The largest value in a list, or zero when it is empty.
fn maxOf(values: Any) -> Int {
    let best = 0
    let first = true
    for value in values {
        if first {
            best = value
            first = false
        } else {
            if value > best {
                best = value
            }
        }
    }
    return best
}

/// The smallest value in a list, or zero when it is empty.
fn minOf(values: Any) -> Int {
    let best = 0
    let first = true
    for value in values {
        if first {
            best = value
            first = false
        } else {
            if value < best {
                best = value
            }
        }
    }
    return best
}

/// A sorted copy of `values`, ascending.
//
// The sort never writes to `values`. That is the point of the function rather
/// than an accident of the implementation: insertion sort builds a fresh list
/// and shifts within it, so the caller's list keeps its original order and a
/// second sort of the same list sees the same input.
fn sortAsc(values: Any) -> Any {
    let sorted = []
    for value in values {
        let index = len(sorted) - 1
        while index >= 0 {
            if sorted[index] <= value {
                break
            }
            sorted[index + 1] = sorted[index]
            index = index - 1
        }
        sorted[index + 1] = value
    }
    return sorted
}

/// Where `target` sits in an ascending list, or `-1` when it is absent. The
/// list has to be sorted already; the function does not sort it for the caller.
fn binarySearch(sorted: Any, target: Int) -> Int {
    let low = 0
    let high = len(sorted) - 1
    while low <= high {
        let middle = low + idiv(high - low, 2)
        let value = sorted[middle]
        if value == target {
            return middle
        }
        if value < target {
            low = middle + 1
        } else {
            high = middle - 1
        }
    }
    return -1
}

/// Applies `f` to every element, in order. The lambda is the argument, so this
/// is also the evidence that a closure survives being passed into a function.
fn mapEach(values: Any, f: Any) -> Any {
    let mapped = []
    for value in values {
        mapped[len(mapped)] = f(value)
    }
    return mapped
}

/// Every element that satisfies `keep`, in order.
fn filterEach(values: Any, keep: Any) -> Any {
    let kept = []
    for value in values {
        if keep(value) {
            kept[len(kept)] = value
        }
    }
    return kept
}

// -- text --------------------------------------------------------------------

/// Whether `text` reads the same forwards and backwards. The comparison is exact:
/// a capital letter is not its own mirror, and spaces count.
fn isPalindrome(text: String) -> Bool {
    let front = 0
    let back = len(text) - 1
    while front < back {
        if codeAt(text, front) != codeAt(text, back) {
            return false
        }
        front = front + 1
        back = back - 1
    }
    return true
}

/// The 32-bit FNV-1a hash of `text`, as an unsigned value.
///
/// The two steps of the algorithm are `hash = (hash xor byte) * prime` truncated
/// to 32 bits. `u32` is that truncation, and `imul` is the multiply that keeps
/// only the low 32 bits, so the hash is computed in Platipus without any 64-bit
/// intermediate to lose precision in.
fn fnv1a32(text: String) -> Int {
    let hash = 2166136261
    let index = 0
    while index < len(text) {
        hash = u32(hash ^ codeAt(text, index))
        hash = u32(imul(hash, 16777619))
        index = index + 1
    }
    return hash
}

// -- reports the test runner cannot build on its own -------------------------
//
// A `test` block in a library module holds actions and expectations only, so a
// test that needs a local binding has to ask a function for it. These helpers
// exist for that reason and for no other.

// Sorting must leave its argument alone, which needs both lists in one value.
fn sortReport() -> String {
    let original = [5, 3, 9, 1]
    return join(original, ",") + " then " + join(sortAsc(original), ",")
}

// Two sorts of one list have to agree, which is the same claim seen from the
// other side: nothing about the first call carried into the second.
fn sortTwiceReport() -> String {
    let original = [8, 2, 7, 4]
    let once = sortAsc(original)
    let twice = sortAsc(once)
    return join(once, ",") + " then " + join(twice, ",")
}

// A lambda closing over nothing still has to arrive as a value.
fn mapReport() -> String {
    return join(mapEach([1, 2, 3, 4], x => x * 10), ",") + " / " + join(filterEach([1, 2, 3, 4], x => x % 2 == 0), ",")
}

// `codeAt` is what both text functions stand on, so it is checked directly.
fn codePointReport() -> String {
    return toText(codeAt("abc", 0)) + "," + toText(codeAt("abc", 2)) + "," + toText(codeAt("abc", 3)) + "," + toText(codeAt("abc", -1)) + "," + toText(codeAt("é", 0))
}

// Where the integer range stops being exact, measured rather than assumed.
//
// `Int` reaches JavaScript as a `Number`, so it holds every integer exactly only
// up to 2^53 - 1 = 9007199254740991. The three values below are what this
// program really produces; the true ones are 8944394323791464,
// 14472334024676221, 23416728348467685, and 2432902008176640000, so the last
// two Fibonacci terms lose one unit while the factorial does not.
fn precisionReport() -> String {
    return toText(fib(78)) + "," + toText(fib(79)) + "," + toText(fib(80)) + "," + toText(factorial(20))
}

// -- tests -------------------------------------------------------------------

test gcdAndLcm {
    expect gcd(48, 18) == 6
    expect gcd(270, 192) == 6
    expect gcd(17, 17) == 17
    expect gcd(0, 5) == 5
    expect gcd(-12, 18) == 6
    expect lcm(4, 6) == 12
    expect lcm(21, 6) == 42
    expect lcm(0, 5) == 0
}

test primality {
    expect isPrime(2) == true
    expect isPrime(3) == true
    expect isPrime(97) == true
    expect isPrime(7919) == true
    expect isPrime(1) == false
    expect isPrime(0) == false
    expect isPrime(-7) == false
    expect isPrime(100) == false
    expect isPrime(7917) == false
    expect isPrime(7919 * 7919) == false
}

test fibonacci {
    expect fib(0) == 0
    expect fib(1) == 1
    expect fib(2) == 1
    expect fib(10) == 55
    expect fib(20) == 6765
    expect fib(30) == 832040
    expect fib(-4) == 0
}

test factorial {
    expect factorial(0) == 1
    expect factorial(1) == 1
    expect factorial(5) == 120
    expect factorial(10) == 3628800
    expect factorial(15) == 1307674368000
    expect factorial(-3) == 0
}

test collatz {
    expect collatzSteps(1) == 0
    expect collatzSteps(2) == 1
    expect collatzSteps(6) == 8
    expect collatzSteps(27) == 111
    expect collatzSteps(97) == 118
    expect collatzSteps(0) == -1
}

test listOperations {
    expect sum([1, 2, 3, 4]) == 10
    expect sum([]) == 0
    expect product([2, 3, 4]) == 24
    expect product([]) == 1
    expect maxOf([3, 9, 2]) == 9
    expect minOf([3, 9, 2]) == 2
    expect maxOf([-4, -9]) == -4
    expect minOf([]) == 0
    expect join(reverse([1, 2, 3]), ",") == "3,2,1"
    expect join(unique([1, 2, 2, 3, 3, 3]), ",") == "1,2,3"
    expect len(merge({ a: 1 }, { b: 2 })) == 2
}

test sortingDoesNotChangeItsInput {
    expect sortReport() == "5,3,9,1 then 1,3,5,9"
    expect sortTwiceReport() == "2,4,7,8 then 2,4,7,8"
    expect join(sortAsc([]), ",") == ""
    expect join(sortAsc([1]), ",") == "1"
    expect join(sortAsc([2, 2, 1]), ",") == "1,2,2"
}

test searchFindsAndRejects {
    expect binarySearch([1, 3, 5, 7, 9], 7) == 3
    expect binarySearch([1, 3, 5, 7, 9], 1) == 0
    expect binarySearch([1, 3, 5, 7, 9], 9) == 4
    expect binarySearch([1, 3, 5, 7, 9], 4) == -1
    expect binarySearch([1, 3, 5, 7, 9], 0) == -1
    expect binarySearch([1, 3, 5, 7, 9], 10) == -1
    expect binarySearch([], 1) == -1
    expect binarySearch([2], 2) == 0
}

test lambdasMapAndFilter {
    expect mapReport() == "10,20,30,40 / 2,4"
    expect join(mapEach([], x => x), ",") == ""
    expect join(mapEach([1, 2, 3], x => x * x), ",") == "1,4,9"
}

test palindromes {
    expect isPalindrome("racecar") == true
    expect isPalindrome("kayak") == true
    expect isPalindrome("a") == true
    expect isPalindrome("") == true
    expect isPalindrome("ab ba") == true
    expect isPalindrome("abc") == false
    expect isPalindrome("Racecar") == false
    expect isPalindrome("ab ca") == false
}

test integerDivisionTruncates {
    expect idiv(7, 2) == 3
    expect idiv(-7, 2) == -3
    expect idiv(8, 2) == 4
    expect idiv(1, 2) == 0
    expect idiv(9, 3) == 3
}

test fnv1a {
    expect fnv1a32("") == 2166136261
    expect fnv1a32("a") == 3826002220
    expect fnv1a32("foobar") == 3214735720
}

test codePointLookup {
    expect codePointReport() == "97,99,-1,-1,233"
}

test integersLosePrecision {
    expect precisionReport() == "8944394323791464,14472334024676220,23416728348467684,2432902008176640000"
    expect 9007199254740991 + 1 == 9007199254740992
}
