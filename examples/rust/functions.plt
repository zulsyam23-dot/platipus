#[rust]
#[export]
fn fibonacci(n: i64) -> i64 {
    if n <= 1 {
        n
    } else {
        fibonacci(n - 1) + fibonacci(n - 2)
    }
}

#[export]
fn double(value: i64) -> i64 {
    value * 2
}

app Main {
    Column {
        Text "fibonacci(10)"
        Text fibonacci(10)
        Text double(21)
    }
}
