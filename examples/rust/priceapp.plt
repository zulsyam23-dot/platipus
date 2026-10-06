#[rust]
#[export]
fn total(prices: i64, count: i64) -> i64 {
    prices * count
}

#[export]
fn discount(total: i64, percent: i64) -> i64 {
    total - (total * percent / 100)
}

app PriceApp {
    Column {
        Text "Price Calculator"
        Text total(120, 3)
        Text discount(360, 10)
    }
}
