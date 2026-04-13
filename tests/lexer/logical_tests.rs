// Logical operator tests

use metorex::lexer::{Lexer, TokenKind};

// ===== Logical Operator Tests =====

#[test]
fn test_lexer_operator_logical_and() {
    let mut lexer = Lexer::new("&&");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::LogicalAnd);
}

#[test]
fn test_lexer_operator_logical_or() {
    let mut lexer = Lexer::new("||");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::LogicalOr);
}

#[test]
fn test_lexer_operator_colon_colon() {
    let mut lexer = Lexer::new("::");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::ColonColon);
}

#[test]
fn test_lexer_single_ampersand_not_logical_and() {
    let mut lexer = Lexer::new("&");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::Ampersand);
}

#[test]
fn test_lexer_single_pipe_not_logical_or() {
    let mut lexer = Lexer::new("|");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::Pipe);
}

#[test]
fn test_lexer_single_colon_not_colon_colon() {
    let mut lexer = Lexer::new(":");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::Colon);
}

#[test]
fn test_lexer_logical_and_expression() {
    let mut lexer = Lexer::new("x && y");

    let token1 = lexer.next_token();
    assert_eq!(token1.kind, TokenKind::Ident("x".to_string()));

    let token2 = lexer.next_token();
    assert_eq!(token2.kind, TokenKind::LogicalAnd);

    let token3 = lexer.next_token();
    assert_eq!(token3.kind, TokenKind::Ident("y".to_string()));
}

#[test]
fn test_lexer_logical_or_expression() {
    let mut lexer = Lexer::new("x || y");

    let token1 = lexer.next_token();
    assert_eq!(token1.kind, TokenKind::Ident("x".to_string()));

    let token2 = lexer.next_token();
    assert_eq!(token2.kind, TokenKind::LogicalOr);

    let token3 = lexer.next_token();
    assert_eq!(token3.kind, TokenKind::Ident("y".to_string()));
}

#[test]
fn test_lexer_scope_resolution_expression() {
    let mut lexer = Lexer::new("Math::PI");

    let token1 = lexer.next_token();
    assert_eq!(token1.kind, TokenKind::Ident("Math".to_string()));

    let token2 = lexer.next_token();
    assert_eq!(token2.kind, TokenKind::ColonColon);

    let token3 = lexer.next_token();
    assert_eq!(token3.kind, TokenKind::Ident("PI".to_string()));
}

// ===== Mixed Operator and Delimiter Tests =====

#[test]
fn test_lexer_arithmetic_expression() {
    let mut lexer = Lexer::new("1 + 2 * 3");

    let token1 = lexer.next_token();
    assert_eq!(token1.kind, TokenKind::Int(1));

    let token2 = lexer.next_token();
    assert_eq!(token2.kind, TokenKind::Plus);

    let token3 = lexer.next_token();
    assert_eq!(token3.kind, TokenKind::Int(2));

    let token4 = lexer.next_token();
    assert_eq!(token4.kind, TokenKind::Star);

    let token5 = lexer.next_token();
    assert_eq!(token5.kind, TokenKind::Int(3));
}

#[test]
fn test_lexer_comparison_expression() {
    let mut lexer = Lexer::new("x == y");

    let token1 = lexer.next_token();
    assert_eq!(token1.kind, TokenKind::Ident("x".to_string()));

    let token2 = lexer.next_token();
    assert_eq!(token2.kind, TokenKind::EqualEqual);

    let token3 = lexer.next_token();
    assert_eq!(token3.kind, TokenKind::Ident("y".to_string()));
}

#[test]
fn test_lexer_assignment_expression() {
    let mut lexer = Lexer::new("x = 42");

    let token1 = lexer.next_token();
    assert_eq!(token1.kind, TokenKind::Ident("x".to_string()));

    let token2 = lexer.next_token();
    assert_eq!(token2.kind, TokenKind::Equal);

    let token3 = lexer.next_token();
    assert_eq!(token3.kind, TokenKind::Int(42));
}

#[test]
fn test_lexer_compound_assignment() {
    let mut lexer = Lexer::new("x += 5");

    let token1 = lexer.next_token();
    assert_eq!(token1.kind, TokenKind::Ident("x".to_string()));

    let token2 = lexer.next_token();
    assert_eq!(token2.kind, TokenKind::PlusEqual);

    let token3 = lexer.next_token();
    assert_eq!(token3.kind, TokenKind::Int(5));
}

