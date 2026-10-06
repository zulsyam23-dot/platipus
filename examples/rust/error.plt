#[rust]
#[export]
fn bad(x: HashMap<String, i64>) -> i64 {
    0
}

app Main {
    Column {
        Text "this file intentionally fails to compile"
    }
}
