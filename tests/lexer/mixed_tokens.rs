// Tests that combine multiple literal types

use metorex::lexer::{Lexer, TokenKind};

// ===== Mixed Token Tests =====

#[test]
fn test_lexer_integer_and_string() {
    let mut lexer = Lexer::new(r#"42 "hello""#);

    let token1 = lexer.next_token();
    assert_eq!(token1.kind, TokenKind::Int(42));

    let token2 = lexer.next_token();
    assert_eq!(token2.kind, TokenKind::String("hello".to_string()));
}

#[test]
fn test_lexer_float_and_string() {
    let mut lexer = Lexer::new(r#"3.14 'pi'"#);

    let token1 = lexer.next_token();
    assert_eq!(token1.kind, TokenKind::Float(3.14));

    let token2 = lexer.next_token();
    assert_eq!(token2.kind, TokenKind::String("pi".to_string()));
}

#[test]
fn test_lexer_literals_with_comments() {
    let mut lexer = Lexer::new("42 # a number\n\"text\" # a string");

    let token1 = lexer.next_token();
    assert_eq!(token1.kind, TokenKind::Int(42));

    let token2 = lexer.next_token();
    assert_eq!(token2.kind, TokenKind::Comment("a number".to_string()));

    let token3 = lexer.next_token();
    assert_eq!(token3.kind, TokenKind::Newline);

    let token4 = lexer.next_token();
    assert_eq!(token4.kind, TokenKind::String("text".to_string()));

    let token5 = lexer.next_token();
    assert_eq!(token5.kind, TokenKind::Comment("a string".to_string()));
}

#[test]
fn test_lexer_all_literal_types() {
    let mut lexer = Lexer::new(r#"123 45.67 "string" 'text'"#);

    let token1 = lexer.next_token();
    assert_eq!(token1.kind, TokenKind::Int(123));

    let token2 = lexer.next_token();
    assert_eq!(token2.kind, TokenKind::Float(45.67));

    let token3 = lexer.next_token();
    assert_eq!(token3.kind, TokenKind::String("string".to_string()));

    let token4 = lexer.next_token();
    assert_eq!(token4.kind, TokenKind::String("text".to_string()));
}
