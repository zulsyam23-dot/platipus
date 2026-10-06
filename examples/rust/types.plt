#[rust]
#[export]
fn square(x: i64) -> i64 {
    x * x
}

#[export]
fn half(value: i64) -> i64 {
    value / 2
}

#[export]
fn average(a: f64, b: f64) -> f64 {
    (a + b) / 2.0
}

app Main {
    Column {
        Text "types"
        Text square(9)
        Text half(9)
        Text average(3.0, 7.0)
    }
}
