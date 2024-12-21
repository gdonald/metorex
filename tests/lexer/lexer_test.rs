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
    let mut lexer = Lexer::new("a b c");

    let token1 = lexer.next_token();
    assert_eq!(token1.position.offset, 0);
    assert_eq!(token1.kind, TokenKind::Ident("a".to_string()));

    let token2 = lexer.next_token();
    assert_eq!(token2.position.offset, 2);
    assert_eq!(token2.kind, TokenKind::Ident("b".to_string()));

    let token3 = lexer.next_token();
    assert_eq!(token3.position.offset, 4);
    assert_eq!(token3.kind, TokenKind::Ident("c".to_string()));
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
fn test_lexer_identifier_character_handling() {
    let mut lexer = Lexer::new("x");

    let token = lexer.next_token();
    // Identifiers are now recognized
    assert_eq!(token.kind, TokenKind::Ident("x".to_string()));
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
fn test_lexer_identifier_then_newline() {
    let mut lexer = Lexer::new("x\n");

    let token1 = lexer.next_token();
    assert_eq!(token1.kind, TokenKind::Ident("x".to_string()));

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
    let mut lexer = Lexer::new(r##""hello #{name"##);
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

// ===== Identifier Tests =====

#[test]
fn test_lexer_simple_identifier() {
    let mut lexer = Lexer::new("foo");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::Ident("foo".to_string()));
}

#[test]
fn test_lexer_identifier_with_underscores() {
    let mut lexer = Lexer::new("foo_bar_baz");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::Ident("foo_bar_baz".to_string()));
}

#[test]
fn test_lexer_identifier_starting_with_underscore() {
    let mut lexer = Lexer::new("_private");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::Ident("_private".to_string()));
}

#[test]
fn test_lexer_identifier_with_numbers() {
    let mut lexer = Lexer::new("var123");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::Ident("var123".to_string()));
}

#[test]
fn test_lexer_identifier_all_underscores() {
    let mut lexer = Lexer::new("___");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::Ident("___".to_string()));
}

#[test]
fn test_lexer_single_letter_identifier() {
    let mut lexer = Lexer::new("x");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::Ident("x".to_string()));
}

#[test]
fn test_lexer_multiple_identifiers() {
    let mut lexer = Lexer::new("foo bar baz");

    let token1 = lexer.next_token();
    assert_eq!(token1.kind, TokenKind::Ident("foo".to_string()));

    let token2 = lexer.next_token();
    assert_eq!(token2.kind, TokenKind::Ident("bar".to_string()));

    let token3 = lexer.next_token();
    assert_eq!(token3.kind, TokenKind::Ident("baz".to_string()));
}

#[test]
fn test_lexer_uppercase_identifier() {
    let mut lexer = Lexer::new("FOO");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::Ident("FOO".to_string()));
}

#[test]
fn test_lexer_mixed_case_identifier() {
    let mut lexer = Lexer::new("FooBar");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::Ident("FooBar".to_string()));
}

// ===== Keyword Tests =====

#[test]
fn test_lexer_keyword_def() {
    let mut lexer = Lexer::new("def");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::Def);
}

#[test]
fn test_lexer_keyword_class() {
    let mut lexer = Lexer::new("class");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::Class);
}

#[test]
fn test_lexer_keyword_if() {
    let mut lexer = Lexer::new("if");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::If);
}

#[test]
fn test_lexer_keyword_else() {
    let mut lexer = Lexer::new("else");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::Else);
}

#[test]
fn test_lexer_keyword_while() {
    let mut lexer = Lexer::new("while");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::While);
}

#[test]
fn test_lexer_keyword_end() {
    let mut lexer = Lexer::new("end");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::End);
}

#[test]
fn test_lexer_keyword_do() {
    let mut lexer = Lexer::new("do");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::Do);
}

#[test]
fn test_lexer_keyword_true() {
    let mut lexer = Lexer::new("true");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::True);
}

#[test]
fn test_lexer_keyword_false() {
    let mut lexer = Lexer::new("false");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::False);
}

