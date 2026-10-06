#[rust]
#[export]
fn sum(n: i64) -> i64 {
    let mut total = 0;
    let mut i = 0;
    while i <= n {
        if i % 2 == 0 {
            total += i;
        }
        i += 1;
    }
    total
}

app Main {
    Column {
        Text "sum of even numbers"
        Text sum(10)
    }
}
