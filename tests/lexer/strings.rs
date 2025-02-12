// String literal tests

use metorex::lexer::{Lexer, TokenKind};

// ===== String Literal Tests =====

#[test]
fn test_lexer_empty_string_double_quotes() {
    let mut lexer = Lexer::new(r#""""#);
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::String("".to_string()));
}

#[test]
fn test_lexer_empty_string_single_quotes() {
    let mut lexer = Lexer::new("''");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::String("".to_string()));
}

#[test]
fn test_lexer_simple_string_double_quotes() {
    let mut lexer = Lexer::new(r#""hello""#);
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::String("hello".to_string()));
}

#[test]
fn test_lexer_simple_string_single_quotes() {
    let mut lexer = Lexer::new("'world'");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::String("world".to_string()));
}

#[test]
fn test_lexer_string_with_spaces() {
    let mut lexer = Lexer::new(r#""hello world""#);
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::String("hello world".to_string()));
}

#[test]
fn test_lexer_string_with_numbers() {
    let mut lexer = Lexer::new(r#""test 123""#);
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::String("test 123".to_string()));
}

#[test]
fn test_lexer_string_with_special_chars() {
    let mut lexer = Lexer::new(r#""!@#$%^&*()""#);
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::String("!@#$%^&*()".to_string()));
}

// ===== Escape Sequence Tests =====

#[test]
fn test_lexer_string_with_newline_escape() {
    let mut lexer = Lexer::new(r#""hello\nworld""#);
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::String("hello\nworld".to_string()));
}

#[test]
fn test_lexer_string_with_tab_escape() {
    let mut lexer = Lexer::new(r#""hello\tworld""#);
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::String("hello\tworld".to_string()));
}

#[test]
fn test_lexer_string_with_carriage_return_escape() {
    let mut lexer = Lexer::new(r#""hello\rworld""#);
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::String("hello\rworld".to_string()));
}

#[test]
fn test_lexer_string_with_backslash_escape() {
    let mut lexer = Lexer::new(r#""hello\\world""#);
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::String("hello\\world".to_string()));
}

#[test]
fn test_lexer_string_with_quote_escape_double() {
    let mut lexer = Lexer::new(r#""say \"hello\"""#);
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::String(r#"say "hello""#.to_string()));
}

#[test]
fn test_lexer_string_with_quote_escape_single() {
    let mut lexer = Lexer::new(r#"'it\'s fine'"#);
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::String("it's fine".to_string()));
}

#[test]
fn test_lexer_string_with_multiple_escapes() {
    let mut lexer = Lexer::new(r#""line1\nline2\ttab\\""#);
    let token = lexer.next_token();
    assert_eq!(
        token.kind,
        TokenKind::String("line1\nline2\ttab\\".to_string())
    );
}

#[test]
fn test_lexer_string_with_unknown_escape() {
    let mut lexer = Lexer::new(r#""test\xabc""#);
    let token = lexer.next_token();
    // Unknown escapes keep the backslash
    assert_eq!(token.kind, TokenKind::String("test\\xabc".to_string()));
}
