// Heredoc literal lexing tests

use metorex::lexer::{Lexer, TokenKind};

#[test]
fn heredoc_dash_basic() {
    let source = "<<-END\nhello world\nEND\n";
    let tokens = Lexer::new(source).tokenize();
    assert_eq!(
        tokens[0].kind,
        TokenKind::String("hello world\n".to_string())
    );
}

#[test]
fn heredoc_dash_indented_terminator() {
    let source = "<<-END\nhello\n  END\n";
    let tokens = Lexer::new(source).tokenize();
    assert_eq!(tokens[0].kind, TokenKind::String("hello\n".to_string()));
}

#[test]
fn heredoc_tilde_strips_indent() {
    let source = "<<~END\n    hello\n    world\n  END\n";
    let tokens = Lexer::new(source).tokenize();
    assert_eq!(
        tokens[0].kind,
        TokenKind::String("hello\nworld\n".to_string())
    );
}

#[test]
fn heredoc_tilde_mixed_indent() {
    let source = "<<~END\n      deep\n    shallow\n  END\n";
    let tokens = Lexer::new(source).tokenize();
    assert_eq!(
        tokens[0].kind,
        TokenKind::String("  deep\nshallow\n".to_string())
    );
}

#[test]
fn heredoc_dash_multiline_body() {
    let source = "<<-EOF\nline one\nline two\nline three\nEOF\n";
    let tokens = Lexer::new(source).tokenize();
    assert_eq!(
        tokens[0].kind,
        TokenKind::String("line one\nline two\nline three\n".to_string())
    );
}

#[test]
fn heredoc_dash_empty_body() {
    let source = "<<-END\nEND\n";
    let tokens = Lexer::new(source).tokenize();
    assert_eq!(tokens[0].kind, TokenKind::String("".to_string()));
}

#[test]
fn heredoc_tilde_empty_body() {
    let source = "<<~END\nEND\n";
    let tokens = Lexer::new(source).tokenize();
    assert_eq!(tokens[0].kind, TokenKind::String("".to_string()));
}

#[test]
fn heredoc_with_double_quoted_terminator() {
    let source = "<<-\"END\"\nhello\nEND\n";
    let tokens = Lexer::new(source).tokenize();
    assert_eq!(tokens[0].kind, TokenKind::String("hello\n".to_string()));
}

#[test]
fn heredoc_with_single_quoted_terminator() {
    let source = "<<-'END'\nhello\nEND\n";
    let tokens = Lexer::new(source).tokenize();
    assert_eq!(tokens[0].kind, TokenKind::String("hello\n".to_string()));
}

#[test]
fn heredoc_with_underscore_in_terminator() {
    let source = "<<-MY_END\ncontent here\nMY_END\n";
    let tokens = Lexer::new(source).tokenize();
    assert_eq!(
        tokens[0].kind,
        TokenKind::String("content here\n".to_string())
    );
}

#[test]
fn heredoc_not_a_heredoc_falls_back_to_shovel() {
    let tokens = Lexer::new("a << b").tokenize();
    assert_eq!(tokens[1].kind, TokenKind::Shovel);
}

#[test]
fn heredoc_tilde_with_blank_lines() {
    let source = "<<~END\n    hello\n\n    world\n  END\n";
    let tokens = Lexer::new(source).tokenize();
    assert_eq!(
        tokens[0].kind,
        TokenKind::String("hello\n\nworld\n".to_string())
    );
}

#[test]
fn heredoc_eof_terminates() {
    let source = "<<-END\nhello";
    let tokens = Lexer::new(source).tokenize();
    assert_eq!(tokens[0].kind, TokenKind::String("hello\n".to_string()));
}

#[test]
fn heredoc_tilde_single_line() {
    let source = "<<~X\n  hi\nX\n";
    let tokens = Lexer::new(source).tokenize();
    assert_eq!(tokens[0].kind, TokenKind::String("hi\n".to_string()));
}

#[test]
fn heredoc_dash_no_trailing_newline_in_body() {
    let source = "<<-END\nsingle line\nEND";
    let tokens = Lexer::new(source).tokenize();
    assert_eq!(
        tokens[0].kind,
        TokenKind::String("single line\n".to_string())
    );
}

#[test]
fn heredoc_invalid_empty_terminator_falls_back() {
    let tokens = Lexer::new("<<- x").tokenize();
    assert_eq!(tokens[0].kind, TokenKind::Shovel);
}

#[test]
fn heredoc_tilde_only_whitespace_lines() {
    let source = "<<~END\n    \n  END\n";
    let tokens = Lexer::new(source).tokenize();
    assert_eq!(tokens[0].kind, TokenKind::String("    \n".to_string()));
}

#[test]
fn heredoc_body_line_at_eof_no_newline() {
    let source = "<<-END\nhello\nworld";
    let tokens = Lexer::new(source).tokenize();
    assert_eq!(
        tokens[0].kind,
        TokenKind::String("hello\nworld\n".to_string())
    );
}

#[test]
fn heredoc_skips_content_after_opener_on_same_line() {
    let source = "<<-END # comment\nhello\nEND\n";
    let tokens = Lexer::new(source).tokenize();
    assert_eq!(tokens[0].kind, TokenKind::String("hello\n".to_string()));
}
