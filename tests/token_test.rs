// Unit tests for token types

use metorex::lexer::{Position, Token, TokenKind};

#[test]
fn test_position_creation() {
    let pos = Position::new(1, 5, 10);
    assert_eq!(pos.line, 1);
    assert_eq!(pos.column, 5);
    assert_eq!(pos.offset, 10);
}

#[test]
fn test_position_equality() {
    let pos1 = Position::new(1, 5, 10);
    let pos2 = Position::new(1, 5, 10);
    let pos3 = Position::new(2, 5, 10);

    assert_eq!(pos1, pos2);
    assert_ne!(pos1, pos3);
}

#[test]
fn test_token_creation() {
    let pos = Position::new(1, 1, 0);
    let token = Token::new(TokenKind::Def, pos);

    assert_eq!(token.kind, TokenKind::Def);
    assert_eq!(token.position.line, 1);
    assert_eq!(token.position.column, 1);
}

#[test]
fn test_keyword_tokens() {
    let keywords = vec![
        (TokenKind::Def, "def"),
        (TokenKind::Class, "class"),
        (TokenKind::If, "if"),
        (TokenKind::Else, "else"),
        (TokenKind::While, "while"),
        (TokenKind::End, "end"),
        (TokenKind::Do, "do"),
    ];

    for (kind, expected) in keywords {
        assert_eq!(kind.to_string(), expected);
    }
}

#[test]
fn test_literal_tokens() {
    assert_eq!(TokenKind::Int(42).to_string(), "42");
    assert_eq!(TokenKind::Int(-10).to_string(), "-10");
    assert_eq!(TokenKind::Float(3.14).to_string(), "3.14");
    assert_eq!(
        TokenKind::String("hello".to_string()).to_string(),
        "\"hello\""
    );
    assert_eq!(TokenKind::True.to_string(), "true");
    assert_eq!(TokenKind::False.to_string(), "false");
    assert_eq!(TokenKind::Nil.to_string(), "nil");
}

#[test]
fn test_identifier_token() {
    let ident = TokenKind::Ident("variable_name".to_string());
    assert_eq!(ident.to_string(), "variable_name");
}

#[test]
fn test_operator_tokens() {
    let operators = vec![
        (TokenKind::Plus, "+"),
        (TokenKind::Minus, "-"),
        (TokenKind::Star, "*"),
        (TokenKind::Slash, "/"),
        (TokenKind::Percent, "%"),
        (TokenKind::Equal, "="),
        (TokenKind::EqualEqual, "=="),
        (TokenKind::BangEqual, "!="),
        (TokenKind::Less, "<"),
        (TokenKind::Greater, ">"),
        (TokenKind::LessEqual, "<="),
        (TokenKind::GreaterEqual, ">="),
    ];

    for (kind, expected) in operators {
        assert_eq!(kind.to_string(), expected);
    }
}

#[test]
fn test_delimiter_tokens() {
    let delimiters = vec![
        (TokenKind::LParen, "("),
        (TokenKind::RParen, ")"),
        (TokenKind::LBrace, "{"),
        (TokenKind::RBrace, "}"),
        (TokenKind::LBracket, "["),
        (TokenKind::RBracket, "]"),
        (TokenKind::Comma, ","),
        (TokenKind::Dot, "."),
        (TokenKind::Colon, ":"),
        (TokenKind::Arrow, "->"),
    ];

    for (kind, expected) in delimiters {
        assert_eq!(kind.to_string(), expected);
    }
}

#[test]
fn test_special_tokens() {
    assert_eq!(TokenKind::Newline.to_string(), "\\n");
    assert_eq!(TokenKind::Semicolon.to_string(), ";");
    assert_eq!(
        TokenKind::Comment("this is a comment".to_string()).to_string(),
        "# this is a comment"
    );
    assert_eq!(TokenKind::EOF.to_string(), "EOF");
}

#[test]
fn test_token_display() {
    let pos = Position::new(5, 10, 42);
    let token = Token::new(TokenKind::Def, pos);

    assert_eq!(token.to_string(), "def at line 5, column 10");
}

#[test]
fn test_token_display_with_literals() {
    let pos = Position::new(1, 1, 0);

    let int_token = Token::new(TokenKind::Int(42), pos);
    assert_eq!(int_token.to_string(), "42 at line 1, column 1");

    let float_token = Token::new(TokenKind::Float(3.14), pos);
    assert_eq!(float_token.to_string(), "3.14 at line 1, column 1");

    let string_token = Token::new(TokenKind::String("hello".to_string()), pos);
    assert_eq!(string_token.to_string(), "\"hello\" at line 1, column 1");
}

#[test]
fn test_token_equality() {
    let pos1 = Position::new(1, 1, 0);
    let pos2 = Position::new(1, 1, 0);

    let token1 = Token::new(TokenKind::Def, pos1);
    let token2 = Token::new(TokenKind::Def, pos2);
    let token3 = Token::new(TokenKind::Class, pos1);

    assert_eq!(token1, token2);
    assert_ne!(token1, token3);
}

#[test]
fn test_token_kind_equality() {
    assert_eq!(TokenKind::Def, TokenKind::Def);
    assert_ne!(TokenKind::Def, TokenKind::Class);

    assert_eq!(TokenKind::Int(42), TokenKind::Int(42));
    assert_ne!(TokenKind::Int(42), TokenKind::Int(43));

    assert_eq!(
        TokenKind::String("hello".to_string()),
        TokenKind::String("hello".to_string())
    );
    assert_ne!(
        TokenKind::String("hello".to_string()),
        TokenKind::String("world".to_string())
    );
}

#[test]
fn test_token_kind_clone() {
    let kind1 = TokenKind::Int(42);
    let kind2 = kind1.clone();

    assert_eq!(kind1, kind2);
}

#[test]
fn test_token_clone() {
    let pos = Position::new(1, 1, 0);
    let token1 = Token::new(TokenKind::Def, pos);
    let token2 = token1.clone();

    assert_eq!(token1, token2);
}

#[test]
fn test_position_clone() {
    let pos1 = Position::new(1, 5, 10);
    let pos2 = pos1;

    assert_eq!(pos1, pos2);
}
