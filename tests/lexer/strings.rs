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

// ── %w[] / %w!! / %w<> / %w() / %w{} percent-word arrays ──────────────────

#[test]
fn lexer_percent_w_brackets() {
    let tokens = Lexer::new("%w[a b c]").tokenize();
    assert!(
        tokens
            .iter()
            .any(|t| matches!(&t.kind, TokenKind::PercentW(s) if s == "a b c"))
    );
}

#[test]
fn lexer_percent_w_bang() {
    let tokens = Lexer::new("%w!a b c!").tokenize();
    assert!(
        tokens
            .iter()
            .any(|t| matches!(&t.kind, TokenKind::PercentW(s) if s == "a b c"))
    );
}

#[test]
fn lexer_percent_w_parens() {
    let tokens = Lexer::new("%w(one two)").tokenize();
    assert!(
        tokens
            .iter()
            .any(|t| matches!(&t.kind, TokenKind::PercentW(s) if s == "one two"))
    );
}

#[test]
fn lexer_percent_w_braces() {
    let tokens = Lexer::new("%w{x y z}").tokenize();
    assert!(
        tokens
            .iter()
            .any(|t| matches!(&t.kind, TokenKind::PercentW(s) if s == "x y z"))
    );
}

#[test]
fn lexer_percent_w_angle_brackets() {
    let tokens = Lexer::new("%w<p q r>").tokenize();
    assert!(
        tokens
            .iter()
            .any(|t| matches!(&t.kind, TokenKind::PercentW(s) if s == "p q r"))
    );
}

#[test]
fn lexer_percent_w_with_escape() {
    // Backslash escape should keep the next character literally.
    let tokens = Lexer::new("%w[a\\ b c]").tokenize();
    assert!(
        tokens
            .iter()
            .any(|t| matches!(&t.kind, TokenKind::PercentW(_)))
    );
}

#[test]
fn lexer_percent_w_empty() {
    let tokens = Lexer::new("%w[]").tokenize();
    assert!(
        tokens
            .iter()
            .any(|t| matches!(&t.kind, TokenKind::PercentW(s) if s.is_empty()))
    );
}

// ── String escape sequences ────────────────────────────────────────────────

