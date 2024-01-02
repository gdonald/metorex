// Unit tests for the lexer core implementation

use metorex::lexer::{InterpolationPart, Lexer, Position, TokenKind};

#[test]
fn test_lexer_creation() {
    let source = "test";
    let _lexer = Lexer::new(source);
    // Just verify it compiles and creates successfully
    assert_eq!(source, source);
}

#[test]
fn test_lexer_empty_source() {
    let mut lexer = Lexer::new("");
    let token = lexer.next_token();

    assert_eq!(token.kind, TokenKind::EOF);
    assert_eq!(token.position.line, 1);
    assert_eq!(token.position.column, 1);
    assert_eq!(token.position.offset, 0);
}

#[test]
fn test_lexer_whitespace_only() {
    let mut lexer = Lexer::new("   \t  ");
    let token = lexer.next_token();

    assert_eq!(token.kind, TokenKind::EOF);
}

#[test]
fn test_lexer_single_newline() {
    let mut lexer = Lexer::new("\n");
    let token = lexer.next_token();

    assert_eq!(token.kind, TokenKind::Newline);
    assert_eq!(token.position.line, 1);
    assert_eq!(token.position.column, 1);
}

#[test]
fn test_lexer_multiple_newlines() {
    let mut lexer = Lexer::new("\n\n\n");

    let token1 = lexer.next_token();
    assert_eq!(token1.kind, TokenKind::Newline);
    assert_eq!(token1.position.line, 1);

    let token2 = lexer.next_token();
    assert_eq!(token2.kind, TokenKind::Newline);
    assert_eq!(token2.position.line, 2);

    let token3 = lexer.next_token();
    assert_eq!(token3.kind, TokenKind::Newline);
    assert_eq!(token3.position.line, 3);

    let token4 = lexer.next_token();
    assert_eq!(token4.kind, TokenKind::EOF);
}

#[test]
fn test_lexer_newline_with_whitespace() {
    let mut lexer = Lexer::new("  \n  ");

    let token1 = lexer.next_token();
    assert_eq!(token1.kind, TokenKind::Newline);

    let token2 = lexer.next_token();
    assert_eq!(token2.kind, TokenKind::EOF);
}

#[test]
fn test_lexer_simple_comment() {
    let mut lexer = Lexer::new("# this is a comment");
    let token = lexer.next_token();

    assert_eq!(
        token.kind,
        TokenKind::Comment("this is a comment".to_string())
    );
    assert_eq!(token.position.line, 1);
    assert_eq!(token.position.column, 1);
}

#[test]
fn test_lexer_comment_with_leading_whitespace() {
    let mut lexer = Lexer::new("   # comment with spaces");
    let token = lexer.next_token();

    assert_eq!(
        token.kind,
        TokenKind::Comment("comment with spaces".to_string())
    );
}

#[test]
fn test_lexer_comment_followed_by_newline() {
    let mut lexer = Lexer::new("# comment\n");

    let token1 = lexer.next_token();
    assert_eq!(token1.kind, TokenKind::Comment("comment".to_string()));

    let token2 = lexer.next_token();
    assert_eq!(token2.kind, TokenKind::Newline);

    let token3 = lexer.next_token();
    assert_eq!(token3.kind, TokenKind::EOF);
}

#[test]
fn test_lexer_multiple_comments() {
    let mut lexer = Lexer::new("# comment 1\n# comment 2\n");

    let token1 = lexer.next_token();
    assert_eq!(token1.kind, TokenKind::Comment("comment 1".to_string()));

    let token2 = lexer.next_token();
    assert_eq!(token2.kind, TokenKind::Newline);

    let token3 = lexer.next_token();
    assert_eq!(token3.kind, TokenKind::Comment("comment 2".to_string()));

    let token4 = lexer.next_token();
    assert_eq!(token4.kind, TokenKind::Newline);

    let token5 = lexer.next_token();
    assert_eq!(token5.kind, TokenKind::EOF);
}

