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
    // After an identifier, / is division not regex
    let tokens = Lexer::new("a / b").tokenize();
    assert_eq!(tokens[1].kind, TokenKind::Slash);
}

#[test]
fn test_lexer_regex_literal() {
    // At start of input, / begins a regex literal
    let tokens = Lexer::new("/pattern/").tokenize();
    assert_eq!(
        tokens[0].kind,
        TokenKind::Regex("pattern".to_string(), "".to_string())
    );
}

#[test]
fn test_lexer_regex_literal_with_flags() {
    let tokens = Lexer::new("/hello/im").tokenize();
    assert_eq!(
        tokens[0].kind,
        TokenKind::Regex("hello".to_string(), "im".to_string())
    );
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
    // After an identifier, /= is division-assignment
    let tokens = Lexer::new("a /= b").tokenize();
    assert_eq!(tokens[1].kind, TokenKind::SlashEqual);
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

// ===== Bang Operator Tests =====

#[test]
fn test_lexer_operator_bang() {
    let mut lexer = Lexer::new("!");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::Bang);
}

#[test]
fn test_lexer_operator_bang_not_bang_equal() {
    let mut lexer = Lexer::new("!");
    let token = lexer.next_token();
    assert_ne!(token.kind, TokenKind::BangEqual);
}

#[test]
fn test_lexer_bang_expression() {
    let mut lexer = Lexer::new("!x");

    let token1 = lexer.next_token();
    assert_eq!(token1.kind, TokenKind::Bang);

    let token2 = lexer.next_token();
    assert_eq!(token2.kind, TokenKind::Ident("x".to_string()));
}