#[test]
fn test_lexer_keyword_nil() {
    let mut lexer = Lexer::new("nil");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::Nil);
}

#[test]
fn test_lexer_keyword_not_partial_match() {
    let mut lexer = Lexer::new("definition");
    let token = lexer.next_token();
    // Should be identifier, not keyword
    assert_eq!(token.kind, TokenKind::Ident("definition".to_string()));
}

#[test]
fn test_lexer_keyword_with_suffix() {
    let mut lexer = Lexer::new("class_name");
    let token = lexer.next_token();
    // Should be identifier, not keyword
    assert_eq!(token.kind, TokenKind::Ident("class_name".to_string()));
}

#[test]
fn test_lexer_keyword_case_sensitive() {
    let mut lexer = Lexer::new("Class");
    let token = lexer.next_token();
    // Keywords are case-sensitive, so this should be an identifier
    assert_eq!(token.kind, TokenKind::Ident("Class".to_string()));
}

#[test]
fn test_lexer_multiple_keywords() {
    let mut lexer = Lexer::new("if else while end");

    let token1 = lexer.next_token();
    assert_eq!(token1.kind, TokenKind::If);

    let token2 = lexer.next_token();
    assert_eq!(token2.kind, TokenKind::Else);

    let token3 = lexer.next_token();
    assert_eq!(token3.kind, TokenKind::While);

    let token4 = lexer.next_token();
    assert_eq!(token4.kind, TokenKind::End);
}

// ===== Instance Variable Tests =====

#[test]
fn test_lexer_instance_variable() {
    let mut lexer = Lexer::new("@name");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::InstanceVar("name".to_string()));
}

#[test]
fn test_lexer_instance_variable_with_underscores() {
    let mut lexer = Lexer::new("@_private_var");
    let token = lexer.next_token();
    assert_eq!(
        token.kind,
        TokenKind::InstanceVar("_private_var".to_string())
    );
}

#[test]
fn test_lexer_instance_variable_with_numbers() {
    let mut lexer = Lexer::new("@value123");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::InstanceVar("value123".to_string()));
}

#[test]
fn test_lexer_instance_variable_single_char() {
    let mut lexer = Lexer::new("@x");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::InstanceVar("x".to_string()));
}

#[test]
fn test_lexer_multiple_instance_variables() {
    let mut lexer = Lexer::new("@foo @bar @baz");

    let token1 = lexer.next_token();
    assert_eq!(token1.kind, TokenKind::InstanceVar("foo".to_string()));

    let token2 = lexer.next_token();
    assert_eq!(token2.kind, TokenKind::InstanceVar("bar".to_string()));

    let token3 = lexer.next_token();
    assert_eq!(token3.kind, TokenKind::InstanceVar("baz".to_string()));
}

// ===== Class Variable Tests =====

#[test]
fn test_lexer_class_variable() {
    let mut lexer = Lexer::new("@@count");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::ClassVar("count".to_string()));
}

#[test]
fn test_lexer_class_variable_with_underscores() {
    let mut lexer = Lexer::new("@@_shared_state");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::ClassVar("_shared_state".to_string()));
}

#[test]
fn test_lexer_class_variable_with_numbers() {
    let mut lexer = Lexer::new("@@version2");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::ClassVar("version2".to_string()));
}

#[test]
fn test_lexer_class_variable_single_char() {
    let mut lexer = Lexer::new("@@x");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::ClassVar("x".to_string()));
}

#[test]
fn test_lexer_multiple_class_variables() {
    let mut lexer = Lexer::new("@@foo @@bar @@baz");

    let token1 = lexer.next_token();
    assert_eq!(token1.kind, TokenKind::ClassVar("foo".to_string()));

    let token2 = lexer.next_token();
    assert_eq!(token2.kind, TokenKind::ClassVar("bar".to_string()));

    let token3 = lexer.next_token();
    assert_eq!(token3.kind, TokenKind::ClassVar("baz".to_string()));
}

// ===== Mixed Identifier, Keyword, and Variable Tests =====

