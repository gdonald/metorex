// Operator and delimiter tests

use metorex::lexer::{Lexer, TokenKind};

// ===== Single-Character Operator Tests =====

#[test]
fn test_lexer_operator_plus() {
    let mut lexer = Lexer::new("+");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::Plus);
}

#[test]
fn test_lexer_operator_minus() {
    let mut lexer = Lexer::new("-");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::Minus);
}

#[test]
fn test_lexer_operator_star() {
    let mut lexer = Lexer::new("*");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::Star);
}

#[test]
fn test_lexer_operator_slash() {
    let mut lexer = Lexer::new("/");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::Slash);
}

#[test]
fn test_lexer_operator_percent() {
    let mut lexer = Lexer::new("%");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::Percent);
}

#[test]
fn test_lexer_operator_equal() {
    let mut lexer = Lexer::new("=");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::Equal);
}

#[test]
fn test_lexer_operator_less() {
    let mut lexer = Lexer::new("<");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::Less);
}

#[test]
fn test_lexer_operator_greater() {
    let mut lexer = Lexer::new(">");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::Greater);
}

// ===== Multi-Character Operator Tests =====

#[test]
fn test_lexer_operator_equal_equal() {
    let mut lexer = Lexer::new("==");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::EqualEqual);
}

#[test]
fn test_lexer_operator_bang_equal() {
    let mut lexer = Lexer::new("!=");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::BangEqual);
}

#[test]
fn test_lexer_operator_less_equal() {
    let mut lexer = Lexer::new("<=");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::LessEqual);
}

#[test]
fn test_lexer_operator_greater_equal() {
    let mut lexer = Lexer::new(">=");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::GreaterEqual);
}

#[test]
fn test_lexer_operator_arrow() {
    let mut lexer = Lexer::new("->");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::Arrow);
}

// ===== Compound Assignment Operator Tests =====

#[test]
fn test_lexer_operator_plus_equal() {
    let mut lexer = Lexer::new("+=");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::PlusEqual);
}

#[test]
fn test_lexer_operator_minus_equal() {
    let mut lexer = Lexer::new("-=");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::MinusEqual);
}

#[test]
fn test_lexer_operator_star_equal() {
    let mut lexer = Lexer::new("*=");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::StarEqual);
}

#[test]
fn test_lexer_operator_slash_equal() {
    let mut lexer = Lexer::new("/=");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::SlashEqual);
}

// ===== Delimiter Tests =====

#[test]
fn test_lexer_delimiter_lparen() {
    let mut lexer = Lexer::new("(");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::LParen);
}

#[test]
fn test_lexer_delimiter_rparen() {
    let mut lexer = Lexer::new(")");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::RParen);
}

#[test]
fn test_lexer_delimiter_lbrace() {
    let mut lexer = Lexer::new("{");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::LBrace);
}

#[test]
fn test_lexer_delimiter_rbrace() {
    let mut lexer = Lexer::new("}");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::RBrace);
}

#[test]
fn test_lexer_delimiter_lbracket() {
    let mut lexer = Lexer::new("[");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::LBracket);
}

#[test]
fn test_lexer_delimiter_rbracket() {
    let mut lexer = Lexer::new("]");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::RBracket);
}

#[test]
fn test_lexer_delimiter_comma() {
    let mut lexer = Lexer::new(",");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::Comma);
}

#[test]
fn test_lexer_delimiter_dot() {
    let mut lexer = Lexer::new(".");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::Dot);
}

#[test]
fn test_lexer_delimiter_colon() {
    let mut lexer = Lexer::new(":");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::Colon);
}

#[test]
fn test_lexer_delimiter_semicolon() {
    let mut lexer = Lexer::new(";");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::Semicolon);
}

// ===== Balanced Delimiter Tests =====

#[test]
fn test_lexer_balanced_parens() {
    let mut lexer = Lexer::new("()");

    let token1 = lexer.next_token();
    assert_eq!(token1.kind, TokenKind::LParen);

    let token2 = lexer.next_token();
    assert_eq!(token2.kind, TokenKind::RParen);
}

#[test]
fn test_lexer_balanced_braces() {
    let mut lexer = Lexer::new("{}");

    let token1 = lexer.next_token();
    assert_eq!(token1.kind, TokenKind::LBrace);

    let token2 = lexer.next_token();
    assert_eq!(token2.kind, TokenKind::RBrace);
}

#[test]
fn test_lexer_balanced_brackets() {
    let mut lexer = Lexer::new("[]");

    let token1 = lexer.next_token();
    assert_eq!(token1.kind, TokenKind::LBracket);

    let token2 = lexer.next_token();
    assert_eq!(token2.kind, TokenKind::RBracket);
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
fn test_lexer_all_comparison_operators() {
    let mut lexer = Lexer::new("< > <= >= == !=");

    let token1 = lexer.next_token();
    assert_eq!(token1.kind, TokenKind::Less);

    let token2 = lexer.next_token();
    assert_eq!(token2.kind, TokenKind::Greater);

    let token3 = lexer.next_token();
    assert_eq!(token3.kind, TokenKind::LessEqual);

    let token4 = lexer.next_token();
    assert_eq!(token4.kind, TokenKind::GreaterEqual);

    let token5 = lexer.next_token();
    assert_eq!(token5.kind, TokenKind::EqualEqual);

    let token6 = lexer.next_token();
    assert_eq!(token6.kind, TokenKind::BangEqual);
}

#[test]
fn test_lexer_all_arithmetic_operators() {
    let mut lexer = Lexer::new("+ - * / %");

    let token1 = lexer.next_token();
    assert_eq!(token1.kind, TokenKind::Plus);

    let token2 = lexer.next_token();
    assert_eq!(token2.kind, TokenKind::Minus);

    let token3 = lexer.next_token();
    assert_eq!(token3.kind, TokenKind::Star);

    let token4 = lexer.next_token();
    assert_eq!(token4.kind, TokenKind::Slash);

    let token5 = lexer.next_token();
    assert_eq!(token5.kind, TokenKind::Percent);
}

#[test]
fn test_lexer_all_compound_assignments() {
    let mut lexer = Lexer::new("+= -= *= /=");

    let token1 = lexer.next_token();
    assert_eq!(token1.kind, TokenKind::PlusEqual);

    let token2 = lexer.next_token();
    assert_eq!(token2.kind, TokenKind::MinusEqual);

    let token3 = lexer.next_token();
    assert_eq!(token3.kind, TokenKind::StarEqual);

    let token4 = lexer.next_token();
    assert_eq!(token4.kind, TokenKind::SlashEqual);
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
