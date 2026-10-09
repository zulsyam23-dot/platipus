// Real end-to-end test of the computation rules: operator precedence, bitwise
// grouping, ranges, loops, closures, the standard library, and derived state.
// Every `expect` reads back a value the program actually computed.

fn square(x: Int) -> Int {
    return x * x
}

fn sumTo(n: Int) -> Int {
    let total = 0
    let i = 1
    while i <= n {
        total = total + i
        i = i + 1
    }
    return total
}

fn maskOr(a: Int, b: Int) -> Int {
    return a | b
}

fn buzzScore(n: Int) -> Int {
    if n % 15 == 0 {
        return 1
    }
    if n % 3 == 0 {
        return 10
    }
    if n % 5 == 0 {
        return 100
    }
    return 1000
}

app Compute {
    state prec = 0
    state bitwise = 0
    state unary = 0
    state loops = 0
    state closure = 0
    state lists = ""
    state maps = ""
    state text = ""
    state math = 0.0
    state clock = 0

    // derived re-runs whenever `base` or `factor` changes.
    state base = 6
    state factor = 7
    derived scaled = base * factor
    derived squared = square(scaled)

    fn runAll() {
        // Precedence: * binds tighter than +, so 1 + 2*3 = 7.
        prec = 1 + 2 * 3
        // Shift binds looser than addition: 1 << (2 + 3) = 32.
        let shifted = 1 << 2 + 3
        // Bitwise groups tighter than equality: (5 & 3) == 1.
        let masked = 5 & 3
        let maskOk = 0
        if masked == 1 {
            maskOk = 100
        }
        bitwise = shifted + maskOk
        // Unary minus, an or of two hex literals, and a not masked to a byte.
        unary = -(-5) + maskOr(0xF0, 0x0F) + (~0 & 0xFF)

        let squares = 0
        for i in 0..5 {
            squares = squares + square(i)
        }
        let ranged = 0
        for i in 1..=4 {
            ranged = ranged + i
        }
        loops = squares + ranged + sumTo(10)

        let double = x => x * 2
        let add = (a, b) => a + b
        let nums = [1, 2, 3, 4]
        let evens = nums.filter(x => x % 2 == 0)
        closure = add(double(21), first(evens))

        lists = join(take(nums, 2), ",") + "|" + join(reverse(nums), ",") + "|" + toText(len(nums))

        let table = { b: 2, a: 1 }
        let wide = { x: 1, y: 2, z: 3 }
        maps = join(keys(table), ",") + "|" + toText(get(table, "a")) + "|" + toText(len(wide))

        text = upper("platipus") + "|" + replace("a-b-a", "a", "z") + "|" + repeat("ab", 3) + "|" + trim("  pad  ") + "|" + join(split("x,y,z", ","), "/")

        math = sqrt(144.0) + pow(2.0, 10.0) + clamp(50.0, 0.0, 10.0) + round(2.6)

        clock = (sumTo(5) * 2) + buzzScore(15) + buzzScore(3) + buzzScore(5) + buzzScore(7)
    }

    Column {
        Text prec
        Text bitwise
        Text unary
        Text loops
        Text closure
        Text lists
        Text maps
        Text text
        Text math
        Text clock
        Text scaled
        Text squared
        Button "run" {
            on click { runAll() }
        }
        Button "bump" {
            on click { base = base + 1 }
        }
    }
}

test arithmeticPrecedence {
    click "run"
    expect prec == 7
}

test bitwiseGroupingAndShifts {
    click "run"
    expect bitwise == 132
}

test unaryOperators {
    click "run"
    expect unary == 515
}

test loopsAndRanges {
    click "run"
    expect loops == 95
}

test closuresAndLists {
    click "run"
    expect closure == 44
    expect lists == "1,2|4,3,2,1|4"
    expect maps == "a,b|1|3"
}

test textAndMathBuiltins {
    click "run"
    expect text == "PLATIPUS|z-b-z|ababab|pad|x/y/z"
    expect math == 1049.0
}

test recursionAndBranching {
    click "run"
    expect clock == 1141
}

test derivedTracksItsDependencies {
    expect scaled == 42
    expect squared == 1764
    click "bump"
    expect scaled == 49
    expect squared == 2401
}