#[test]
fn test_lexer_identifier_and_keyword() {
    let mut lexer = Lexer::new("var if name");

    let token1 = lexer.next_token();
    assert_eq!(token1.kind, TokenKind::Ident("var".to_string()));

    let token2 = lexer.next_token();
    assert_eq!(token2.kind, TokenKind::If);

    let token3 = lexer.next_token();
    assert_eq!(token3.kind, TokenKind::Ident("name".to_string()));
}

#[test]
fn test_lexer_all_variable_types() {
    let mut lexer = Lexer::new("local @instance @@class");

    let token1 = lexer.next_token();
    assert_eq!(token1.kind, TokenKind::Ident("local".to_string()));

    let token2 = lexer.next_token();
    assert_eq!(token2.kind, TokenKind::InstanceVar("instance".to_string()));

    let token3 = lexer.next_token();
    assert_eq!(token3.kind, TokenKind::ClassVar("class".to_string()));
}

#[test]
fn test_lexer_keywords_and_variables() {
    let mut lexer = Lexer::new("def @name class @@count end");

    let token1 = lexer.next_token();
    assert_eq!(token1.kind, TokenKind::Def);

    let token2 = lexer.next_token();
    assert_eq!(token2.kind, TokenKind::InstanceVar("name".to_string()));

    let token3 = lexer.next_token();
    assert_eq!(token3.kind, TokenKind::Class);

    let token4 = lexer.next_token();
    assert_eq!(token4.kind, TokenKind::ClassVar("count".to_string()));

    let token5 = lexer.next_token();
    assert_eq!(token5.kind, TokenKind::End);
}

#[test]
fn test_lexer_identifier_with_keyword_substring() {
    let mut lexer = Lexer::new("ifdef classify endgame");

    let token1 = lexer.next_token();
    assert_eq!(token1.kind, TokenKind::Ident("ifdef".to_string()));

    let token2 = lexer.next_token();
    assert_eq!(token2.kind, TokenKind::Ident("classify".to_string()));

    let token3 = lexer.next_token();
    assert_eq!(token3.kind, TokenKind::Ident("endgame".to_string()));
}

#[test]
fn test_lexer_empty_instance_variable() {
    let mut lexer = Lexer::new("@ ");
    let token = lexer.next_token();
    // Instance variable with empty name
    assert_eq!(token.kind, TokenKind::InstanceVar("".to_string()));
}

#[test]
fn test_lexer_empty_class_variable() {
    let mut lexer = Lexer::new("@@ ");
    let token = lexer.next_token();
    // Class variable with empty name
    assert_eq!(token.kind, TokenKind::ClassVar("".to_string()));
}

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

// ===== Iterator Tests =====

#[test]
fn test_lexer_iterator_basic() {
    let lexer = Lexer::new("1 2 3");
    let tokens: Vec<_> = lexer.collect();

    assert_eq!(tokens.len(), 3);
    assert_eq!(tokens[0].kind, TokenKind::Int(1));
    assert_eq!(tokens[1].kind, TokenKind::Int(2));
    assert_eq!(tokens[2].kind, TokenKind::Int(3));
}

#[test]
fn test_lexer_iterator_for_loop() {
    let lexer = Lexer::new("x y z");
    let mut count = 0;

    for token in lexer {
        count += 1;
        assert!(matches!(token.kind, TokenKind::Ident(_)));
    }

    assert_eq!(count, 3);
}

#[test]
fn test_lexer_iterator_filter() {
    let lexer = Lexer::new("1 + 2 - 3");
    let operators: Vec<_> = lexer
        .filter(|t| {
            matches!(
                t.kind,
                TokenKind::Plus | TokenKind::Minus | TokenKind::Star | TokenKind::Slash
            )
        })
        .collect();

    assert_eq!(operators.len(), 2);
    assert_eq!(operators[0].kind, TokenKind::Plus);
    assert_eq!(operators[1].kind, TokenKind::Minus);
}

