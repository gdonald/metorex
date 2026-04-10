// String literal tests

use metorex::lexer::{Lexer, TokenKind};

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

#[test]
fn test_lexer_string_with_esc_escape() {
    let mut lexer = Lexer::new(r#""\e[32m""#);
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::String("\x1b[32m".to_string()));
}

#[test]
fn test_lexer_string_with_null_escape() {
    let mut lexer = Lexer::new(r#""\0""#);
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::String("\0".to_string()));
}

// ── Unterminated strings (lexer/mod.rs lines 281-289) ──────────────────

#[test]
fn test_lexer_unterminated_double_string() {
    let mut lexer = Lexer::new("\"hello");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::EOF);
}

#[test]
fn test_lexer_unterminated_string_at_newline() {
    let mut lexer = Lexer::new("\"hello\n");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::EOF);
}

#[test]
fn test_lexer_escape_at_eof() {
    let mut lexer = Lexer::new("\"hello\\");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::EOF);
}

// ── Escape sequences: \r and \' ────────────────────────────────────────

#[test]
fn test_lexer_escape_return() {
    let mut lexer = Lexer::new(r#""hello\rworld""#);
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::String("hello\rworld".to_string()));
}

#[test]
fn test_lexer_escape_single_quote_in_double() {
    let mut lexer = Lexer::new(r#""hello\'world""#);
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::String("hello'world".to_string()));
}

#[test]
fn test_lexer_escape_hash_literal() {
    let mut lexer = Lexer::new(r#""hello\#world""#);
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::String("hello#world".to_string()));
}

// ── Interpolation edge cases (lexer/mod.rs lines 358-402) ──────────────

#[test]
fn test_lexer_interpolation_unterminated() {
    let mut lexer = Lexer::new("\"hello #{name");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::EOF);
}

#[test]
fn test_lexer_interpolation_newline_inside() {
    let mut lexer = Lexer::new("\"hello #{name\n}\"");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::EOF);
}

#[test]
fn test_lexer_interpolation_nested_braces() {
    let mut lexer = Lexer::new("\"#{{1 => 2}}\"");
    let token = lexer.next_token();
    assert!(matches!(token.kind, TokenKind::InterpolatedString(_)));
}

// ── %[] / %Q[] / %() / %{} / %<> string literals ───────────────────────────

#[test]
fn test_percent_bracket_string() {
    let tokens = Lexer::new("%[hello world]").tokenize();
    assert!(matches!(&tokens[0].kind, TokenKind::String(s) if s == "hello world"));
}

#[test]
fn test_percent_q_string() {
    let tokens = Lexer::new("%Q[hello]").tokenize();
    assert!(matches!(&tokens[0].kind, TokenKind::String(s) if s == "hello"));
}

#[test]
fn test_percent_paren_string() {
    let tokens = Lexer::new("%(hello world)").tokenize();
    assert!(matches!(&tokens[0].kind, TokenKind::String(s) if s == "hello world"));
}

#[test]
fn test_percent_brace_string() {
    let tokens = Lexer::new("%{hello}").tokenize();
    assert!(matches!(&tokens[0].kind, TokenKind::String(s) if s == "hello"));
}

#[test]
fn test_percent_angle_string() {
    let tokens = Lexer::new("%<hello>").tokenize();
    assert!(matches!(&tokens[0].kind, TokenKind::String(s) if s == "hello"));
}

// ── From additional_tests ───────────────────────────────────────────────────

#[test]
fn lexer_unterminated_single_quote_string() {
    let tokens = Lexer::new("'unterminated").tokenize();
    assert!(tokens.iter().any(|t| t.kind == TokenKind::EOF));
}

#[test]
fn lexer_newline_in_single_quote_string() {
    let tokens = Lexer::new("'line1\nline2'").tokenize();
    assert!(tokens.iter().any(|t| t.kind == TokenKind::EOF));
}
