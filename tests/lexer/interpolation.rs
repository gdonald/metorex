// String interpolation tests

use metorex::lexer::{InterpolationPart, Lexer, TokenKind};

// ===== String Interpolation Tests =====

#[test]
fn test_lexer_interpolated_string_simple() {
    let mut lexer = Lexer::new(r#""hello #{name}""#);
    let token = lexer.next_token();

    match token.kind {
        TokenKind::InterpolatedString(parts) => {
            assert_eq!(parts.len(), 2);
            assert_eq!(parts[0], InterpolationPart::Text("hello ".to_string()));
            assert_eq!(parts[1], InterpolationPart::Expression("name".to_string()));
        }
        _ => panic!("Expected InterpolatedString, got {:?}", token.kind),
    }
}

#[test]
fn test_lexer_interpolated_string_multiple() {
    let mut lexer = Lexer::new(r##""#{x} + #{y} = #{z}""##);
    let token = lexer.next_token();

    match token.kind {
        TokenKind::InterpolatedString(parts) => {
            assert_eq!(parts.len(), 5);
            assert_eq!(parts[0], InterpolationPart::Expression("x".to_string()));
            assert_eq!(parts[1], InterpolationPart::Text(" + ".to_string()));
            assert_eq!(parts[2], InterpolationPart::Expression("y".to_string()));
            assert_eq!(parts[3], InterpolationPart::Text(" = ".to_string()));
            assert_eq!(parts[4], InterpolationPart::Expression("z".to_string()));
        }
        _ => panic!("Expected InterpolatedString, got {:?}", token.kind),
    }
}

#[test]
fn test_lexer_interpolated_string_at_start() {
    let mut lexer = Lexer::new(r##""#{greeting}, world!""##);
    let token = lexer.next_token();

    match token.kind {
        TokenKind::InterpolatedString(parts) => {
            assert_eq!(parts.len(), 2);
            assert_eq!(
                parts[0],
                InterpolationPart::Expression("greeting".to_string())
            );
            assert_eq!(parts[1], InterpolationPart::Text(", world!".to_string()));
        }
        _ => panic!("Expected InterpolatedString, got {:?}", token.kind),
    }
}

#[test]
fn test_lexer_interpolated_string_at_end() {
    let mut lexer = Lexer::new(r##""result: #{value}""##);
    let token = lexer.next_token();

    match token.kind {
        TokenKind::InterpolatedString(parts) => {
            assert_eq!(parts.len(), 2);
            assert_eq!(parts[0], InterpolationPart::Text("result: ".to_string()));
            assert_eq!(parts[1], InterpolationPart::Expression("value".to_string()));
        }
        _ => panic!("Expected InterpolatedString, got {:?}", token.kind),
    }
}

#[test]
fn test_lexer_interpolated_string_only_expression() {
    let mut lexer = Lexer::new(r##""#{value}""##);
    let token = lexer.next_token();

    match token.kind {
        TokenKind::InterpolatedString(parts) => {
            assert_eq!(parts.len(), 1);
            assert_eq!(parts[0], InterpolationPart::Expression("value".to_string()));
        }
        _ => panic!("Expected InterpolatedString, got {:?}", token.kind),
    }
}

#[test]
fn test_lexer_string_no_interpolation_single_quotes() {
    let mut lexer = Lexer::new(r##"'#{name}'"##);
    let token = lexer.next_token();
    // Single quotes don't support interpolation
    assert_eq!(token.kind, TokenKind::String("#{name}".to_string()));
}

#[test]
fn test_lexer_string_escaped_hash() {
    let mut lexer = Lexer::new(r##""\#{not_interpolated}""##);
    let token = lexer.next_token();
    // Escaped hash should not trigger interpolation
    assert_eq!(
        token.kind,
        TokenKind::String("#{not_interpolated}".to_string())
    );
}

#[test]
fn test_lexer_interpolated_string_with_complex_expression() {
    let mut lexer = Lexer::new(r##""result: #{x + y * 2}""##);
    let token = lexer.next_token();

    match token.kind {
        TokenKind::InterpolatedString(parts) => {
            assert_eq!(parts.len(), 2);
            assert_eq!(parts[0], InterpolationPart::Text("result: ".to_string()));
            assert_eq!(
                parts[1],
                InterpolationPart::Expression("x + y * 2".to_string())
            );
        }
        _ => panic!("Expected InterpolatedString, got {:?}", token.kind),
    }
}