#[test]
fn test_lexer_iterator_map() {
    let lexer = Lexer::new("x y z");
    let positions: Vec<_> = lexer.map(|t| t.position.column).collect();

    assert_eq!(positions.len(), 3);
    assert_eq!(positions[0], 1);
    assert_eq!(positions[1], 3);
    assert_eq!(positions[2], 5);
}

#[test]
fn test_lexer_tokenize_helper() {
    let lexer = Lexer::new("1 + 2");
    let tokens = lexer.tokenize();

    assert_eq!(tokens.len(), 4); // 1, +, 2, EOF
    assert_eq!(tokens[0].kind, TokenKind::Int(1));
    assert_eq!(tokens[1].kind, TokenKind::Plus);
    assert_eq!(tokens[2].kind, TokenKind::Int(2));
    assert_eq!(tokens[3].kind, TokenKind::EOF);
}

#[test]
fn test_lexer_peek_token() {
    let mut lexer = Lexer::new("42");

    // Peek should not consume
    let peeked = lexer.peek_token();
    assert_eq!(peeked.kind, TokenKind::Int(42));

    // Next should return the same token
    let next = lexer.next_token();
    assert_eq!(next.kind, TokenKind::Int(42));

    // After consuming, peek should return EOF
    let peeked_eof = lexer.peek_token();
    assert_eq!(peeked_eof.kind, TokenKind::EOF);
}

// ===== Integration Tests =====

#[test]
fn test_lexer_full_expression() {
    let source = "def add(x, y)\n  x + y\nend";
    let lexer = Lexer::new(source);
    let tokens: Vec<_> = lexer.collect();

    assert_eq!(tokens[0].kind, TokenKind::Def);
    assert_eq!(tokens[1].kind, TokenKind::Ident("add".to_string()));
    assert_eq!(tokens[2].kind, TokenKind::LParen);
    assert_eq!(tokens[3].kind, TokenKind::Ident("x".to_string()));
    assert_eq!(tokens[4].kind, TokenKind::Comma);
    assert_eq!(tokens[5].kind, TokenKind::Ident("y".to_string()));
    assert_eq!(tokens[6].kind, TokenKind::RParen);
    assert_eq!(tokens[7].kind, TokenKind::Newline);
    assert_eq!(tokens[8].kind, TokenKind::Ident("x".to_string()));
    assert_eq!(tokens[9].kind, TokenKind::Plus);
    assert_eq!(tokens[10].kind, TokenKind::Ident("y".to_string()));
    assert_eq!(tokens[11].kind, TokenKind::Newline);
    assert_eq!(tokens[12].kind, TokenKind::End);
}

#[test]
fn test_lexer_class_definition() {
    let source = "class Person\n  def initialize(@name, @age)\n    @@count += 1\n  end\nend";
    let mut lexer = Lexer::new(source);

    assert_eq!(lexer.next_token().kind, TokenKind::Class);
    assert_eq!(
        lexer.next_token().kind,
        TokenKind::Ident("Person".to_string())
    );
    assert_eq!(lexer.next_token().kind, TokenKind::Newline);
    assert_eq!(lexer.next_token().kind, TokenKind::Def);
    assert_eq!(
        lexer.next_token().kind,
        TokenKind::Ident("initialize".to_string())
    );
    assert_eq!(lexer.next_token().kind, TokenKind::LParen);
    assert_eq!(
        lexer.next_token().kind,
        TokenKind::InstanceVar("name".to_string())
    );
    assert_eq!(lexer.next_token().kind, TokenKind::Comma);
    assert_eq!(
        lexer.next_token().kind,
        TokenKind::InstanceVar("age".to_string())
    );
    assert_eq!(lexer.next_token().kind, TokenKind::RParen);
    assert_eq!(lexer.next_token().kind, TokenKind::Newline);
    assert_eq!(
        lexer.next_token().kind,
        TokenKind::ClassVar("count".to_string())
    );
    assert_eq!(lexer.next_token().kind, TokenKind::PlusEqual);
}

