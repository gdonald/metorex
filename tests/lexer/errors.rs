// Lexer error handling and recovery tests

use metorex::lexer::{InterpolationPart, Lexer, TokenKind};

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

// ===== Error Recovery Tests =====

#[test]
fn test_lexer_standalone_dollar_is_global_var() {
    let mut lexer = Lexer::new("$");
    let token = lexer.next_token();
    // $ with no name produces an empty global variable token
    assert_eq!(token.kind, TokenKind::GlobalVar("".to_string()));
}

#[test]
fn test_lexer_dollar_in_stream() {
    let source = "x = 1 $y z = 2";
    let lexer = Lexer::new(source);
    let tokens: Vec<_> = lexer.collect();

    assert_eq!(tokens[0].kind, TokenKind::Ident("x".to_string()));
    assert_eq!(tokens[1].kind, TokenKind::Equal);
    assert_eq!(tokens[2].kind, TokenKind::Int(1));
    assert_eq!(tokens[3].kind, TokenKind::GlobalVar("y".to_string()));
    assert_eq!(tokens[4].kind, TokenKind::Ident("z".to_string()));
}

#[test]
fn test_lexer_standalone_bang() {
    let mut lexer = Lexer::new("!");
    let token = lexer.next_token();
    // Standalone ! is the logical NOT operator
    assert_eq!(token.kind, TokenKind::Bang);
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

// ── Escape sequence at end of input (lexer/mod.rs lines 348-351) ──────────────

#[test]
fn test_lexer_string_unterminated_after_backslash() {
    // Source: " followed by \ with no more input — hits the None arm in escape handling
    let mut lexer = Lexer::new(r#""\"#);
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::EOF);
}

// ── Newline inside string interpolation (lexer/mod.rs lines 381-384) ──────────

#[test]
fn test_lexer_interpolation_newline_in_expression_is_error() {
    // Source: "#{\n}" — a newline inside #{...} triggers an unterminated-interpolation error
    let mut lexer = Lexer::new("\"#{\n}\"");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::EOF);
}

// ── Nested braces inside interpolation (lexer/mod.rs lines 387-390, 399-400) ──

#[test]
fn test_lexer_interpolation_nested_braces() {
    // Source: "#{{x}}" — the inner {} increments and then decrements depth
    let mut lexer = Lexer::new("\"#{{x}}\"");
    let token = lexer.next_token();
    match token.kind {
        TokenKind::InterpolatedString(parts) => {
            assert_eq!(parts.len(), 1);
            assert_eq!(parts[0], InterpolationPart::Expression("{x}".to_string()));
        }
        _ => panic!("Expected InterpolatedString, got {:?}", token.kind),
    }
}