#[test]
fn test_lexer_empty_comment() {
    let mut lexer = Lexer::new("#");
    let token = lexer.next_token();

    assert_eq!(token.kind, TokenKind::Comment("".to_string()));
}

#[test]
fn test_lexer_comment_with_special_chars() {
    let mut lexer = Lexer::new("# comment with !@#$%^&*()");
    let token = lexer.next_token();

    assert_eq!(
        token.kind,
        TokenKind::Comment("comment with !@#$%^&*()".to_string())
    );
}

#[test]
fn test_lexer_position_tracking() {
    let mut lexer = Lexer::new("  \n  # comment\n");

    let token1 = lexer.next_token();
    assert_eq!(token1.position, Position::new(1, 3, 2));

    let token2 = lexer.next_token();
    assert_eq!(token2.position, Position::new(2, 3, 5));
}

#[test]
fn test_lexer_carriage_return_handling() {
    let mut lexer = Lexer::new("\r\n");

    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::Newline);
}

#[test]
fn test_lexer_mixed_whitespace() {
    let mut lexer = Lexer::new(" \t \t # comment");
    let token = lexer.next_token();

    assert_eq!(token.kind, TokenKind::Comment("comment".to_string()));
}

#[test]
fn test_lexer_sequential_eof() {
    let mut lexer = Lexer::new("");

    let token1 = lexer.next_token();
    assert_eq!(token1.kind, TokenKind::EOF);

    let token2 = lexer.next_token();
    assert_eq!(token2.kind, TokenKind::EOF);

    // Should be able to call next_token multiple times at EOF
    let token3 = lexer.next_token();
    assert_eq!(token3.kind, TokenKind::EOF);
}

#[test]
fn test_lexer_tabs_as_whitespace() {
    let mut lexer = Lexer::new("\t\t\t# comment");
    let token = lexer.next_token();

    assert_eq!(token.kind, TokenKind::Comment("comment".to_string()));
}

#[test]
fn test_lexer_offset_tracking() {
    let mut lexer = Lexer::new("abc");

    let token1 = lexer.next_token();
    assert_eq!(token1.position.offset, 0);

    let token2 = lexer.next_token();
    assert_eq!(token2.position.offset, 1);

    let token3 = lexer.next_token();
    assert_eq!(token3.position.offset, 2);
}

#[test]
fn test_lexer_column_reset_after_newline() {
    let mut lexer = Lexer::new("\n# comment");

    let token1 = lexer.next_token();
    assert_eq!(token1.position.column, 1);

    let token2 = lexer.next_token();
    assert_eq!(token2.position.line, 2);
    assert_eq!(token2.position.column, 1);
}

#[test]
fn test_lexer_comment_trim() {
    let mut lexer = Lexer::new("#   comment with spaces   ");
    let token = lexer.next_token();

    assert_eq!(
        token.kind,
        TokenKind::Comment("comment with spaces".to_string())
    );
}

#[test]
fn test_lexer_unicode_character() {
    let mut lexer = Lexer::new("🦀");
    let token = lexer.next_token();

    // This will consume the character and return EOF (default case)
    assert_eq!(token.kind, TokenKind::EOF);
    assert_eq!(token.position.offset, 0);
}

#[test]
fn test_lexer_multi_byte_utf8() {
    let mut lexer = Lexer::new("😀abc");

    // First token consumes the emoji
    let token1 = lexer.next_token();
    assert_eq!(token1.kind, TokenKind::EOF);
    assert_eq!(token1.position.offset, 0);

    // Next tokens consume the letters
    let token2 = lexer.next_token();
    assert_eq!(token2.position.offset, 4); // emoji is 4 bytes
}

#[test]
fn test_lexer_unknown_character_handling() {
    let mut lexer = Lexer::new("x");

    let token = lexer.next_token();
    // Unknown characters return EOF for now (will be expanded later)
    assert_eq!(token.kind, TokenKind::EOF);
}

