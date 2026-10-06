#[rust]
#[export]
fn count_braces(text: i64) -> i64 {
    let label = r#"}"#;
    let note = "}";
    // }
    /* } */
    text + note.len() as i64
}

app Main {
    Column {
        Text "nested braces in strings and comments"
        Text count_braces(1)
    }
}
