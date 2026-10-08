use platipus_compiler::lexer::{
    CONTEXTUAL_ALL, KEYWORDS, OPERATORS, Literal, Token, TokenKind, tokenize, tokenize_lossy,
};

fn lexemes(tokens: &[Token]) -> Vec<&str> {
    tokens.iter().map(|token| token.lexeme.as_str()).collect()
}

fn assert_scans(source: &str) -> Vec<Token> {
    let tokens = tokenize(source).unwrap_or_else(|errors| {
        panic!("expected {source:?} to scan cleanly, got {errors:?}");
    });
    assert_eq!(
        tokens.last().map(|token| token.kind),
        Some(TokenKind::Eof),
        "token stream must be terminated by EOF"
    );
    tokens
}

#[test]
fn scans_every_keyword_as_a_single_token() {
    let source = KEYWORDS
        .iter()
        .map(|(_, text)| *text)
        .collect::<Vec<_>>()
        .join(" ");
    let tokens = assert_scans(&source);
    let body = &tokens[..tokens.len() - 1];
    assert_eq!(body.len(), KEYWORDS.len());
    assert!(body.iter().all(|token| token.kind == TokenKind::Keyword));
    assert_eq!(lexemes(body).join(" "), source);
}

#[test]
fn scans_contextual_keywords_as_ordinary_identifiers() {
    for (_, text) in CONTEXTUAL_ALL {
        let tokens = assert_scans(text);
        assert_eq!(
            tokens[0].kind,
            TokenKind::Identifier,
            "`{text}` is contextual and must stay usable as an identifier"
        );
        assert_eq!(tokens[0].lexeme, *text);
    }
}

#[test]
fn round_trips_every_operator_symbol() {
    for info in OPERATORS {
        let tokens = assert_scans(info.symbol);
        assert_eq!(
            tokens[0].kind, info.kind,
            "wrong kind for {:?}",
            info.symbol
        );
        assert_eq!(tokens[0].lexeme, info.symbol);
    }
}

#[test]
fn prefers_the_longest_operator_match() {
    let tokens = assert_scans("a += b == c != d >= e <= f -> g ? h");
    assert_eq!(
        lexemes(&tokens),
        [
            "a", "+=", "b", "==", "c", "!=", "d", ">=", "e", "<=", "f", "->", "g", "?", "h", ""
        ]
    );
    assert_eq!(tokens[1].kind, TokenKind::PlusAssign);
    assert_eq!(tokens[3].kind, TokenKind::Equal);
    assert_eq!(tokens[5].kind, TokenKind::NotEqual);
    assert_eq!(tokens[11].kind, TokenKind::Arrow);
}

#[test]
fn scans_number_literals() {
    let tokens = assert_scans("0 42 3.14 1_000 1e3 2.5e-2");
    let numbers: Vec<TokenKind> = tokens
        .iter()
        .map(|token| token.kind)
        .filter(|kind| matches!(kind, TokenKind::IntLiteral | TokenKind::FloatLiteral))
        .collect();
    assert_eq!(numbers.len(), 6);
    assert_eq!(
        lexemes(&tokens)[..6],
        ["0", "42", "3.14", "1_000", "1e3", "2.5e-2"]
    );
    assert!(numbers.contains(&TokenKind::IntLiteral));
    assert!(numbers.contains(&TokenKind::FloatLiteral));
}

#[test]
fn keeps_a_trailing_dot_out_of_an_integer() {
    let tokens = assert_scans("count.value 1.");
    assert_eq!(lexemes(&tokens), ["count", ".", "value", "1", ".", ""]);
    assert_eq!(tokens[3].kind, TokenKind::IntLiteral);
    assert_eq!(tokens[4].kind, TokenKind::Dot);
}

