// Unit tests for token types

use metorex::lexer::{InterpolationPart, Position, Token, TokenKind};

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
fn test_remaining_keyword_tokens_display() {
    let keywords = vec![
        (TokenKind::Elsif, "elsif"),
        (TokenKind::Unless, "unless"),
        (TokenKind::For, "for"),
        (TokenKind::In, "in"),
        (TokenKind::Begin, "begin"),
        (TokenKind::Rescue, "rescue"),
        (TokenKind::Ensure, "ensure"),
        (TokenKind::Raise, "raise"),
        (TokenKind::Break, "break"),
        (TokenKind::Continue, "continue"),
        (TokenKind::Return, "return"),
        (TokenKind::Lambda, "lambda"),
        (TokenKind::Super, "super"),
        (TokenKind::Case, "case"),
        (TokenKind::When, "when"),
        (TokenKind::Then, "then"),
        (TokenKind::AttrReader, "attr_reader"),
        (TokenKind::AttrWriter, "attr_writer"),
        (TokenKind::AttrAccessor, "attr_accessor"),
        (TokenKind::Module, "module"),
        (TokenKind::Include, "include"),
        (TokenKind::Extend, "extend"),
    ];

    for (kind, expected) in keywords {
        assert_eq!(kind.to_string(), expected);
    }
}

#[test]
fn test_interpolated_string_token_display() {
    let parts = vec![
        InterpolationPart::Text("hello ".to_string()),
        InterpolationPart::Expression("name".to_string()),
    ];
    let token = TokenKind::InterpolatedString(parts);
    let display = token.to_string();
    assert!(display.contains("hello"));
}

#[test]
fn test_variable_tokens_display() {
    assert_eq!(
        TokenKind::InstanceVar("foo".to_string()).to_string(),
        "@foo"
    );
    assert_eq!(
        TokenKind::ClassVar("count".to_string()).to_string(),
        "@@count"
    );
    assert_eq!(TokenKind::GlobalVar("var".to_string()).to_string(), "$var");
}

#[test]
fn test_compound_assignment_tokens_display() {
    let tokens = vec![
        (TokenKind::PlusEqual, "+="),
        (TokenKind::MinusEqual, "-="),
        (TokenKind::StarEqual, "*="),
        (TokenKind::SlashEqual, "/="),
    ];
    for (kind, expected) in tokens {
        assert_eq!(kind.to_string(), expected);
    }
}

#[test]
fn test_additional_operator_tokens_display() {
    let tokens = vec![
        (TokenKind::Bang, "!"),
        (TokenKind::DotDot, ".."),
        (TokenKind::DotDotDot, "..."),
        (TokenKind::FatArrow, "=>"),
        (TokenKind::Pipe, "|"),
        (TokenKind::Ampersand, "&"),
        (TokenKind::LogicalAnd, "&&"),
        (TokenKind::LogicalOr, "||"),
        (TokenKind::ColonColon, "::"),
    ];
    for (kind, expected) in tokens {
        assert_eq!(kind.to_string(), expected);
    }
}

#[test]
fn test_keyword_yield_defined_display() {
    assert_eq!(TokenKind::Yield.to_string(), "yield");
    assert_eq!(TokenKind::Defined.to_string(), "defined?");
}

#[test]
fn test_magic_constants_display() {
    assert_eq!(TokenKind::MagicFile.to_string(), "__FILE__");
    assert_eq!(TokenKind::MagicLine.to_string(), "__LINE__");
}

#[test]
fn test_percent_w_display() {
    assert_eq!(
        TokenKind::PercentW("a b c".to_string()).to_string(),
        "%w[a b c]"
    );
}

#[test]
fn test_regex_token_display() {
    assert_eq!(
        TokenKind::Regex("foo".to_string(), "i".to_string()).to_string(),
        "/foo/i"
    );
}

#[test]
fn test_string_literal_display() {
    assert_eq!(TokenKind::String("hi".to_string()).to_string(), "\"hi\"");
}

#[test]
fn test_star_star_display() {
    assert_eq!(TokenKind::StarStar.to_string(), "**");
}

#[test]
fn test_less_equal_display() {
    assert_eq!(TokenKind::LessEqual.to_string(), "<=");
}

#[test]
fn test_spaceship_display() {
    assert_eq!(TokenKind::Spaceship.to_string(), "<=>");
}

#[test]
fn test_shovel_display() {
    assert_eq!(TokenKind::Shovel.to_string(), "<<");
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