#[test]
fn test_lexer_conditionals_and_loops() {
    let source = "if x > 0\n  while y < 10\n    y += 1\n  end\nelse\n  x = 0\nend";
    let lexer = Lexer::new(source);
    let tokens: Vec<_> = lexer.collect();

    // Verify key tokens exist
    assert!(tokens.iter().any(|t| t.kind == TokenKind::If));
    assert!(tokens.iter().any(|t| t.kind == TokenKind::While));
    assert!(tokens.iter().any(|t| t.kind == TokenKind::Else));
    assert!(tokens.iter().any(|t| t.kind == TokenKind::End));
    assert!(tokens.iter().any(|t| t.kind == TokenKind::PlusEqual));
}

#[test]
fn test_lexer_array_and_hash_syntax() {
    let source = "[1, 2, 3] {x: 1, y: 2}";
    let lexer = Lexer::new(source);
    let tokens: Vec<_> = lexer.collect();

    assert_eq!(tokens[0].kind, TokenKind::LBracket);
    assert_eq!(tokens[1].kind, TokenKind::Int(1));
    assert_eq!(tokens[2].kind, TokenKind::Comma);
    assert_eq!(tokens[3].kind, TokenKind::Int(2));
    assert_eq!(tokens[4].kind, TokenKind::Comma);
    assert_eq!(tokens[5].kind, TokenKind::Int(3));
    assert_eq!(tokens[6].kind, TokenKind::RBracket);
    assert_eq!(tokens[7].kind, TokenKind::LBrace);
    assert_eq!(tokens[8].kind, TokenKind::Ident("x".to_string()));
    assert_eq!(tokens[9].kind, TokenKind::Colon);
}

#[test]
fn test_lexer_string_interpolation_complex() {
    let source = r##""Hello #{name}, you are #{age} years old""##;
    let mut lexer = Lexer::new(source);
    let token = lexer.next_token();

    match token.kind {
        TokenKind::InterpolatedString(parts) => {
            assert_eq!(parts.len(), 5);
            assert_eq!(parts[0], InterpolationPart::Text("Hello ".to_string()));
            assert_eq!(parts[1], InterpolationPart::Expression("name".to_string()));
            assert_eq!(parts[2], InterpolationPart::Text(", you are ".to_string()));
            assert_eq!(parts[3], InterpolationPart::Expression("age".to_string()));
            assert_eq!(parts[4], InterpolationPart::Text(" years old".to_string()));
        }
        _ => panic!("Expected InterpolatedString"),
    }
}

#[test]
fn test_lexer_mixed_literals_and_operators() {
    let source = "42 + 3.14 * \"hello\" - true";
    let lexer = Lexer::new(source);
    let tokens: Vec<_> = lexer.collect();

    assert_eq!(tokens[0].kind, TokenKind::Int(42));
    assert_eq!(tokens[1].kind, TokenKind::Plus);
    assert_eq!(tokens[2].kind, TokenKind::Float(3.14));
    assert_eq!(tokens[3].kind, TokenKind::Star);
    assert_eq!(tokens[4].kind, TokenKind::String("hello".to_string()));
    assert_eq!(tokens[5].kind, TokenKind::Minus);
    assert_eq!(tokens[6].kind, TokenKind::True);
}

#[test]
fn test_lexer_comments_in_code() {
    let source = "x = 1 # assign x\ny = 2 # assign y\n# final comment";
    let lexer = Lexer::new(source);
    let tokens: Vec<_> = lexer.collect();

    // Should have: x, =, 1, comment, newline, y, =, 2, comment, newline, comment
    assert!(
        tokens
            .iter()
            .any(|t| matches!(&t.kind, TokenKind::Comment(c) if c == "assign x"))
    );
    assert!(
        tokens
            .iter()
            .any(|t| matches!(&t.kind, TokenKind::Comment(c) if c == "assign y"))
    );
    assert!(
        tokens
            .iter()
            .any(|t| matches!(&t.kind, TokenKind::Comment(c) if c == "final comment"))
    );
}

// ===== Error Recovery Tests =====

#[test]
fn test_lexer_invalid_character() {
    let mut lexer = Lexer::new("$");
    let token = lexer.next_token();
    // Invalid characters return EOF
    assert_eq!(token.kind, TokenKind::EOF);
}

