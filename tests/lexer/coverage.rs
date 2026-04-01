// Additional coverage tests for lexer/mod.rs uncovered paths

use metorex::lexer::{Lexer, TokenKind};

// ── lexer/mod.rs line 48: advance returns None at EOF ───────────────────────

#[test]
fn lexer_empty_input_returns_eof() {
    let mut lexer = Lexer::new("");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::EOF);
}

// ── lexer/mod.rs line 667: unknown character returns EOF ────────────────────

#[test]
fn lexer_unknown_character_returns_eof() {
    let mut lexer = Lexer::new("\u{0007}"); // BEL control character
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::EOF);
}

#[test]
fn lexer_tilde_is_unknown() {
    let mut lexer = Lexer::new("~");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::EOF);
}

#[test]
fn lexer_backtick_is_unknown() {
    let mut lexer = Lexer::new("`");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::EOF);
}

// ── lexer/mod.rs: tokenize method (line 453-464) ───────────────────────────

#[test]
fn lexer_tokenize_collects_all_tokens() {
    let lexer = Lexer::new("x = 1 + 2");
    let tokens = lexer.tokenize();
    // Should have: Ident(x), Equal, Int(1), Plus, Int(2), EOF
    assert_eq!(tokens.len(), 6);
    assert_eq!(tokens.last().unwrap().kind, TokenKind::EOF);
}

#[test]
fn lexer_tokenize_empty_source() {
    let lexer = Lexer::new("");
    let tokens = lexer.tokenize();
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].kind, TokenKind::EOF);
}

// ── lexer/mod.rs: peek_token (line 432-450) ────────────────────────────────

#[test]
fn lexer_peek_token_does_not_consume() {
    let mut lexer = Lexer::new("42");
    let peeked = lexer.peek_token();
    assert_eq!(peeked.kind, TokenKind::Int(42));
    // Should still get the same token on next_token
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::Int(42));
}

#[test]
fn lexer_peek_token_multiple_times() {
    let mut lexer = Lexer::new("hello");
    let first = lexer.peek_token();
    let second = lexer.peek_token();
    assert_eq!(first.kind, second.kind);
}

// ── lexer/mod.rs: Iterator implementation ───────────────────────────────────

#[test]
fn lexer_iterator_collects_tokens() {
    let lexer = Lexer::new("1 + 2");
    let tokens: Vec<_> = lexer.collect();
    // Int(1), Plus, Int(2) - Iterator stops before EOF
    assert_eq!(tokens.len(), 3);
}

#[test]
fn lexer_iterator_empty_source() {
    let lexer = Lexer::new("");
    let tokens: Vec<_> = lexer.collect();
    assert_eq!(tokens.len(), 0);
}

// ── lexer/mod.rs line 309: escape at EOF in string ──────────────────────────

#[test]
fn lexer_unterminated_single_quote_string() {
    let mut lexer = Lexer::new("'hello");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::EOF);
}

#[test]
fn lexer_newline_in_single_quote_string() {
    let mut lexer = Lexer::new("'hello\nworld'");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::EOF);
}

// ── lexer/mod.rs: hash not followed by brace in double string ───────────────

#[test]
fn lexer_hash_not_interpolation() {
    let mut lexer = Lexer::new("\"hello # world\"");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::String("hello # world".to_string()));
}

// ── lexer/mod.rs: class variable (@@var) ────────────────────────────────────

#[test]
fn lexer_class_variable() {
    let mut lexer = Lexer::new("@@count");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::ClassVar("count".to_string()));
}

// ── lexer/mod.rs: global variable ($var) ────────────────────────────────────

#[test]
fn lexer_global_variable() {
    let mut lexer = Lexer::new("$stdout");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::GlobalVar("stdout".to_string()));
}

// ── lexer/mod.rs: identifier with ? and ! ───────────────────────────────────

#[test]
fn lexer_identifier_with_question_mark() {
    let mut lexer = Lexer::new("empty?");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::Ident("empty?".to_string()));
}

#[test]
fn lexer_identifier_with_bang() {
    let mut lexer = Lexer::new("save!");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::Ident("save!".to_string()));
}

