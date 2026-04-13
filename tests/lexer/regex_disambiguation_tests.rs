// Regex/slash disambiguation tests

use metorex::lexer::{Lexer, TokenKind};

// ── Regex literal coverage ────────────────────────────────────────────────

#[test]
fn regex_with_escaped_slash() {
    let tokens = Lexer::new(r"/path\/to/").tokenize();
    assert_eq!(
        tokens[0].kind,
        TokenKind::Regex("path\\/to".to_string(), "".to_string())
    );
}

#[test]
fn regex_with_character_class() {
    let tokens = Lexer::new("/[a-z]+/").tokenize();
    assert_eq!(
        tokens[0].kind,
        TokenKind::Regex("[a-z]+".to_string(), "".to_string())
    );
}

#[test]
fn regex_slash_inside_character_class() {
    let tokens = Lexer::new("/[/]+/").tokenize();
    assert_eq!(
        tokens[0].kind,
        TokenKind::Regex("[/]+".to_string(), "".to_string())
    );
}

#[test]
fn regex_with_interpolation() {
    let tokens = Lexer::new("/hello#{1+2}world/").tokenize();
    assert_eq!(
        tokens[0].kind,
        TokenKind::Regex("hello#{1+2}world".to_string(), "".to_string())
    );
}

#[test]
fn regex_interpolation_with_nested_braces() {
    let tokens = Lexer::new("/#{a{b}}/").tokenize();
    assert_eq!(
        tokens[0].kind,
        TokenKind::Regex("#{a{b}}".to_string(), "".to_string())
    );
}

#[test]
fn regex_hash_without_interpolation() {
    let tokens = Lexer::new("/#comment/").tokenize();
    assert_eq!(
        tokens[0].kind,
        TokenKind::Regex("#comment".to_string(), "".to_string())
    );
}

#[test]
fn regex_empty() {
    let tokens = Lexer::new("//").tokenize();
    assert_eq!(
        tokens[0].kind,
        TokenKind::Regex("".to_string(), "".to_string())
    );
}

#[test]
fn regex_unterminated_at_newline() {
    let tokens = Lexer::new("/abc\ndef").tokenize();
    assert_eq!(
        tokens[0].kind,
        TokenKind::Regex("abc".to_string(), "".to_string())
    );
}

#[test]
fn regex_newline_inside_interpolation_terminates() {
    let tokens = Lexer::new("/#{x\n}/").tokenize();
    assert_eq!(
        tokens[0].kind,
        TokenKind::Regex("#{x".to_string(), "".to_string())
    );
}

#[test]
fn slash_is_division_after_ident() {
    let tokens = Lexer::new("x / 2").tokenize();
    assert_eq!(tokens[1].kind, TokenKind::Slash);
}

#[test]
fn slash_is_division_after_int() {
    let tokens = Lexer::new("1 / 2").tokenize();
    assert_eq!(tokens[1].kind, TokenKind::Slash);
}

#[test]
fn slash_is_division_after_float() {
    let tokens = Lexer::new("1.0 / 2").tokenize();
    assert_eq!(tokens[1].kind, TokenKind::Slash);
}

#[test]
fn slash_is_division_after_string() {
    let tokens = Lexer::new("\"s\" / 2").tokenize();
    assert_eq!(tokens[1].kind, TokenKind::Slash);
}

#[test]
fn slash_is_division_after_rparen() {
    let tokens = Lexer::new("(x) / 2").tokenize();
    assert_eq!(tokens[3].kind, TokenKind::Slash);
}

#[test]
fn slash_is_division_after_rbracket() {
    let tokens = Lexer::new("a[0] / 2").tokenize();
    let slash = tokens.iter().find(|t| t.kind == TokenKind::Slash);
    assert!(slash.is_some());
}

#[test]
fn slash_is_division_after_end() {
    let tokens = Lexer::new("end / 2").tokenize();
    assert_eq!(tokens[1].kind, TokenKind::Slash);
}

#[test]
fn slash_is_division_after_true() {
    let tokens = Lexer::new("true / 2").tokenize();
    assert_eq!(tokens[1].kind, TokenKind::Slash);
}

#[test]
fn slash_is_division_after_false() {
    let tokens = Lexer::new("false / 2").tokenize();
    assert_eq!(tokens[1].kind, TokenKind::Slash);
}

#[test]
fn slash_is_division_after_nil() {
    let tokens = Lexer::new("nil / 2").tokenize();
    assert_eq!(tokens[1].kind, TokenKind::Slash);
}

#[test]
fn slash_is_regex_after_operator() {
    let tokens = Lexer::new("x = /abc/").tokenize();
    assert!(matches!(&tokens[2].kind, TokenKind::Regex(_, _)));
}

#[test]
fn slash_is_division_after_ivar() {
    let tokens = Lexer::new("@x / 2").tokenize();
    assert_eq!(tokens[1].kind, TokenKind::Slash);
}

#[test]
fn slash_is_division_after_cvar() {
    let tokens = Lexer::new("@@x / 2").tokenize();
    assert_eq!(tokens[1].kind, TokenKind::Slash);
}

#[test]
fn slash_is_division_after_gvar() {
    let tokens = Lexer::new("$x / 2").tokenize();
    assert_eq!(tokens[1].kind, TokenKind::Slash);
}

#[test]
fn slash_is_division_after_magic_file() {
    let tokens = Lexer::new("__FILE__ / 2").tokenize();
    assert_eq!(tokens[1].kind, TokenKind::Slash);
}

#[test]
fn slash_is_division_after_magic_line() {
    let tokens = Lexer::new("__LINE__ / 2").tokenize();
    assert_eq!(tokens[1].kind, TokenKind::Slash);
}

#[test]
fn slash_is_division_after_def() {
    let tokens = Lexer::new("def / end").tokenize();
    assert_eq!(tokens[1].kind, TokenKind::Slash);
}

#[test]
fn regex_escaped_bracket_in_char_class() {
    let tokens = Lexer::new(r"/[a\]b]/").tokenize();
    assert_eq!(
        tokens[0].kind,
        TokenKind::Regex("[a\\]b]".to_string(), "".to_string())
    );
}

#[test]
fn slash_is_division_after_rbrace() {
    let tokens = Lexer::new("} / 2").tokenize();
    assert_eq!(tokens[1].kind, TokenKind::Slash);
}

#[test]
fn slash_is_division_after_regex() {
    let tokens = Lexer::new("/a/ / 2").tokenize();
    let slash = tokens.iter().filter(|t| t.kind == TokenKind::Slash).count();
    assert_eq!(slash, 1);
}

#[test]
fn slash_is_division_after_interpolated_string() {
    let tokens = Lexer::new("\"hello #{x}\" / 2").tokenize();
    let slash = tokens.iter().find(|t| t.kind == TokenKind::Slash);
    assert!(slash.is_some());
}

#[test]
fn slash_is_regex_at_start_of_input() {
    let tokens = Lexer::new("/abc/i").tokenize();
    assert!(matches!(&tokens[0].kind, TokenKind::Regex(_, _)));
}

#[test]
fn slash_is_regex_after_plus() {
    let tokens = Lexer::new("x + /abc/").tokenize();
    assert!(matches!(&tokens[2].kind, TokenKind::Regex(_, _)));
}

#[test]
fn regex_hash_at_end_of_pattern() {
    let tokens = Lexer::new("/#/").tokenize();
    assert_eq!(
        tokens[0].kind,
        TokenKind::Regex("#".to_string(), "".to_string())
    );
}