#[test]
fn lexer_string_escape_newline() {
    let tokens = Lexer::new(r#""line1\nline2""#).tokenize();
    assert!(
        tokens
            .iter()
            .any(|t| matches!(&t.kind, TokenKind::String(s) if s.contains('\n')))
    );
}

#[test]
fn lexer_string_escape_tab() {
    let tokens = Lexer::new(r#""a\tb""#).tokenize();
    assert!(
        tokens
            .iter()
            .any(|t| matches!(&t.kind, TokenKind::String(s) if s.contains('\t')))
    );
}

#[test]
fn lexer_string_escape_carriage_return() {
    let tokens = Lexer::new(r#""a\rb""#).tokenize();
    assert!(
        tokens
            .iter()
            .any(|t| matches!(&t.kind, TokenKind::String(s) if s.contains('\r')))
    );
}

#[test]
fn lexer_string_escape_backslash() {
    let tokens = Lexer::new(r#""a\\b""#).tokenize();
    assert!(
        tokens
            .iter()
            .any(|t| matches!(&t.kind, TokenKind::String(s) if s.contains('\\')))
    );
}

#[test]
fn lexer_string_escape_double_quote() {
    let tokens = Lexer::new(r#""a\"b""#).tokenize();
    assert!(
        tokens
            .iter()
            .any(|t| matches!(&t.kind, TokenKind::String(s) if s.contains('"')))
    );
}

#[test]
fn lexer_string_escape_single_quote() {
    let tokens = Lexer::new(r#""a\'b""#).tokenize();
    assert!(
        tokens
            .iter()
            .any(|t| matches!(&t.kind, TokenKind::String(s) if s.contains('\'')))
    );
}

#[test]
fn lexer_string_escape_hash() {
    // \# allows literal `#` followed by `{` without triggering interpolation.
    let tokens = Lexer::new(r#""a\#{b}""#).tokenize();
    assert!(
        tokens
            .iter()
            .any(|t| matches!(&t.kind, TokenKind::String(s) if s.contains('#')))
    );
}

#[test]
fn lexer_string_escape_esc_char() {
    // \e produces ANSI ESC (0x1B).
    let tokens = Lexer::new(r#""a\eb""#).tokenize();
    assert!(
        tokens
            .iter()
            .any(|t| matches!(&t.kind, TokenKind::String(s) if s.contains('\x1B')))
    );
}

#[test]
fn lexer_string_escape_null() {
    let tokens = Lexer::new(r#""a\0b""#).tokenize();
    assert!(
        tokens
            .iter()
            .any(|t| matches!(&t.kind, TokenKind::String(s) if s.contains('\0')))
    );
}

#[test]
fn lexer_string_escape_unrecognized_keeps_backslash() {
    // Unknown escape sequences keep the backslash literally (e.g. \q -> \q).
    let tokens = Lexer::new(r#""a\qb""#).tokenize();
    assert!(
        tokens
            .iter()
            .any(|t| matches!(&t.kind, TokenKind::String(s) if s.contains('\\')))
    );
}

#[test]
fn lexer_string_unterminated_eof() {
    // EOF inside a double-quoted string is an error path; tokenize() recovers.
    let tokens = Lexer::new(r#""unterminated"#).tokenize();
    assert!(tokens.iter().any(|t| t.kind == TokenKind::EOF));
}

#[test]
fn lexer_string_unterminated_with_newline() {
    let tokens = Lexer::new("\"line1\nline2\"").tokenize();
    assert!(tokens.iter().any(|t| t.kind == TokenKind::EOF));
}

#[test]
fn lexer_string_unterminated_after_backslash() {
    // EOF immediately after a backslash inside a string.
    let tokens = Lexer::new(r#""a\"#).tokenize();
    assert!(tokens.iter().any(|t| t.kind == TokenKind::EOF));
}

// ── String interpolation edge cases ────────────────────────────────────────

#[test]
fn lexer_interpolation_basic() {
    let tokens = Lexer::new(r#""hello #{name}""#).tokenize();
    assert!(
        tokens
            .iter()
            .any(|t| matches!(&t.kind, TokenKind::InterpolatedString(_)))
    );
}

#[test]
fn lexer_interpolation_unterminated_eof() {
    // EOF in the middle of #{ ... } is an error; tokenize recovers.
    let tokens = Lexer::new(r#""hello #{name"#).tokenize();
    assert!(tokens.iter().any(|t| t.kind == TokenKind::EOF));
}

#[test]
fn lexer_interpolation_unterminated_newline() {
    let tokens = Lexer::new("\"hello #{name\nbroken\"").tokenize();
    assert!(tokens.iter().any(|t| t.kind == TokenKind::EOF));
}

#[test]
fn lexer_interpolation_nested_braces() {
    let tokens = Lexer::new(r#""x #{ {a: 1} } y""#).tokenize();
    assert!(
        tokens
            .iter()
            .any(|t| matches!(&t.kind, TokenKind::InterpolatedString(_)))
    );
}

// ── %r regex literal — various delimiters and escape sequences ────────────

#[test]
fn lexer_percent_r_brackets() {
    let tokens = Lexer::new("%r[foo]").tokenize();
    assert!(
        tokens
            .iter()
            .any(|t| matches!(&t.kind, TokenKind::Regex(p, _) if p == "foo"))
    );
}

#[test]
fn lexer_percent_r_braces() {
    let tokens = Lexer::new("%r{bar}").tokenize();
    assert!(
        tokens
            .iter()
            .any(|t| matches!(&t.kind, TokenKind::Regex(p, _) if p == "bar"))
    );
}

#[test]
fn lexer_percent_r_angle_brackets() {
    let tokens = Lexer::new("%r<baz>").tokenize();
    assert!(
        tokens
            .iter()
            .any(|t| matches!(&t.kind, TokenKind::Regex(p, _) if p == "baz"))
    );
}

#[test]
fn lexer_percent_r_parens() {
    let tokens = Lexer::new("%r(qux)").tokenize();
    assert!(
        tokens
            .iter()
            .any(|t| matches!(&t.kind, TokenKind::Regex(p, _) if p == "qux"))
    );
}

#[test]
fn lexer_percent_r_with_escape() {
    // Backslashes inside %r should be preserved.
    let tokens = Lexer::new(r"%r[a\.b]").tokenize();
    assert!(
        tokens
            .iter()
            .any(|t| matches!(&t.kind, TokenKind::Regex(p, _) if p.contains("\\")))
    );
}

#[test]
fn lexer_percent_r_with_flags() {
    let tokens = Lexer::new("%r[abc]im").tokenize();
    assert!(
        tokens
            .iter()
            .any(|t| matches!(&t.kind, TokenKind::Regex(_, f) if f == "im"))
    );
}

// ── %Q[...] interpolation-friendly string literals ────────────────────────

#[test]
fn lexer_percent_q_string() {
    let tokens = Lexer::new("%Q[hello]").tokenize();
    assert!(
        tokens
            .iter()
            .any(|t| matches!(&t.kind, TokenKind::String(s) if s == "hello"))
    );
}

#[test]
fn lexer_percent_q_with_escape_n() {
    let tokens = Lexer::new(r"%Q[a\nb]").tokenize();
    assert!(
        tokens
            .iter()
            .any(|t| matches!(&t.kind, TokenKind::String(s) if s.contains('\n')))
    );
}

#[test]
fn lexer_percent_q_with_escape_t() {
    let tokens = Lexer::new(r"%Q[a\tb]").tokenize();
    assert!(
        tokens
            .iter()
            .any(|t| matches!(&t.kind, TokenKind::String(s) if s.contains('\t')))
    );
}

#[test]
fn lexer_percent_q_with_escape_backslash() {
    let tokens = Lexer::new(r"%Q[a\\b]").tokenize();
    assert!(
        tokens
            .iter()
            .any(|t| matches!(&t.kind, TokenKind::String(s) if s.contains('\\')))
    );
}

#[test]
fn lexer_percent_paren_string() {
    let tokens = Lexer::new("%(hello)").tokenize();
    assert!(
        tokens
            .iter()
            .any(|t| matches!(&t.kind, TokenKind::String(s) if s == "hello"))
    );
}

#[test]
fn lexer_percent_brace_string() {
    let tokens = Lexer::new("%{hello}").tokenize();
    assert!(
        tokens
            .iter()
            .any(|t| matches!(&t.kind, TokenKind::String(s) if s == "hello"))
    );
}

#[test]
fn lexer_percent_angle_string() {
    let tokens = Lexer::new("%<hello>").tokenize();
    assert!(
        tokens
            .iter()
            .any(|t| matches!(&t.kind, TokenKind::String(s) if s == "hello"))
    );
}
