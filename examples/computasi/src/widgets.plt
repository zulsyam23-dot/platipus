// Widgets that present the numbers `lib.plt` computes.
//
// These components carry no colours of their own. Every colour they use is a
// `--plt-*` token, so a program that declares a `theme` restyles all of them at
// once and a program that declares none still gets the base palette and the
// dark-mode variant the runtime ships. That is what keeps them library
// components rather than a private look: the library decides the shape, the
// application decides the colours.
//
// One constraint shapes the whole design, so it is worth stating plainly. A
// `style { }` block is a **stylesheet**, and a stylesheet cannot read state. The
// checker enforces this with `style-value-not-literal`. It follows that:
//
//   * anything static -- a colour, a radius, a transition -- goes in `style`
//     and must be a literal;
//   * anything dynamic -- a bar's height, a meter's fill -- goes in an element
//     property, which the compiler emits as an inline declaration on that node.
//
// The tiles lean on a second CSS fact to stay short: a custom property set on a
// wrapper is inherited by everything inside it, so three tiles can share one
// body while each supplies its own rule colour as a literal.
//
// `widgets.plt` is loaded through `import` from `lib.plt`, so the same
// components, the same functions, and the same tests travel with the package.

// One figure with its label and an optional footnote, ruled on the left with
// the accent. This is the neutral case.
component StatTile {
    input label = ""
    input value = ""
    input hint = ""

    Column {
        style {
            "--plt-tile-rule": "3px solid var(--plt-accent)"
        }
        StatBody label: label value: value hint: hint
    }
}

// The same figure, ruled in the success colour. A result that came out right
// should not have to be read carefully to be told apart from one that did not.
component SuccessTile {
    input label = ""
    input value = ""
    input hint = ""

    Column {
        style {
            "--plt-tile-rule": "3px solid var(--plt-success)"
        }
        StatBody label: label value: value hint: hint
    }
}

// The same figure again, ruled in the danger colour.
component DangerTile {
    input label = ""
    input value = ""
    input hint = ""

    Column {
        style {
            "--plt-tile-rule": "3px solid var(--plt-danger)"
        }
        StatBody label: label value: value hint: hint
    }
}

// The shared body of the three tiles. The rule is read from the custom property
// the tile set, which is what lets one body serve three colours without any of
// them being a dynamic style value.
component StatBody {
    input label = ""
    input value = ""
    input hint = ""

    Card {
        padding: 16
        style {
            background: "var(--plt-surface)"
            border: "1px solid var(--plt-border)"
            "border-radius": "var(--plt-radius)"
            "box-shadow": "var(--plt-shadow)"
            "border-left": "var(--plt-tile-rule, 1px solid var(--plt-border))"
        }
        Column {
            gap: 4
            Text label {
                style {
                    color: "var(--plt-muted)"
                    "font-size": "12px"
                    "letter-spacing": "0.06em"
                    "text-transform": "uppercase"
                }
            }
            Text value {
                style {
                    color: "var(--plt-text)"
                    "font-size": "28px"
                    "font-weight": 600
                    "line-height": "1.2"
                }
            }
            Text hint {
                style {
                    color: "var(--plt-muted)"
                    "font-size": "13px"
                }
            }
        }
    }
}

// A labelled proportion. The percentage is clamped here rather than trusted from
// the caller, because a fill wider than its track is the one thing a meter must
// never do.
component Meter {
    input label = ""
    input percent = 0.0
    input caption = ""

    derived safe = clamp(percent, 0.0, 100.0)
    derived width = toText(round(safe * 100.0) / 100.0) + "%"

    Column {
        gap: 6
        style {
            width: "100%"
        }
        Row {
            style {
                justify: "space-between"
                "align-items": "baseline"
            }
            Text label {
                style {
                    color: "var(--plt-text)"
                    "font-size": "14px"
                    "font-weight": 500
                }
            }
            Text caption {
                style {
                    color: "var(--plt-muted)"
                    "font-size": "13px"
                }
            }
        }
        Row {
            height: "8px"
            style {
                width: "100%"
                "border-radius": "999px"
                background: "var(--plt-surface-2)"
                overflow: "hidden"
            }
            Row {
                // The fill's width depends on the value, so it is an element
                // property rather than a style declaration.
                width: width
                height: "100%"
                style {
                    "border-radius": "999px"
                    background: "var(--plt-accent)"
                    transition: "width 200ms ease"
                }
            }
        }
    }
}

// A row of small bars, each scaled against the tallest value in the series.
// Good for a prime-density run or a Collatz trajectory, where the shape of the
// thing matters more than the exact figures.
component SparkBars {
    input values = []
    input height = 40

    derived heights = barHeights(values)
    derived heightText = toText(height) + "px"

    Row {
        style {
            "align-items": "flex-end"
            gap: 3
            width: "100%"
        }
        height: heightText
        for bar in heights {
            Bar value: bar
        }
    }
}

// One bar of a `SparkBars`. The height is an element property because it comes
// from the data; everything about how a bar looks is a literal.
component Bar {
    input value = 0.0

    Row {
        width: "6px"
        height: value
        style {
            "border-radius": "3px 3px 0 0"
            background: "var(--plt-accent)"
            transition: "height 200ms ease"
        }
    }
}