#[test]
fn test_lexer_comment_at_eof() {
    let mut lexer = Lexer::new("# comment at end");

    let token1 = lexer.next_token();
    assert_eq!(
        token1.kind,
        TokenKind::Comment("comment at end".to_string())
    );

    // After comment with no newline, should get EOF
    let token2 = lexer.next_token();
    assert_eq!(token2.kind, TokenKind::EOF);
}

#[test]
fn test_lexer_newline_position_before_advance() {
    let mut lexer = Lexer::new("  \nabc");

    let token1 = lexer.next_token();
    assert_eq!(token1.kind, TokenKind::Newline);
    assert_eq!(token1.position.line, 1);
    assert_eq!(token1.position.column, 3);

    // After newline, line increments
    let token2 = lexer.next_token();
    assert_eq!(token2.position.line, 2);
}

#[test]
fn test_lexer_advance_returns_none_at_eof() {
    let mut lexer = Lexer::new("");

    let token1 = lexer.next_token();
    assert_eq!(token1.kind, TokenKind::EOF);

    // Multiple calls should still return EOF
    let token2 = lexer.next_token();
    assert_eq!(token2.kind, TokenKind::EOF);
}

#[test]
fn test_lexer_comment_with_no_newline_at_eof() {
    let mut lexer = Lexer::new("# test");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::Comment("test".to_string()));
}

#[test]
fn test_lexer_non_newline_then_newline() {
    let mut lexer = Lexer::new("x\n");

    let token1 = lexer.next_token();
    assert_eq!(token1.kind, TokenKind::EOF); // 'x' returns EOF in skeleton

    let token2 = lexer.next_token();
    assert_eq!(token2.kind, TokenKind::Newline);
}

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

// ===== String Interpolation Tests =====

#[test]
fn test_lexer_interpolated_string_simple() {
    let mut lexer = Lexer::new(r#""hello {name}""#);
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
    let mut lexer = Lexer::new(r#""{x} + {y} = {z}""#);
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
    let mut lexer = Lexer::new(r#""{greeting}, world!""#);
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
    let mut lexer = Lexer::new(r#""result: {value}""#);
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
    let mut lexer = Lexer::new(r#""{value}""#);
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
    let mut lexer = Lexer::new("'{name}'");
    let token = lexer.next_token();
    // Single quotes don't support interpolation
    assert_eq!(token.kind, TokenKind::String("{name}".to_string()));
}

#[test]
fn test_lexer_string_escaped_brace() {
    let mut lexer = Lexer::new(r#""\{not_interpolated}""#);
    let token = lexer.next_token();
    // Escaped brace should not trigger interpolation
    assert_eq!(
        token.kind,
        TokenKind::String("{not_interpolated}".to_string())
    );
}

#[test]
fn test_lexer_interpolated_string_with_complex_expression() {
    let mut lexer = Lexer::new(r#""result: {x + y * 2}""#);
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

// ===== Error Handling Tests =====

#[test]
fn test_lexer_unterminated_string_double_quotes() {
    let mut lexer = Lexer::new(r#""hello"#);
    let token = lexer.next_token();
    // Should return EOF on error (temporary error handling)
    assert_eq!(token.kind, TokenKind::EOF);
}

#[test]
fn test_lexer_unterminated_string_single_quotes() {
    let mut lexer = Lexer::new("'hello");
    let token = lexer.next_token();
    // Should return EOF on error (temporary error handling)
    assert_eq!(token.kind, TokenKind::EOF);
}

#[test]
fn test_lexer_string_with_newline_unescaped() {
    let mut lexer = Lexer::new("\"hello\nworld\"");
    let token = lexer.next_token();
    // Should return EOF on error (newline in string is not allowed)
    assert_eq!(token.kind, TokenKind::EOF);
}

#[test]
fn test_lexer_unterminated_interpolation() {
    let mut lexer = Lexer::new(r#""hello {name"#);
    let token = lexer.next_token();
    // Should return EOF on error
    assert_eq!(token.kind, TokenKind::EOF);
}

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
