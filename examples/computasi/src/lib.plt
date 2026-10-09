// `computasi` -- computation written in Platipus, with no `#[rust]` block and
// no host helper. Every function here is called by `widgets.plt` to fill a
// widget, and every one of them is pinned by a `test` block at the bottom.
//
// The two things this library is careful about:
//
//   * a sorted result is always a fresh list, so sorting never rewrites the
//     caller's argument;
//   * a number that cannot be represented exactly is not quietly returned as a
//     nearby number. `/` is true division, so `idiv` is what integer division
//     means, and `Int` reaching a browser is a JavaScript `Number` exact only up
//     to 2^53 - 1.

import Widget from "./widgets.plt"

// -- number theory -----------------------------------------------------------

/// The greatest common divisor by Euclid's algorithm.
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

/// The least common multiple. A zero argument has no meaningful multiple.
fn lcm(a: Int, b: Int) -> Int {
    if a == 0 || b == 0 {
        return 0
    }
    return idiv(abs(a * b), gcd(a, b))
}

/// Whether `n` is prime. Only odd divisors up to the square root are tried.
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

/// Every prime up to and including `limit`, in order.
///
/// This is a sieve of Eratosthenes: once a number is known to be prime, its
/// multiples from its square onwards are marked composite, so a composite is
/// marked once instead of being divided by every possible divisor as `isPrime`
/// would do on its own.
fn primesUpTo(limit: Int) -> Any {
    if limit < 2 {
        return []
    }
    let composite = []
    let found = []
    let candidate = 2
    while candidate <= limit {
        if !contains(composite, candidate) {
            found[len(found)] = candidate
            let multiple = candidate * candidate
            while multiple <= limit {
                composite[len(composite)] = multiple
                multiple = multiple + candidate
            }
        }
        candidate = candidate + 1
    }
    return found
}

/// The distinct prime factors of `n`, ascending. Repeated factors appear once,
/// so `primeFactors(360)` is `2, 3, 5` rather than `2, 2, 2, 3, 3, 5`; a number
/// below two has none. `primeFactorPairs` is the form that keeps the exponents.
fn primeFactors(n: Int) -> Any {
    if n < 2 {
        return []
    }
    let remaining = n
    let distinct = []
    let divisor = 2
    while divisor * divisor <= remaining {
        if remaining % divisor == 0 {
            distinct[len(distinct)] = divisor
            while remaining % divisor == 0 {
                remaining = idiv(remaining, divisor)
            }
        }
        divisor = divisor + 1
    }
    if remaining > 1 {
        distinct[len(distinct)] = remaining
    }
    return distinct
}

/// The prime factors of `n` with their exponents, as a list of `factor`,
/// `exponent` pairs in ascending order.
fn primeFactorPairs(n: Int) -> Any {
    if n < 2 {
        return []
    }
    let remaining = n
    let pairs = []
    let divisor = 2
    while divisor * divisor <= remaining {
        if remaining % divisor == 0 {
            let exponent = 0
            while remaining % divisor == 0 {
                remaining = idiv(remaining, divisor)
                exponent = exponent + 1
            }
            pairs[len(pairs)] = { factor: divisor, exponent: exponent }
        }
        divisor = divisor + 1
    }
    if remaining > 1 {
        pairs[len(pairs)] = { factor: remaining, exponent: 1 }
    }
    return pairs
}

/// The `n`th Fibonacci number, counted from `fib(0) == 0`. Iterative, so the
/// cost is linear in the argument.
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

/// `n` factorial, with `factorial(0) == 1`.
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

/// How many steps `n` needs to reach one under Collatz.
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

/// The whole Collatz trajectory of `n`, starting with `n` itself.
fn collatzSequence(n: Int) -> Any {
    if n < 1 {
        return []
    }
    let path = []
    let current = n
    let seen = []
    while current != 1 {
        if contains(seen, current) {
            return path
        }
        seen[len(seen)] = current
        path[len(path)] = current
        if current % 2 == 0 {
            current = idiv(current, 2)
        } else {
            current = current * 3 + 1
        }
    }
    path[len(path)] = 1
    return path
}

