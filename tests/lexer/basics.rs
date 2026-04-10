// Basic lexer behavior tests

use metorex::lexer::{Lexer, Position, TokenKind};

// Unit tests for the lexer core implementation

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

// ── advance() returning None at end (lexer/mod.rs line 48) ─────────────

#[test]
fn test_lexer_advance_past_end() {
    let mut lexer = Lexer::new("");
    let tok = lexer.next_token();
    assert_eq!(tok.kind, TokenKind::EOF);
    let tok2 = lexer.next_token();
    assert_eq!(tok2.kind, TokenKind::EOF);
}

// ── Unknown character (lexer/mod.rs line 667) ──────────────────────────

#[test]
fn test_lexer_unknown_character() {
    let mut lexer = Lexer::new("~");
    let tok = lexer.next_token();
    assert_eq!(tok.kind, TokenKind::EOF);
}

// ── From additional_tests ───────────────────────────────────────────────────

#[test]
fn lexer_empty_input_returns_eof() {
    let mut lexer = Lexer::new("");
    assert_eq!(lexer.next_token().kind, TokenKind::EOF);
}

#[test]
fn lexer_unknown_character_returns_eof() {
    let mut lexer = Lexer::new("~");
    assert_eq!(lexer.next_token().kind, TokenKind::EOF);
}

#[test]
fn lexer_tilde_is_unknown() {
    let mut lexer = Lexer::new("~");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::EOF);
}

#[test]
fn lexer_backtick_command_string() {
    let tokens = Lexer::new("`echo hello`").tokenize();
    assert!(matches!(&tokens[0].kind, TokenKind::String(s) if s == "echo hello"));
}

#[test]
fn lexer_tokenize_collects_all_tokens() {
    let tokens = Lexer::new("x = 1").tokenize();
    assert!(tokens.len() >= 4); // x, =, 1, EOF
}

#[test]
fn lexer_tokenize_empty_source() {
    let tokens = Lexer::new("").tokenize();
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].kind, TokenKind::EOF);
}

#[test]
fn lexer_skips_tabs_and_carriage_returns() {
    let tokens = Lexer::new("\t\rx = 1").tokenize();
    assert!(matches!(tokens[0].kind, TokenKind::Ident(_)));
}

#[test]
fn lexer_newline_token() {
    let tokens = Lexer::new("x\ny").tokenize();
    assert!(tokens.iter().any(|t| t.kind == TokenKind::Newline));
}

#[test]
fn lexer_comment_token() {
    let tokens = Lexer::new("# comment").tokenize();
    assert!(matches!(tokens[0].kind, TokenKind::Comment(_)));
}

#[test]
fn lexer_all_keywords() {
    let kws = "def class if elsif else unless while for in end do begin rescue ensure raise break return lambda super case when then attr_reader attr_writer attr_accessor module include extend true false nil";
    let tokens = Lexer::new(kws).tokenize();
    assert!(tokens.len() > 20);
}

#[test]
fn lexer_position_tracking_across_newlines() {
    let tokens = Lexer::new("x\ny").tokenize();
    let y_token = tokens
        .iter()
        .find(|t| matches!(&t.kind, TokenKind::Ident(n) if n == "y"))
        .unwrap();
    assert_eq!(y_token.position.line, 2);
}