#[test]
fn scans_string_literals_with_escapes() {
    let tokens = assert_scans(r#""plain" "with \"quote\"" "line\nbreak" "A""#);
    assert_eq!(tokens.len(), 5);
    assert!(
        tokens[..4]
            .iter()
            .all(|t| t.kind == TokenKind::StringLiteral)
    );
    assert_eq!(tokens[0].lexeme, "plain");
    assert_eq!(tokens[1].lexeme, "with \"quote\"");
    assert_eq!(tokens[2].lexeme, "line\nbreak");
    assert_eq!(tokens[3].lexeme, "A");
}

#[test]
fn treats_most_operators_as_trivia_boundaries_only_where_expected() {
    let tokens = assert_scans("( [ { , : . ] } )");
    assert_eq!(
        kinds(&tokens),
        vec![
            TokenKind::LeftParen,
            TokenKind::LeftBracket,
            TokenKind::LeftBrace,
            TokenKind::Comma,
            TokenKind::Colon,
            TokenKind::Dot,
            TokenKind::RightBracket,
            TokenKind::RightBrace,
            TokenKind::RightParen,
            TokenKind::Eof,
        ]
    );
}

fn kinds(tokens: &[Token]) -> Vec<TokenKind> {
    tokens.iter().map(|token| token.kind).collect()
}

#[test]
fn skips_line_and_block_comments() {
    let tokens = assert_scans("count // trailing\n+ 1 /* block\nspans lines */ - 2");
    assert_eq!(lexemes(&tokens), ["count", "+", "1", "-", "2", ""]);
}

#[test]
fn accepts_unicode_identifiers() {
    let tokens = assert_scans("nilai café عدد");
    assert!(tokens[..3].iter().all(|t| t.kind == TokenKind::Identifier));
    assert_eq!(lexemes(&tokens)[..3], ["nilai", "café", "عدد"]);
}

#[test]
fn tracks_spans_across_lines() {
    let source = "app Main {\n    state count = 0\n}";
    let tokens = assert_scans(source);
    let count = tokens
        .iter()
        .find(|token| token.lexeme == "count")
        .expect("identifier `count`");
    assert_eq!(
        &source[count.span.start as usize..count.span.end as usize],
        "count"
    );
    assert_eq!(count.span.start, source.find("count").unwrap() as u32);
    let brace = tokens
        .iter()
        .find(|token| token.kind == TokenKind::RightBrace)
        .expect("closing brace");
    assert_eq!(brace.span.start as usize, source.len() - 1);
    let eof = tokens.last().expect("EOF");
    assert_eq!(eof.span.start as usize, source.len());
}

#[test]
fn tolerates_a_leading_utf8_bom() {
    let tokens = assert_scans("\u{feff}app Main {}");
    assert_eq!(tokens[0].kind, TokenKind::Keyword);
    assert_eq!(tokens[0].lexeme, "app");
    assert_eq!(tokens[0].span.start, 0);
}

#[test]
fn reports_unterminated_string_and_still_terminates() {
    let (tokens, errors) = tokenize_lossy("Text \"oops\n");
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].code, "unterminated-string");
    assert_eq!(tokens.last().map(|token| token.kind), Some(TokenKind::Eof));
}

#[test]
fn reports_unterminated_block_comment() {
    let (_, errors) = tokenize_lossy("/* never closed");
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].code, "unterminated-comment");
}

#[test]
fn reports_unknown_character_and_keeps_scanning() {
    let (tokens, errors) = tokenize_lossy("count § more");
    assert_eq!(
        errors.len(),
        1,
        "a multi-byte character must produce exactly one diagnostic"
    );
    assert_eq!(errors[0].code, "unexpected-character");
    assert!(
        errors[0].message.contains('§'),
        "diagnostic should name the offending character, got {}",
        errors[0].message
    );
    assert!(
        tokens.iter().any(|token| token.lexeme == "more"),
        "scanner must resynchronize after an unknown character"
    );
}

#[test]
fn rejects_an_invalid_escape_sequence() {
    let (_, errors) = tokenize_lossy(r#""bad \q escape""#);
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].code, "invalid-escape");
}

#[test]
fn handles_empty_and_whitespace_only_input() {
    for source in ["", "   ", "\n\n\t"] {
        let tokens = assert_scans(source);
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].kind, TokenKind::Eof);
    }
}

#[test]
fn scans_a_rust_block_as_one_token() {
    let source = "#[rust]\nfn add(a: i64, b: i64) -> i64 {\n    a + b\n}\n";
    let tokens = assert_scans(source);
    assert_eq!(tokens[0].kind, TokenKind::RustBlock);
    assert!(tokens[0].lexeme.contains("fn add"));
}

#[test]
fn rust_block_braces_inside_strings_do_not_terminate() {
    let source = "#[rust]\nfn f() {\n    println!(\"}\");\n}\n";
    let tokens = assert_scans(source);
    assert_eq!(tokens[0].kind, TokenKind::RustBlock);
    assert!(tokens[0].lexeme.contains("println!"));
}

#[test]
fn rust_block_raw_strings_and_comments_are_safe() {
    let source = "#[rust]\nfn f() {\n    let x = r#\"}\"#;\n    // }\n    /* } */\n}\n";
    let tokens = assert_scans(source);
    assert_eq!(tokens[0].kind, TokenKind::RustBlock);
    assert!(tokens[0].lexeme.contains("r#\"}\"#"));
}

#[test]
fn unterminated_rust_block_is_an_error() {
    let (_, errors) = tokenize_lossy("#[rust]\nfn f( {\n");
    assert!(!errors.is_empty());
}

#[test]
fn unknown_attributes_are_rejected() {
    let (_, errors) = tokenize_lossy("#[foo]\nfn f() {}\n");
    assert!(errors.iter().any(|error| error.code == "unknown-attribute"));
}