#[test]
fn test_lexer_function_call_syntax() {
    let mut lexer = Lexer::new("foo(x, y)");

    let token1 = lexer.next_token();
    assert_eq!(token1.kind, TokenKind::Ident("foo".to_string()));

    let token2 = lexer.next_token();
    assert_eq!(token2.kind, TokenKind::LParen);

    let token3 = lexer.next_token();
    assert_eq!(token3.kind, TokenKind::Ident("x".to_string()));

    let token4 = lexer.next_token();
    assert_eq!(token4.kind, TokenKind::Comma);

    let token5 = lexer.next_token();
    assert_eq!(token5.kind, TokenKind::Ident("y".to_string()));

    let token6 = lexer.next_token();
    assert_eq!(token6.kind, TokenKind::RParen);
}

#[test]
fn test_lexer_array_syntax() {
    let mut lexer = Lexer::new("[1, 2, 3]");

    let token1 = lexer.next_token();
    assert_eq!(token1.kind, TokenKind::LBracket);

    let token2 = lexer.next_token();
    assert_eq!(token2.kind, TokenKind::Int(1));

    let token3 = lexer.next_token();
    assert_eq!(token3.kind, TokenKind::Comma);

    let token4 = lexer.next_token();
    assert_eq!(token4.kind, TokenKind::Int(2));

    let token5 = lexer.next_token();
    assert_eq!(token5.kind, TokenKind::Comma);

    let token6 = lexer.next_token();
    assert_eq!(token6.kind, TokenKind::Int(3));

    let token7 = lexer.next_token();
    assert_eq!(token7.kind, TokenKind::RBracket);
}

#[test]
fn test_lexer_method_chaining() {
    let mut lexer = Lexer::new("obj.method");

    let token1 = lexer.next_token();
    assert_eq!(token1.kind, TokenKind::Ident("obj".to_string()));

    let token2 = lexer.next_token();
    assert_eq!(token2.kind, TokenKind::Dot);

    let token3 = lexer.next_token();
    assert_eq!(token3.kind, TokenKind::Ident("method".to_string()));
}

#[test]
fn test_lexer_arrow_function_syntax() {
    let mut lexer = Lexer::new("x -> x + 1");

    let token1 = lexer.next_token();
    assert_eq!(token1.kind, TokenKind::Ident("x".to_string()));

    let token2 = lexer.next_token();
    assert_eq!(token2.kind, TokenKind::Arrow);

    let token3 = lexer.next_token();
    assert_eq!(token3.kind, TokenKind::Ident("x".to_string()));

    let token4 = lexer.next_token();
    assert_eq!(token4.kind, TokenKind::Plus);

    let token5 = lexer.next_token();
    assert_eq!(token5.kind, TokenKind::Int(1));
}

#[test]
fn test_lexer_operators_without_spaces() {
    let mut lexer = Lexer::new("x==y");

    let token1 = lexer.next_token();
    assert_eq!(token1.kind, TokenKind::Ident("x".to_string()));

    let token2 = lexer.next_token();
    assert_eq!(token2.kind, TokenKind::EqualEqual);

    let token3 = lexer.next_token();
    assert_eq!(token3.kind, TokenKind::Ident("y".to_string()));
}

#[test]
fn test_lexer_complex_expression() {
    let mut lexer = Lexer::new("(x + y) * (a - b) / z");

    let token1 = lexer.next_token();
    assert_eq!(token1.kind, TokenKind::LParen);

    let token2 = lexer.next_token();
    assert_eq!(token2.kind, TokenKind::Ident("x".to_string()));

    let token3 = lexer.next_token();
    assert_eq!(token3.kind, TokenKind::Plus);

    let token4 = lexer.next_token();
    assert_eq!(token4.kind, TokenKind::Ident("y".to_string()));

    let token5 = lexer.next_token();
    assert_eq!(token5.kind, TokenKind::RParen);

    let token6 = lexer.next_token();
    assert_eq!(token6.kind, TokenKind::Star);

    let token7 = lexer.next_token();
    assert_eq!(token7.kind, TokenKind::LParen);

    let token8 = lexer.next_token();
    assert_eq!(token8.kind, TokenKind::Ident("a".to_string()));

    let token9 = lexer.next_token();
    assert_eq!(token9.kind, TokenKind::Minus);

    let token10 = lexer.next_token();
    assert_eq!(token10.kind, TokenKind::Ident("b".to_string()));

    let token11 = lexer.next_token();
    assert_eq!(token11.kind, TokenKind::RParen);

    let token12 = lexer.next_token();
    assert_eq!(token12.kind, TokenKind::Slash);

    let token13 = lexer.next_token();
    assert_eq!(token13.kind, TokenKind::Ident("z".to_string()));
}
