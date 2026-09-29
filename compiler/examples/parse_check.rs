use platipus_compiler::diagnostics::SourceFile;
use platipus_compiler::parser::parse;

fn main() {
    let path = std::env::args().nth(1).unwrap();
    let source = std::fs::read_to_string(&path).unwrap();
    let file = SourceFile::new(&path, source.clone());
    let outcome = parse(&path, source.as_str());
    for error in &outcome.errors {
        println!("{}", error.label());
        if let Some(span) = error.span {
            println!("{}", file.caret_line(span));
        }
        if let Some(help) = &error.help {
            println!("  help: {help}");
        }
        println!();
    }
    println!("errors: {}", outcome.errors.len());
    if let Some(app) = &outcome.program.app {
        println!("app {} items:", app.name);
        for item in &app.body {
            println!("  {item:?}");
        }
    } else {
        println!("no app");
    }
}