// A small pill for a value that deserves emphasis but not a whole row: a prime,
// a factor, a step count.
component Chip {
    input value = ""

    Row {
        padding: 3
        style {
            "border-radius": "999px"
            background: "var(--plt-surface-2)"
        }
        Text value {
            style {
                color: "var(--plt-muted)"
                "font-size": "12px"
                "font-weight": 500
            }
        }
    }
}

// The same pill, filled with the accent, for the one value in a row that is the
// answer rather than one of the inputs.
component AccentChip {
    input value = ""

    Row {
        padding: 3
        style {
            "border-radius": "999px"
            background: "var(--plt-accent)"
        }
        Text value {
            style {
                color: "var(--plt-accent-text)"
                "font-size": "12px"
                "font-weight": 500
            }
        }
    }
}

// A titled panel with a grid of chips underneath. This is how a widget asks
// "and what else did you find?" without the caller hand-building a heading and
// a wrapping grid every time.
component FindingPanel {
    input title = ""
    input items = []
    input columns = 6

    Card {
        padding: 16
        style {
            background: "var(--plt-surface)"
            border: "1px solid var(--plt-border)"
            "border-radius": "var(--plt-radius)"
        }
        Column {
            gap: 12
            Text title {
                style {
                    color: "var(--plt-text)"
                    "font-size": "15px"
                    "font-weight": 600
                }
            }
            Grid {
                columns: columns
                gap: 6
                for item in items {
                    Chip value: toText(item)
                }
            }
        }
    }
}

// A titled card holding a list of `ResultRow`s, with a heading rule that ties
// it to the section it sits in.
component ResultList {
    input title = ""
    input rows = []

    Card {
        padding: 18
        style {
            background: "var(--plt-surface)"
            border: "1px solid var(--plt-border)"
            "border-radius": "var(--plt-radius)"
            "box-shadow": "var(--plt-shadow)"
        }
        Column {
            gap: 10
            Text title {
                style {
                    color: "var(--plt-text)"
                    "font-size": "14px"
                    "font-weight": 600
                    "text-transform": "uppercase"
                    "letter-spacing": "0.06em"
                    padding: "0 0 8px 0"
                    "border-bottom": "2px solid var(--plt-accent)"
                }
            }
            for row in rows {
                ResultRow label: get(row, "label") value: get(row, "value")
            }
        }
    }
}

// One label/value line. Its own component so the two cells can be laid out
// independently without the caller repeating the row.
component ResultRow {
    input label = ""
    input value = ""

    Row {
        style {
            justify: "space-between"
            "align-items": "baseline"
            gap: 16
            padding: "5px 0"
            "border-bottom": "1px solid var(--plt-border)"
        }
        Text label {
            style {
                color: "var(--plt-muted)"
                "font-size": "13px"
            }
        }
        Text value {
            style {
                color: "var(--plt-text)"
                "font-family": "var(--plt-mono)"
                "font-size": "13px"
                "font-weight": 600
            }
        }
    }
}

// -- figures the widgets are filled with --------------------------------------

/// The label/value pairs a `ResultList` is usually handed.
fn resultPairs() -> Any {
    return [
        { label: "gcd(1071, 462)", value: toText(gcd(1071, 462)) },
        { label: "lcm(4, 6)", value: toText(lcm(4, 6)) },
        { label: "fib(30)", value: toText(fib(30)) },
        { label: "factorial(15)", value: toText(factorial(15)) },
        { label: "collatz(27)", value: toText(collatzSteps(27)) + " langkah" },
        { label: "fnv1a32 platipus", value: toText(fnv1a32("platipus")) },
    ]
}

/// Prime density under 200 as a percentage, which is what `Meter` wants.
fn primeDensity() -> Float {
    return round(len(primesUpTo(200)) * 10000.0 / 200.0) / 100.0
}

/// The Collatz trajectory of 27, ready for `SparkBars`.
fn collatzBars() -> Any {
    return collatzSequence(27)
}

/// The primes under 100, ready for `FindingPanel`.
fn smallPrimes() -> Any {
    return primesUpTo(100)
}

/// Bar heights for a series, each as a percentage of the tallest value. A flat
/// or empty series is left flat rather than divided by zero.
fn barHeights(values: Any) -> Any {
    let peak = maxOf(values)
    if peak <= 0 {
        return []
    }
    let peakFloat = peak * 1.0
    return mapEach(values, v => round(clamp(v * 100.0 / peakFloat, 4.0, 100.0)))
}

// -- tests -------------------------------------------------------------------
//
// A component has nothing to assert on its own -- it produces nodes, not
// numbers -- so what these check is the computation that fills a widget. If a
// figure here is wrong, every tile that shows it is wrong too.

test widgetFigures {
    expect toText(get(resultPairs()[0], "value")) == "21"
    expect toText(get(resultPairs()[1], "value")) == "12"
    expect toText(get(resultPairs()[2], "value")) == "832040"
    expect toText(get(resultPairs()[4], "value")) == "111 langkah"
    expect primeDensity() == 23.0
    expect len(collatzBars()) == 112
    expect maxOf(collatzBars()) == 9232
    expect len(smallPrimes()) == 25
}

test barsAreScaledAgainstTheTallestValue {
    expect join(barHeights([1, 2, 4]), ",") == "25,50,100"
    expect join(barHeights([5, 5]), ",") == "100,100"
    expect join(barHeights([0, 0]), ",") == ""
    expect join(barHeights([]), ",") == ""
}
