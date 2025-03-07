use crate::cursor::Cursor;

#[test]
fn lexer_general_tests() {
    run_tests(&[
        "", "a", "_", "_a", "abc", "\n", " ", "\r\n", "\n\n", "\r\n\r\n", "\u{2028}",
    ]);
}

#[test]
fn lexer_literal_tests() {
    run_tests(&[
        "1", "1_", "12", "0x11", "0b11", "0o11", "011", "0x", "0b", "0o", "0f", // int literals
        "12.1", "12._1", "12.", "12e1", "12e+1", "12e-1", "12e+1_", "12e+_1", "1e", "1e+", "1e-", "1.1e1", "1.e1",
        "0x1e.1e1", // float literals
        "'a'", "'\\n'", "'abc'", "''", "'", "' \n", // char literals
        r#""""#, r#"" ""#, r#""''""#, r#""\"""#, r#"""#, "\"\n\"", // str literals
    ]);
}

#[test]
fn lexer_expr_tests() {
    run_tests(&["a + b", "a+b", "a+ b", "1 + 2", "+-*/%&|^!=<>.,:;", "()[]{}"]);
}

#[test]
fn lexer_block_comment_tests() {
    run_tests(&["/* */", "/** */", "/*! */", "/**/", "/*/**/*/"]);
}

#[test]
fn lexer_block_comment_fails() {
    run_tests(&["/*", "/*/**/", "*/", "/*/"]);
}

#[test]
fn lexer_line_comment_tests() {
    run_tests(&["//", "///", "//!", "// A", "// \n", "////"]);
}

fn run_tests(texts: &[&str]) {
    for (num, text) in texts.into_iter().enumerate() {
        println!(r#"#{num}: "{text}""#);
        let mut cursor = Cursor::new(text);
        std::iter::from_fn(|| cursor.next_token())
            .enumerate()
            .for_each(|(num, token)| println!("  #{num}: {token:?}"));
    }
}