// ── lexer/mod.rs: whitespace with tabs and carriage returns ─────────────────

#[test]
fn lexer_skips_tabs_and_carriage_returns() {
    let mut lexer = Lexer::new("\t\r42");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::Int(42));
}

// ── lexer/mod.rs: newline token ─────────────────────────────────────────────

#[test]
fn lexer_newline_token() {
    let mut lexer = Lexer::new("\n");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::Newline);
}

// ── lexer/mod.rs: comment token ─────────────────────────────────────────────

#[test]
fn lexer_comment_token() {
    let mut lexer = Lexer::new("# this is a comment");
    let token = lexer.next_token();
    assert!(matches!(token.kind, TokenKind::Comment(_)));
}

// ── lexer/mod.rs: number with trailing dot (method call, not float) ─────────

#[test]
fn lexer_number_followed_by_dot_method() {
    let mut lexer = Lexer::new("42.to_s");
    let t1 = lexer.next_token();
    assert_eq!(t1.kind, TokenKind::Int(42));
    let t2 = lexer.next_token();
    assert_eq!(t2.kind, TokenKind::Dot);
    let t3 = lexer.next_token();
    assert_eq!(t3.kind, TokenKind::Ident("to_s".to_string()));
}

// ── lexer/mod.rs: number with dot at EOF ────────────────────────────────────

#[test]
fn lexer_number_dot_at_eof() {
    let mut lexer = Lexer::new("42.");
    let t1 = lexer.next_token();
    assert_eq!(t1.kind, TokenKind::Int(42));
    let t2 = lexer.next_token();
    assert_eq!(t2.kind, TokenKind::Dot);
}

// ── lexer/mod.rs: all keywords ──────────────────────────────────────────────

#[test]
fn lexer_all_keywords() {
    let keywords = vec![
        ("def", TokenKind::Def),
        ("class", TokenKind::Class),
        ("if", TokenKind::If),
        ("elsif", TokenKind::Elsif),
        ("else", TokenKind::Else),
        ("unless", TokenKind::Unless),
        ("while", TokenKind::While),
        ("for", TokenKind::For),
        ("in", TokenKind::In),
        ("end", TokenKind::End),
        ("do", TokenKind::Do),
        ("begin", TokenKind::Begin),
        ("rescue", TokenKind::Rescue),
        ("ensure", TokenKind::Ensure),
        ("raise", TokenKind::Raise),
        ("break", TokenKind::Break),
        ("continue", TokenKind::Continue),
        ("return", TokenKind::Return),
        ("lambda", TokenKind::Lambda),
        ("super", TokenKind::Super),
        ("case", TokenKind::Case),
        ("when", TokenKind::When),
        ("then", TokenKind::Then),
        ("attr_reader", TokenKind::AttrReader),
        ("attr_writer", TokenKind::AttrWriter),
        ("attr_accessor", TokenKind::AttrAccessor),
        ("module", TokenKind::Module),
        ("include", TokenKind::Include),
        ("extend", TokenKind::Extend),
        ("true", TokenKind::True),
        ("false", TokenKind::False),
        ("nil", TokenKind::Nil),
    ];
    for (input, expected) in keywords {
        let mut lexer = Lexer::new(input);
        let token = lexer.next_token();
        assert_eq!(token.kind, expected, "Failed for keyword: {}", input);
    }
}

// ── lexer/mod.rs: position tracking ─────────────────────────────────────────

#[test]
fn lexer_position_tracking_across_newlines() {
    let mut lexer = Lexer::new("a\nb");
    let t1 = lexer.next_token();
    assert_eq!(t1.position.line, 1);
    assert_eq!(t1.position.column, 1); // column where 'a' starts

    let _newline = lexer.next_token(); // newline

    let t2 = lexer.next_token();
    assert_eq!(t2.position.line, 2);
}

// ── lexer: instance variable ────────────────────────────────────────────────

#[test]
fn lexer_instance_variable() {
    let mut lexer = Lexer::new("@name");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::InstanceVar("name".to_string()));
}
