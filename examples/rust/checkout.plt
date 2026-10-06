#[rust]
#[export]
fn subtotal(price: i64, qty: i64) -> i64 {
    price * qty
}

#[export]
fn tax(amount: i64, rate: i64) -> i64 {
    amount * rate / 100
}

#[export]
fn grand_total(price: i64, qty: i64, rate: i64) -> i64 {
    let s = subtotal(price, qty);
    s + tax(s, rate)
}

component Money {
    input label = ""
    input value = 0

    Row {
        justify: "space-between"
        Text label
        Text value
    }
}

app CheckoutApp {
    state price = 150
    state qty = 3
    state rate = 11

    Column {
        gap: 12
        Heading "Checkout" {
            level: 1
            size: 28
        }

        Card {
            padding: 16
            Row {
                gap: 16
                Column {
                    gap: 6
                    Text "Item price"
                    Text price
                }
                Column {
                    gap: 6
                    Text "Quantity"
                    Row {
                        gap: 8
                        Button "-" {
                            on click {
                                if qty > 1 {
                                    qty = qty - 1
                                }
                            }
                        }
                        Text qty
                        Button "+" {
                            on click {
                                qty = qty + 1
                            }
                        }
                    }
                }
                Column {
                    gap: 6
                    Text "Tax rate"
                    Text rate
                }
            }
        }

        Card {
            padding: 16
            Column {
                gap: 6
                Money label: "Subtotal" value: subtotal(price, qty)
                Money label: "Tax" value: tax(subtotal(price, qty), rate)
                Money label: "Grand Total" value: grand_total(price, qty, rate)
            }
        }
    }
}