/// Every positive divisor of `n`, ascending.
fn divisors(n: Int) -> Any {
    if n < 1 {
        return []
    }
    let found = []
    let candidate = 1
    while candidate * candidate <= n {
        if n % candidate == 0 {
            found[len(found)] = candidate
            if candidate != idiv(n, candidate) {
                found[len(found)] = idiv(n, candidate)
            }
        }
        candidate = candidate + 1
    }
    return sortAsc(found)
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

/// The arithmetic mean of a list, as a float. An empty list has no mean, so it
/// reports zero rather than dividing by zero.
fn mean(values: Any) -> Float {
    if len(values) == 0 {
        return 0.0
    }
    let total = 0.0
    for value in values {
        total = total + value
    }
    return total / (1.0 * len(values))
}

/// The middle value of an already-sorted list, averaging the two middles when
/// the length is even.
fn median(sorted: Any) -> Float {
    let count = len(sorted)
    if count == 0 {
        return 0.0
    }
    let half = idiv(count, 2)
    if count % 2 == 1 {
        return sorted[half] * 1.0
    }
    return (sorted[half - 1] + sorted[half]) / 2.0
}

/// The population variance of a list.
fn variance(values: Any) -> Float {
    let count = len(values)
    if count == 0 {
        return 0.0
    }
    let average = mean(values)
    let total = 0.0
    for value in values {
        let delta = value * 1.0 - average
        total = total + delta * delta
    }
    return total / (1.0 * count)
}

/// A sorted copy of `values`, ascending. Never writes to the caller's list.
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

/// A sorted copy of `values`, descending. Also leaves the caller's list alone.
fn sortDesc(values: Any) -> Any {
    let sorted = []
    for value in values {
        let index = len(sorted) - 1
        while index >= 0 {
            if sorted[index] >= value {
                break
            }
            sorted[index + 1] = sorted[index]
            index = index - 1
        }
        sorted[index + 1] = value
    }
    return sorted
}

/// Where `target` sits in an ascending list, or `-1` when it is absent.
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

/// Applies `f` to every element, in order.
fn mapEach(values: Any, f: Any) -> Any {
    let mapped = []
    for value in values {
        mapped[len(mapped)] = f(value)
    }
    return mapped
}

/// The elements from `start` up to but not including `end`. An empty or
/// backwards range gives an empty list rather than failing.
fn sliceOf(values: Any, start: Int, end: Int) -> Any {
    let sliced = []
    let index = maxOf([start, 0])
    while index < minOf([end, len(values)]) {
        sliced[len(sliced)] = values[index]
        index = index + 1
    }
    return sliced
}

/// `size` elements at a time, the last one possibly shorter. A size below one
/// would not advance the loop, so it gives back an empty list.
fn chunk(values: Any, size: Int) -> Any {
    let blocks = []
    if size < 1 {
        return blocks
    }
    let index = 0
    while index < len(values) {
        blocks[len(blocks)] = sliceOf(values, index, index + size)
        index = index + size
    }
    return blocks
}

/// Each pair of values added together, stopping at the shorter list.
fn zipSum(left: Any, right: Any) -> Any {
    let shortest = minOf([len(left), len(right)])
    let pairs = []
    let index = 0
    while index < shortest {
        pairs[len(pairs)] = left[index] + right[index]
        index = index + 1
    }
    return pairs
}

// -- text --------------------------------------------------------------------

/// Whether `text` reads the same forwards and backwards, exactly.
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

/// How many words `text` holds, where a word is a run of non-space characters.
fn wordCount(text: String) -> Int {
    let words = []
    let current = []
    let index = 0
    while index < len(text) {
        let ch = text[index]
        if ch == " " || ch == "\n" || ch == "\t" {
            if len(current) > 0 {
                words[len(words)] = join(current, "")
                current = []
            }
        } else {
            current[len(current)] = ch
        }
        index = index + 1
    }
    if len(current) > 0 {
        words[len(words)] = join(current, "")
    }
    return len(words)
}

/// The Levenshtein distance between two strings: the fewest single-character
/// insertions, deletions, or substitutions that turn one into the other. Only
/// the previous row of the grid is kept, so the memory cost is one row.
fn editDistance(left: String, right: String) -> Int {
    let previous = []
    let column = 0
    while column <= len(right) {
        previous[len(previous)] = column
        column = column + 1
    }
    let row = 1
    while row <= len(left) {
        let current = []
        current[0] = row
        let inner = 1
        while inner <= len(right) {
            let substitutionCost = 1
            if left[row - 1] == right[inner - 1] {
                substitutionCost = 0
            }
            current[len(current)] = minOf([
                previous[inner - 1] + substitutionCost,
                previous[inner] + 1,
                current[inner - 1] + 1,
            ])
            inner = inner + 1
        }
        previous = current
        row = row + 1
    }
    return previous[len(previous) - 1]
}

/// Whether two texts are anagrams of one another, ignoring case and spacing.
fn isAnagram(left: String, right: String) -> Bool {
    return join(sortAsc(split(lower(replace(left, " ", "")), "")), "") == join(sortAsc(split(lower(replace(right, " ", "")), "")), "")
}

// -- reports the test runner cannot build on its own -------------------------

// A `test` block in a library module holds actions and expectations only, so a
// test that needs a local binding asks a function for it.

fn sortReport() -> String {
    let original = [5, 3, 9, 1]
    return join(original, ",") + " then " + join(sortAsc(original), ",")
}

fn chunkReport() -> String {
    let blocks = chunk([1, 2, 3, 4, 5], 2)
    return join(mapEach(blocks, part => join(part, "-")), " | ")
}

fn statisticsReport() -> String {
    let data = [2, 4, 4, 4, 5, 5, 7, 9]
    return toText(round(mean(data) * 1000.0) / 1000.0) + "|" + toText(round(median(sortAsc(data)) * 1000.0) / 1000.0) + "|" + toText(round(variance(data) * 1000.0) / 1000.0)
}

fn editDistanceReport() -> String {
    return toText(editDistance("kitten", "sitting")) + "|" + toText(editDistance("flaw", "lawn")) + "|" + toText(editDistance("", "abc")) + "|" + toText(editDistance("abc", "abc"))
}

/// Deep equality is not `==`. Two lists holding the same numbers in the same
/// order are still two different lists as far as `==` is concerned, so a test
/// that wants to compare them has to compare their text.
fn listEqualityReport() -> String {
    let left = [1, 2, 3]
    let right = [1, 2, 3]
    return toText(left == right) + "|" + toText(left == left) + "|" + join(left, ",") + "|" + join(right, ",")
}

// -- tests -------------------------------------------------------------------

test numberTheory {
    expect gcd(48, 18) == 6
    expect gcd(0, 5) == 5
    expect gcd(-12, 18) == 6
    expect lcm(4, 6) == 12
    expect lcm(21, 6) == 42
    expect isPrime(97) == true
    expect isPrime(7919) == true
    expect isPrime(1) == false
    expect isPrime(7919 * 7919) == false
    expect join(primeFactors(360), ",") == "2,3,5"
    expect join(primeFactors(97), ",") == "97"
    expect join(primeFactors(1), ",") == ""
    expect len(primeFactorPairs(360)) == 3
    expect toText(get(primeFactorPairs(360)[0], "factor")) == "2"
    expect toText(get(primeFactorPairs(360)[0], "exponent")) == "3"
}

test primesUpToLimit {
    expect join(primesUpTo(2), ",") == "2"
    expect join(primesUpTo(30), ",") == "2,3,5,7,11,13,17,19,23,29"
    expect join(primesUpTo(1), ",") == ""
    expect len(primesUpTo(100)) == 25
}

test sequences {
    expect fib(0) == 0
    expect fib(10) == 55
    expect fib(30) == 832040
    expect fib(-4) == 0
    expect factorial(0) == 1
    expect factorial(5) == 120
    expect factorial(15) == 1307674368000
    expect collatzSteps(1) == 0
    expect collatzSteps(27) == 111
    expect collatzSteps(0) == -1
    expect join(collatzSequence(6), ",") == "6,3,10,5,16,8,4,2,1"
}

test divisors {
    expect join(divisors(1), ",") == "1"
    expect join(divisors(12), ",") == "1,2,3,4,6,12"
    expect join(divisors(28), ",") == "1,2,4,7,14,28"
    expect join(divisors(0), ",") == ""
}

test listArithmetic {
    expect sum([1, 2, 3, 4]) == 10
    expect sum([]) == 0
    expect product([2, 3, 4]) == 24
    expect product([]) == 1
    expect maxOf([3, 9, 2]) == 9
    expect maxOf([-4, -9]) == -4
    expect minOf([3, 9, 2]) == 2
    expect join(zipSum([1, 2, 3], [10, 20, 30, 40]), ",") == "11,22,33"
}

test sortingNeverTouchesItsInput {
    expect sortReport() == "5,3,9,1 then 1,3,5,9"
    expect join(sortAsc([]), ",") == ""
    expect join(sortAsc([2, 2, 1]), ",") == "1,2,2"
    expect join(sortDesc([2, 2, 1]), ",") == "2,2,1"
    expect join(sortDesc([5, 3, 9, 1]), ",") == "9,5,3,1"
}

test searchingAndSlicing {
    expect binarySearch([1, 3, 5, 7, 9], 7) == 3
    expect binarySearch([1, 3, 5, 7, 9], 4) == -1
    expect binarySearch([], 1) == -1
    expect chunkReport() == "1-2 | 3-4 | 5"
    expect join(sliceOf([1, 2, 3, 4, 5], 1, 3), ",") == "2,3"
}

test statistics {
    // mean 5.0, median 4.5, population variance 4.0
    expect statisticsReport() == "5|4.5|4"
}

test lambdas {
    expect join(mapEach([1, 2, 3], x => x * x), ",") == "1,4,9"
    expect join(filterEach([1, 2, 3, 4], x => x % 2 == 0), ",") == "2,4"
}

test textOperations {
    expect isPalindrome("racecar") == true
    expect isPalindrome("Racecar") == false
    expect isPalindrome("") == true
    expect fnv1a32("") == 2166136261
    expect fnv1a32("a") == 3826002220
    expect fnv1a32("foobar") == 3214735720
    expect wordCount("  satu  dua   tiga ") == 3
    expect wordCount("   ") == 0
    expect editDistanceReport() == "3|2|3|0"
    expect isAnagram("listen", "silent") == true
    expect isAnagram("Listen", "Silent ") == true
    expect isAnagram("abc", "abd") == false
}

test equalityOnListsComparesReferences {
    // `==` on a list or a map is reference equality, not a structural one. This
    // is stated here because it is the sort of thing a reader of the widgets
    // would otherwise have to discover the hard way.
    expect listEqualityReport() == "false|true|1,2,3|1,2,3"
}
