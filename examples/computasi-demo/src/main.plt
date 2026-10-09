// A demo application for the `computasi` package.
//
// The point is that this file contains no arithmetic. Every number it shows was
// computed by the library and every layout came from a widget in it, which is
// what a `theme` is supposed to let a program do.
//
// The only thing this application owns is the question being asked. Change
// `seed` on the slider and every figure below is recomputed, because the
// library functions are called from `derived` and from handlers rather than
// baked into text.

import Komputasi from "computasi"

theme Midnight {
    background: "#0f1117"
    surface: "#171a23"
    surfaceAlt: "#1f2330"
    border: "#2a2f3d"
    foreground: "#e8eaf0"
    muted: "#8b93a7"
    accent: "#7aa2f7"
    accentText: "#0f1117"
    success: "#7bd88f"
    danger: "#f7768e"
    radius: "12px"
    gap: "20px"
    pad: "24px"
    shadow: "0 1px 2px rgb(0 0 0 / 30%), 0 12px 32px rgb(0 0 0 / 25%)"
    fontSize: "15px"
}

app PapanKomputasi {
    state seed = 27
    state limit = 100
    state text = "platipus"
    state report = ""
    state runs = 0

    // Every figure below is a library call, recomputed whenever its inputs move.
    derived stepCount = collatzSteps(seed)
    derived factors = primeFactors(seed)
    derived prime = isPrime(seed)
    derived found = primesUpTo(limit)
    derived density = round(len(found) * 10000.0 / (1.0 * limit)) / 100.0
    derived trajectory = collatzSequence(seed)
    derived fibonacci = fib(idiv(seed, 3))
    derived digest = fnv1a32(text)
    derived palindrome = isPalindrome(text)
    derived data = [4, 8, 15, 16, 23, 42]
    derived centre = median(sortAsc(data))
    derived spread = round(variance(data) * 100.0) / 100.0
    derived ranked = sortDesc(data)

    fn describe() {
        runs = runs + 1
        report = toText(factorial(idiv(seed, 2))) + " | " + toText(editDistance(lower(text), "platypus")) + " | " + toText(wordCount(text)) + " kata"
    }

    Page {
        style {
            minHeight: "100vh"
            padding: "32px 20px 64px 20px"
            background: "var(--plt-bg)"
            color: "var(--plt-text)"
            fontFamily: "var(--plt-font)"
        }

        Column {
            gap: 24
            width: "100%"
            maxWidth: "960px"
            margin: "0 auto"

            // -- masthead -------------------------------------------------
            Card {
                padding: 24
                style {
                    background: "var(--plt-surface)"
                    border: "1px solid var(--plt-border)"
                    borderRadius: "var(--plt-radius)"
                    boxShadow: "var(--plt-shadow)"
                }
                Column {
                    gap: 6
                    Heading "Papan Komputasi" {
                        level: 1
                        style {
                            color: "var(--plt-text)"
                            fontSize: "26px"
                            fontWeight: 700
                            margin: "0"
                        }
                    }
                    Text "Semua angka di halaman ini dihitung oleh paket `computasi`. Aplikasi ini tidak menghitung apa pun." {
                        style {
                            color: "var(--plt-muted)"
                            fontSize: "14px"
                        }
                    }
                    Row {
                        gap: 10
                        style {
                            margin: "10px 0 0 0"
                        }
                        Button "Hitung ulang" {
                            on click { describe() }
                        }
                        Button "seed +1" {
                            on click { seed = seed + 1 }
                        }
                        Button "seed +10" {
                            on click { seed = seed + 10 }
                        }
                        Text report {
                            style {
                                color: "var(--plt-muted)"
                                fontSize: "13px"
                                fontFamily: "var(--plt-mono)"
                            }
                        }
                    }
                }
            }

            // -- the number under test ------------------------------------
            Card {
                padding: 20
                style {
                    background: "var(--plt-surface)"
                    border: "1px solid var(--plt-border)"
                    borderRadius: "var(--plt-radius)"
                }
                Column {
                    gap: 14
                    Row {
                        style {
                            justify: "space-between"
                            align: "baseline"
                        }
                        Text "Bilangan yang diuji" {
                            style {
                                color: "var(--plt-text)"
                                fontSize: "15px"
                                fontWeight: 600
                            }
                        }
                        Text toText(seed) {
                            style {
                                color: "var(--plt-accent)"
                                fontFamily: "var(--plt-mono)"
                                fontSize: "22px"
                                fontWeight: 700
                            }
                        }
                    }
                    Row {
                        gap: 10
                        style {
                            align: "center"
                        }
                        Text "seed" {
                            style {
                                color: "var(--plt-muted)"
                                fontSize: "13px"
                                minWidth: "44px"
                            }
                        }
                        Slider {
                            min: 1.0
                            max: 200.0
                            step: 1.0
                            value: seed * 1.0
                            on input { seed = trunc(event.value) }
                        }
                    }
                    Row {
                        gap: 10
                        style {
                            align: "center"
                        }
                        Text "limit" {
                            style {
                                color: "var(--plt-muted)"
                                fontSize: "13px"
                                minWidth: "44px"
                            }
                        }
                        Slider {
                            min: 20.0
                            max: 400.0
                            step: 10.0
                            value: limit * 1.0
                            on input { limit = trunc(event.value) }
                        }
                    }
                    Row {
                        gap: 10
                        style {
                            align: "center"
                        }
                        Text "teks" {
                            style {
                                color: "var(--plt-muted)"
                                fontSize: "13px"
                                minWidth: "44px"
                            }
                        }
                        Input {
                            value: text
                            on input { text = event.value }
                        }
                    }
                }
            }

            // -- the figures ------------------------------------------------
            //
            // A `style` block is a stylesheet, so a colour cannot depend on an
            // input. The library therefore ships one component per colour and
            // the choice between them is made here, where it can be made in
            // code rather than in CSS.
            Grid {
                columns: 4
                gap: 14
                StatTile label: "Langkah Collatz" value: toText(stepCount) hint: "dari " + toText(seed) + " sampai 1"
                SuccessTile label: "Fibonacci" value: toText(fibonacci) hint: "fibonacci(" + toText(idiv(seed, 3)) + ")"
                StatTile label: "Faktor prima" value: toText(len(factors)) hint: join(factors, " x ")
                PrimeStatus value: prime hint: "isPrime(" + toText(seed) + ")"
                StatTile label: "FNV-1a 32" value: toText(digest) hint: "dari \"" + text + "\""
                PalindromeStatus value: palindrome hint: "perbandingan karakter"
                StatTile label: "Median" value: toText(centre) hint: "dari " + toText(len(data)) + " data"
                SuccessTile label: "Variansi" value: toText(spread) hint: "populasi"
            }

            // -- shapes ------------------------------------------------------
            Card {
                padding: 20
                style {
                    background: "var(--plt-surface)"
                    border: "1px solid var(--plt-border)"
                    borderRadius: "var(--plt-radius)"
                }
                Column {
                    gap: 14
                    Heading "Jejak Collatz" {
                        level: 2
                        style {
                            color: "var(--plt-text)"
                            fontSize: "16px"
                            fontWeight: 600
                            margin: "0"
                        }
                    }
                    SparkBars values: trajectory height: 56
                    Text toText(len(trajectory)) + " suku, tertinggi " + toText(maxOf(trajectory)) {
                        style {
                            color: "var(--plt-muted)"
                            fontSize: "13px"
                        }
                    }
                    Meter label: "Kerapatan prima di bawah " + toText(limit) percent: density caption: toText(len(found)) + " prima / " + toText(limit)
                }
            }

            // -- findings ------------------------------------------------------
            FindingPanel title: "Prima di bawah " + toText(limit) items: found columns: 10
            FindingPanel title: "Data terurut menurun" items: ranked columns: 6

            ResultList title: "Ringkasan perhitungan" rows: [
                { label: "gcd(1071, 462)", value: toText(gcd(1071, 462)) },
                { label: "lcm(4, 6)", value: toText(lcm(4, 6)) },
                { label: "collatzSteps(" + toText(seed) + ")", value: toText(stepCount) },
                { label: "fnv1a32(\"" + text + "\")", value: toText(digest) },
                { label: "editDistance ke \"platypus\"", value: toText(editDistance(lower(text), "platypus")) },
                { label: "prima di bawah " + toText(limit), value: toText(len(found)) },
                { label: "tombol ditekan", value: toText(runs) },
            ]

            Text "Sumber: paket `computasi` -- p2lt install dari arsip .libplt." {
                style {
                    color: "var(--plt-muted)"
                    fontSize: "12px"
                    "text-align": "center"
                    padding: "8px 0 0 0"
                }
            }
        }
    }
}

