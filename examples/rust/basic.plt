#[rust]
#[export]
fn add(a: i64, b: i64) -> i64 {
    a + b
}

app Main {
    Column {
        Text "Rust integration"
        Text add(10, 20)
    }
}