#[test]
fn test_lexer_invalid_character_in_stream() {
    let source = "x = 1 $ y = 2";
    let lexer = Lexer::new(source);
    let tokens: Vec<_> = lexer.collect();

    // Should lex valid tokens and skip invalid character
    assert_eq!(tokens[0].kind, TokenKind::Ident("x".to_string()));
    assert_eq!(tokens[1].kind, TokenKind::Equal);
    assert_eq!(tokens[2].kind, TokenKind::Int(1));
    // $ returns EOF, so iteration stops
}

#[test]
fn test_lexer_standalone_bang() {
    let mut lexer = Lexer::new("!");
    let token = lexer.next_token();
    // Standalone ! returns EOF (not yet implemented as unary operator)
    assert_eq!(token.kind, TokenKind::EOF);
}

#[test]
fn test_lexer_empty_string_edge_case() {
    let mut lexer = Lexer::new(r#""""#);
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::String("".to_string()));
}

#[test]
fn test_lexer_position_accuracy() {
    let source = "abc\ndef\nghi";
    let lexer = Lexer::new(source);
    let tokens: Vec<_> = lexer.collect();

    // First identifier on line 1
    assert_eq!(tokens[0].position.line, 1);
    assert_eq!(tokens[0].position.column, 1);

    // Newline
    assert_eq!(tokens[1].kind, TokenKind::Newline);

    // Second identifier on line 2
    assert_eq!(tokens[2].position.line, 2);
    assert_eq!(tokens[2].position.column, 1);

    // Newline
    assert_eq!(tokens[3].kind, TokenKind::Newline);

    // Third identifier on line 3
    assert_eq!(tokens[4].position.line, 3);
    assert_eq!(tokens[4].position.column, 1);
}

#[test]
fn test_lexer_long_source() {
    let source = "x = 1\ny = 2\nz = 3\nw = 4\nv = 5";
    let lexer = Lexer::new(source);
    let tokens: Vec<_> = lexer.collect();

    // Should have: 5 * (ident, =, int, newline) - 1 (last newline missing)
    assert!(tokens.len() >= 19); // At least these tokens
}

#[test]
fn test_lexer_nested_expressions() {
    let source = "((x + y) * (a - b))";
    let lexer = Lexer::new(source);
    let tokens: Vec<_> = lexer.collect();

    assert_eq!(tokens[0].kind, TokenKind::LParen);
    assert_eq!(tokens[1].kind, TokenKind::LParen);
    assert_eq!(tokens[11].kind, TokenKind::RParen);
    assert_eq!(tokens[12].kind, TokenKind::RParen);
}

#[test]
fn test_lexer_all_delimiters_balanced() {
    let source = "(){}[]";
    let lexer = Lexer::new(source);
    let tokens: Vec<_> = lexer.collect();

    assert_eq!(tokens[0].kind, TokenKind::LParen);
    assert_eq!(tokens[1].kind, TokenKind::RParen);
    assert_eq!(tokens[2].kind, TokenKind::LBrace);
    assert_eq!(tokens[3].kind, TokenKind::RBrace);
    assert_eq!(tokens[4].kind, TokenKind::LBracket);
    assert_eq!(tokens[5].kind, TokenKind::RBracket);
}

#[test]
fn test_lexer_semicolon_separator() {
    let source = "x = 1; y = 2; z = 3";
    let lexer = Lexer::new(source);
    let tokens: Vec<_> = lexer.collect();

    assert_eq!(tokens[3].kind, TokenKind::Semicolon);
    assert_eq!(tokens[7].kind, TokenKind::Semicolon);
}

#[test]
fn test_lexer_method_chain_long() {
    let source = "obj.method1().method2().method3()";
    let lexer = Lexer::new(source);
    let tokens: Vec<_> = lexer.collect();

    let dots: Vec<_> = tokens.iter().filter(|t| t.kind == TokenKind::Dot).collect();
    assert_eq!(dots.len(), 3);
}
