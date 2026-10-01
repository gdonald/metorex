// Reading a source's bytes as text, when some of them are not UTF-8.

use metorex::file_loader::{escaped_source_bytes, escaped_source_text, source_text};

#[test]
fn utf8_source_reads_as_written() {
    assert_eq!(source_text("p 'é'".as_bytes()), Some("p 'é'".to_string()));
}

#[test]
fn source_that_is_not_utf8_without_a_magic_comment_is_refused() {
    assert_eq!(source_text(b"p '\xa7A'"), None);
}

#[test]
fn source_naming_utf8_that_is_not_utf8_is_refused() {
    assert_eq!(source_text(b"# encoding: utf-8\np '\xa7A'"), None);
}

#[test]
fn source_in_a_latin_encoding_reads_through_its_table() {
    assert_eq!(
        source_text(b"# encoding: iso-8859-1\np '\xe9'"),
        Some("# encoding: iso-8859-1\np 'é'".to_string())
    );
}

#[test]
fn source_in_an_encoding_with_no_table_escapes_its_other_bytes() {
    let text = source_text(b"# encoding: big5\np '\xa7A'").expect("the source reads");
    assert_eq!(
        escaped_source_bytes(&text),
        Some(b"# encoding: big5\np '\xa7A'".to_vec())
    );
}

#[test]
fn escaped_text_keeps_utf8_runs_and_escapes_the_rest() {
    let text = escaped_source_text(
        "é".as_bytes()
            .iter()
            .chain(b"\xff")
            .copied()
            .collect::<Vec<_>>()
            .as_slice(),
    );
    assert!(text.starts_with('é'));
    assert_eq!(escaped_source_bytes(&text), Some(vec![0xC3, 0xA9, 0xFF]));
}

#[test]
fn text_with_no_escaped_characters_has_no_escaped_bytes() {
    assert_eq!(escaped_source_bytes("plain é"), None);
}

#[test]
fn eval_of_code_in_an_encoding_with_no_table_keeps_the_literals_bytes() {
    use metorex::lexer::Lexer;
    use metorex::parser::Parser;
    use metorex::vm::VirtualMachine;
    let code = "code = \"# encoding: big5\\n'\\xA7A'.bytes\".force_encoding('UTF-8')\neval(code)";
    let statements = Parser::new(Lexer::new(code).tokenize())
        .parse()
        .expect("parse failed");
    let answer = VirtualMachine::new()
        .execute_program(&statements)
        .expect("execution failed");
    assert_eq!(
        answer.map(|held| held.to_string()),
        Some("[167, 65]".to_string())
    );
}
