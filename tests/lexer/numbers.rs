// Integer and float literal tests

use metorex::lexer::{Lexer, TokenKind};

// ===== Integer Literal Tests =====

#[test]
fn test_lexer_single_digit_integer() {
    let mut lexer = Lexer::new("5");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::Int(5));
    assert_eq!(token.position.line, 1);
    assert_eq!(token.position.column, 1);
}

#[test]
fn test_lexer_multi_digit_integer() {
    let mut lexer = Lexer::new("12345");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::Int(12345));
}

#[test]
fn test_lexer_zero() {
    let mut lexer = Lexer::new("0");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::Int(0));
}

#[test]
fn test_lexer_large_integer() {
    let mut lexer = Lexer::new("9876543210");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::Int(9876543210));
}

#[test]
fn test_lexer_integer_with_whitespace() {
    let mut lexer = Lexer::new("  42  ");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::Int(42));
}

#[test]
fn test_lexer_multiple_integers() {
    let mut lexer = Lexer::new("1 2 3");

    let token1 = lexer.next_token();
    assert_eq!(token1.kind, TokenKind::Int(1));

    let token2 = lexer.next_token();
    assert_eq!(token2.kind, TokenKind::Int(2));

    let token3 = lexer.next_token();
    assert_eq!(token3.kind, TokenKind::Int(3));
}

// ===== Float Literal Tests =====

#[test]
fn test_lexer_simple_float() {
    let mut lexer = Lexer::new("3.14");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::Float(3.14));
}

#[test]
fn test_lexer_float_with_zero_before_decimal() {
    let mut lexer = Lexer::new("0.5");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::Float(0.5));
}

#[test]
fn test_lexer_float_with_multiple_decimal_digits() {
    let mut lexer = Lexer::new("123.456789");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::Float(123.456789));
}

#[test]
fn test_lexer_float_with_trailing_zeros() {
    let mut lexer = Lexer::new("1.00");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::Float(1.0));
}

#[test]
fn test_lexer_float_with_leading_zeros() {
    let mut lexer = Lexer::new("0.001");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::Float(0.001));
}

#[test]
fn test_lexer_very_small_float() {
    let mut lexer = Lexer::new("0.0000001");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::Float(0.0000001));
}

#[test]
fn test_lexer_multiple_floats() {
    let mut lexer = Lexer::new("1.1 2.2 3.3");

    let token1 = lexer.next_token();
    assert_eq!(token1.kind, TokenKind::Float(1.1));

    let token2 = lexer.next_token();
    assert_eq!(token2.kind, TokenKind::Float(2.2));

    let token3 = lexer.next_token();
    assert_eq!(token3.kind, TokenKind::Float(3.3));
}