fn int_values(tokens: &[Token]) -> Vec<i64> {
    tokens
        .iter()
        .filter(|token| token.kind == TokenKind::IntLiteral)
        .map(|token| match &token.literal {
            Some(Literal::Int(value)) => *value,
            other => panic!("expected an Int literal, got {other:?}"),
        })
        .collect()
}

#[test]
fn scans_hexadecimal_and_binary_literals() {
    let tokens = assert_scans("0xFF 0b1010 0x811c9dc5 1_000_000");
    assert_eq!(int_values(&tokens), [255, 10, 2_166_136_261, 1_000_000]);
    assert_eq!(
        lexemes(&tokens),
        ["0xFF", "0b1010", "0x811c9dc5", "1_000_000", ""]
    );
}

#[test]
fn radix_literal_markers_are_case_insensitive() {
    let tokens = assert_scans("0X1f 0B11");
    assert_eq!(int_values(&tokens), [31, 3]);
}

#[test]
fn reports_malformed_radix_literals() {
    for (source, code) in [
        ("state x = 0b2", "invalid-int"),
        ("state x = 0xG", "invalid-int"),
        ("state x = 0x", "invalid-int"),
        ("state x = 0b", "invalid-int"),
        ("state x = 0xFFFFFFFFFFFFFFFFFF", "invalid-int"),
    ] {
        let (_, errors) = tokenize_lossy(source);
        assert_eq!(errors.len(), 1, "{source} produced {errors:?}");
        assert_eq!(errors[0].code, code, "{source}");
        assert!(
            errors[0].message.contains("literal"),
            "diagnostic should name the literal, got {}",
            errors[0].message
        );
    }
}

#[test]
fn a_decimal_zero_followed_by_an_identifier_is_untouched() {
    let tokens = assert_scans("0 b 0 x 1 + 0 b");
    assert_eq!(
        lexemes(&tokens),
        ["0", "b", "0", "x", "1", "+", "0", "b", ""]
    );
    assert_eq!(int_values(&tokens), [0, 0, 1, 0]);
}

#[test]
fn scans_range_bitwise_and_arrow_symbols() {
    let tokens = assert_scans("0..5 1..=5 x => x n & 1 | 2 ^ 3 ~mask a << b >> c");
    assert_eq!(
        kinds(&tokens),
        [
            TokenKind::IntLiteral,
            TokenKind::DotDot,
            TokenKind::IntLiteral,
            TokenKind::IntLiteral,
            TokenKind::DotDotEqual,
            TokenKind::IntLiteral,
            TokenKind::Identifier,
            TokenKind::FatArrow,
            TokenKind::Identifier,
            TokenKind::Identifier,
            TokenKind::Amp,
            TokenKind::IntLiteral,
            TokenKind::Pipe,
            TokenKind::IntLiteral,
            TokenKind::Caret,
            TokenKind::IntLiteral,
            TokenKind::Tilde,
            TokenKind::Identifier,
            TokenKind::Identifier,
            TokenKind::Shl,
            TokenKind::Identifier,
            TokenKind::Shr,
            TokenKind::Identifier,
            TokenKind::Eof,
        ]
    );
}

#[test]
fn three_character_symbols_win_over_their_two_character_prefixes() {
    let tokens = assert_scans("a >>= b <<=");
    assert_eq!(
        lexemes(&tokens),
        ["a", ">>=", "b", "<<=", ""]
    );
    assert_eq!(tokens[1].kind, TokenKind::ShrAssign);
    assert_eq!(tokens[3].kind, TokenKind::ShlAssign);
}

#[test]
fn bitwise_assignments_are_distinct_from_boolean_operators() {
    let tokens = assert_scans("& && | || ^ ~ != = > >= >> >>=");
    assert_eq!(
        kinds(&tokens),
        [
            TokenKind::Amp,
            TokenKind::AndAnd,
            TokenKind::Pipe,
            TokenKind::OrOr,
            TokenKind::Caret,
            TokenKind::Tilde,
            TokenKind::NotEqual,
            TokenKind::Assign,
            TokenKind::Greater,
            TokenKind::GreaterEqual,
            TokenKind::Shr,
            TokenKind::ShrAssign,
            TokenKind::Eof,
        ]
    );
}

#[test]
fn member_access_stays_a_single_dot() {
    let tokens = assert_scans("xs.length 1.5..2.5");
    assert_eq!(
        lexemes(&tokens),
        ["xs", ".", "length", "1.5", "..", "2.5", ""]
    );
    assert_eq!(tokens[4].kind, TokenKind::DotDot);
}

#[test]
fn let_and_while_scan_as_keywords() {
    for text in ["let", "while"] {
        let tokens = assert_scans(&format!("{text} value"));
        assert_eq!(
            tokens[0].kind,
            TokenKind::Keyword,
            "`{text}` must be a keyword"
        );
        assert_eq!(tokens[0].lexeme, text);
        assert_eq!(tokens[1].kind, TokenKind::Identifier);
    }
}
