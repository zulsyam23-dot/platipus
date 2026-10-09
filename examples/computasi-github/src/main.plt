// An application that installs `computasi` straight from GitHub and calls it.
//
// The package is not vendored here. `p2lt add github.com/zulsyam23-dot/lib-uji-coba-`
// clones the repository, reads `p2lt.toml` from its root, and installs
// `computasi` into the Library Store; `p2lt.lock` beside this file records the
// revision that was resolved. Everything below is therefore a question asked of
// a package fetched over the network, and every figure it shows was computed
// there rather than here.
//
// Nothing in this file does arithmetic. What it owns is the theme and the
// question.

import Komputasi from "computasi"

theme Uji {
    background: "#0d1117"
    surface: "#161b22"
    surfaceAlt: "#21262d"
    border: "#30363d"
    foreground: "#e6edf3"
    muted: "#8b949e"
    accent: "#58a6ff"
    accentText: "#0d1117"
    success: "#3fb950"
    danger: "#f85149"
    radius: "12px"
    gap: "20px"
    pad: "24px"
    shadow: "0 1px 2px rgb(0 0 0 / 40%), 0 12px 32px rgb(0 0 0 / 30%)"
}

app UjiPaket {
    state seed = 27
    state limit = 100
    state text = "platipus"
    state presses = 0
    state note = ""

    // Every figure below is a call into the installed package.
    derived steps = collatzSteps(seed)
    derived trajectory = collatzSequence(seed)
    derived primes = primesUpTo(limit)
    derived density = round(len(primes) * 10000.0 / (1.0 * limit)) / 100.0
    derived factors = primeFactors(seed)
    derived prime = isPrime(seed)
    derived digest = fnv1a32(text)
    derived palindrome = isPalindrome(text)
    derived data = [12, 7, 31, 4, 19, 7]
    derived centre = median(sortAsc(data))
    derived spread = round(variance(data) * 100.0) / 100.0

    fn inspect() {
        presses = presses + 1
        note = toText(editDistance(lower(text), "platypus")) + "|" + toText(wordCount(text)) + " kata|" + toText(factorial(idiv(seed, 2)))
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
            gap: 22
            width: "100%"
            maxWidth: "980px"
            margin: "0 auto"

            Card {
                padding: 22
                style {
                    background: "var(--plt-surface)"
                    border: "1px solid var(--plt-border)"
                    borderRadius: "var(--plt-radius)"
                    boxShadow: "var(--plt-shadow)"
                }
                Column {
                    gap: 6
                    Heading "Paket dari GitHub" {
                        level: 1
                        style {
                            color: "var(--plt-text)"
                            fontSize: "26px"
                            fontWeight: 700
                            margin: "0"
                        }
                    }
                    Text "computasi 0.1.0, diklon dari github.com/zulsyam23-dot/lib-uji-coba-. Aplikasi ini tidak menghitung apa pun." {
                        style {
                            color: "var(--plt-muted)"
                            fontSize: "14px"
                        }
                    }
                    Row {
                        gap: 10
                        style {
                            margin: "12px 0 0 0"
                            align: "center"
                        }
                        Button "Periksa" {
                            on click { inspect() }
                        }
                        Text note {
                            style {
                                color: "var(--plt-muted)"
                                fontSize: "13px"
                                fontFamily: "var(--plt-mono)"
                            }
                        }
                    }
                }
            }

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
                        Text "Masukan" {
                            style {
                                color: "var(--plt-text)"
                                fontSize: "15px"
                                fontWeight: 600
                            }
                        }
                        Text toText(seed) + " / " + toText(limit) {
                            style {
                                color: "var(--plt-accent)"
                                fontFamily: "var(--plt-mono)"
                                fontSize: "18px"
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
                                minWidth: "46px"
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
                                minWidth: "46px"
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
                                minWidth: "46px"
                            }
                        }
                        Input {
                            value: text
                            on input { text = event.value }
                        }
                    }
                    Row {
                        gap: 10
                        Button "seed +1" {
                            on click { seed = seed + 1 }
                        }
                        Button "seed +10" {
                            on click { seed = seed + 10 }
                        }
                    }
                }
            }

            Grid {
                columns: 4
                gap: 14
                StatTile label: "Langkah Collatz" value: toText(steps) hint: toText(seed) + " sampai 1"
                SuccessTile label: "Fibonacci" value: toText(fib(idiv(seed, 3))) hint: "fib(" + toText(idiv(seed, 3)) + ")"
                StatTile label: "Faktor prima" value: toText(len(factors)) hint: join(factors, " x ")
                PrimeStatus value: prime hint: "isPrime(" + toText(seed) + ")"
                StatTile label: "FNV-1a 32" value: toText(digest) hint: text
                PalindromeStatus value: palindrome hint: "karakter demi karakter"
                StatTile label: "Median" value: toText(centre) hint: toText(len(data)) + " data"
                SuccessTile label: "Variansi" value: toText(spread) hint: "populasi"
            }

            Card {
                padding: 20
                style {
                    background: "var(--plt-surface)"
                    border: "1px solid var(--plt-border)"
                    borderRadius: "var(--plt-radius)"
                }
                Column {
                    gap: 14
                    Heading "Jejak dan kerapatan" {
                        level: 2
                        style {
                            color: "var(--plt-text)"
                            fontSize: "16px"
                            fontWeight: 600
                            margin: "0"
                        }
                    }
                    SparkBars values: trajectory height: 54
                    Text toText(len(trajectory)) + " suku, tertinggi " + toText(maxOf(trajectory)) {
                        style {
                            color: "var(--plt-muted)"
                            fontSize: "13px"
                        }
                    }
                    Meter label: "Kerapatan prima di bawah " + toText(limit) percent: density caption: toText(len(primes)) + " prima"
                }
            }

            FindingPanel title: "Prima di bawah " + toText(limit) items: primes columns: 12

            ResultList title: "Ringkasan dari paket" rows: [
                { label: "gcd(1071, 462)", value: toText(gcd(1071, 462)) },
                { label: "lcm(4, 6)", value: toText(lcm(4, 6)) },
                { label: "collatzSteps(" + toText(seed) + ")", value: toText(steps) },
                { label: "fnv1a32(" + text + ")", value: toText(digest) },
                { label: "prima di bawah " + toText(limit), value: toText(len(primes)) },
                { label: "tombol ditekan", value: toText(presses) },
            ]

            Text "Paket: github.com/zulsyam23-dot/lib-uji-coba- -- dipasang dengan `p2lt add`." {
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

// A tile whose colour follows the answer. The choice cannot live in a `style`
// block -- a stylesheet cannot read state -- so it is made here, in code, and
// the shape comes from the package either way.
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

// -- tests -------------------------------------------------------------------
//
// These read the package through the same import the application uses, so a
// number the widgets show is proved here rather than only in the package's own
// test run.

test theInstalledPackageComputes {
    expect steps == 111
    expect len(trajectory) == 112
    expect maxOf(trajectory) == 9232
    expect len(primes) == 25
    expect density == 25.0
    expect len(factors) == 1
    expect prime == false
    expect digest == 135729481
    expect palindrome == false
    expect centre == 9.5
    expect spread == 85.56
}

test thePackageHelpersAreReachable {
    expect toText(get(resultPairs()[0], "value")) == "21"
    expect primeDensity() == 23.0
    expect len(smallPrimes()) == 25
    expect len(collatzBars()) == 112
    expect join(barHeights([1, 2, 4]), ",") == "25,50,100"
}

test movingTheSeedMovesThePackageFigures {
    // 28 walks eighteen steps and has two distinct prime factors.
    click "seed +1"
    expect steps == 18
    expect len(factors) == 2
    expect len(trajectory) == 19
}

test inspectingFillsTheReport {
    expect note == ""
    click "Periksa"
    expect presses == 1
    expect note != ""
}