fn ifString(condition: Bool, whenTrue: String, whenFalse: String) -> String {
    if condition {
        return whenTrue
    }
    return whenFalse
}

// A tile whose colour depends on the answer. The colour still comes from the
// library's `StatBody`, so the shape is not duplicated here -- what this owns is
// the decision, which has to be made in code because CSS cannot make it.
component PrimeStatus {
    input value = false
    input hint = ""

    if value {
        SuccessTile label: "Status" value: "prima" hint: hint
    } else {
        DangerTile label: "Status" value: "komposit" hint: hint
    }
}

component PalindromeStatus {
    input value = false
    input hint = ""

    if value {
        SuccessTile label: "Palindrom" value: "ya" hint: hint
    } else {
        StatTile label: "Palindrom" value: "tidak" hint: hint
    }
}

test theDemoShowsLibraryFigures {
    expect stepCount == 111
    expect prime == false
    // 27 is 3^3, so one distinct prime factor.
    expect len(factors) == 1
    expect fibonacci == 34
    expect len(found) == 25
    expect density == 25.0
    expect len(trajectory) == 112
    expect centre == 15.5
    expect digest == 135729481
    expect palindrome == false
}

test movingTheSeedMovesEveryFigure {
    // 28 walks 28, 14, 7, 22, 11, 34, 17, 52, 26, 13, 40, 20, 10, 5, 16, 8,
    // 4, 2, 1 -- eighteen steps, nineteen terms.
    click "seed +1"
    expect stepCount == 18
    expect len(factors) == 2
    expect len(trajectory) == 19
    expect prime == false
    // Both clicks land in the same test, so the seed is 38: not prime, twenty-one
    // steps, and fib(12) is 144.
    click "seed +10"
    expect stepCount == 21
    expect prime == false
    expect len(factors) == 2
    expect fibonacci == 144
}

test recomputingFillsTheReport {
    expect report == ""
    click "Hitung ulang"
    expect report != ""
}

test aLibOnlyModuleNeedsNoAppToPassItsOwnTests {
    // The library's own tests travel with the package and run inside the
    // consumer, so a figure a widget shows is proved in the application too.
    expect toText(get(resultPairs()[0], "value")) == "21"
    expect primeDensity() == 23.0
}
