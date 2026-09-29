use platipus_compiler::lexer::{TokenKind, tokenize_lossy};

fn main() {
    let src = std::fs::read_to_string(std::env::args().nth(1).unwrap()).unwrap();
    let (tokens, lex_errors) = tokenize_lossy(&src);
    for e in &lex_errors {
        println!("LEX {e}");
    }
    let mut depth = 0i32;
    for (i, t) in tokens.iter().enumerate() {
        match t.kind {
            TokenKind::LeftBrace => {
                println!(
                    "{i:4} {:>indent$}{} {:?}",
                    "",
                    t.lexeme,
                    t.kind,
                    indent = depth as usize * 2
                );
                depth += 1;
            }
            TokenKind::RightBrace => {
                depth -= 1;
                println!(
                    "{i:4} {:>indent$}{} {:?}",
                    "",
                    t.lexeme,
                    t.kind,
                    indent = depth.max(0) as usize * 2
                );
            }
            _ => println!(
                "{i:4} {:>indent$}{} {:?}",
                "",
                t.lexeme,
                t.kind,
                indent = depth as usize * 2
            ),
        }
    }
}
