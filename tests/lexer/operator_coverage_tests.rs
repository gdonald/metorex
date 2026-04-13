// Operator coverage tests — compound assignment coverage, comparison,
// delimiter coverage, and new token types.

use metorex::lexer::{Lexer, TokenKind};

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
    let tokens = Lexer::new("a + b - c * d / e % f").tokenize();
    // tokens: a + b - c * d / e % f EOF
    assert_eq!(tokens[1].kind, TokenKind::Plus);
    assert_eq!(tokens[3].kind, TokenKind::Minus);
    assert_eq!(tokens[5].kind, TokenKind::Star);
    assert_eq!(tokens[7].kind, TokenKind::Slash);
    assert_eq!(tokens[9].kind, TokenKind::Percent);
}

#[test]
fn test_lexer_all_compound_assignments() {
    let tokens = Lexer::new("a += 1\nb -= 1\nc *= 1\nd /= 1").tokenize();
    // a += 1 \n b -= 1 \n c *= 1 \n d /= 1
    assert_eq!(tokens[1].kind, TokenKind::PlusEqual);
    assert_eq!(tokens[5].kind, TokenKind::MinusEqual);
    assert_eq!(tokens[9].kind, TokenKind::StarEqual);
    assert_eq!(tokens[13].kind, TokenKind::SlashEqual);
}

// ── Compound assignment operators (lexer/mod.rs lines 509-547) ─────────

#[test]
fn test_lexer_plus_equal() {
    let mut lexer = Lexer::new("+=");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::PlusEqual);
}

#[test]
fn test_lexer_minus_equal() {
    let mut lexer = Lexer::new("-=");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::MinusEqual);
}

#[test]
fn test_lexer_star_equal() {
    let mut lexer = Lexer::new("*=");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::StarEqual);
}

#[test]
fn test_lexer_slash_equal() {
    // After an identifier, /= is division-assignment
    let tokens = Lexer::new("x /= 2").tokenize();
    assert_eq!(tokens[1].kind, TokenKind::SlashEqual);
}

#[test]
fn test_lexer_arrow() {
    let mut lexer = Lexer::new("->");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::Arrow);
}

// ── Comparison operators (lexer/mod.rs lines 552-590) ──────────────────

#[test]
fn test_lexer_equal_equal() {
    let mut lexer = Lexer::new("==");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::EqualEqual);
}

#[test]
fn test_lexer_fat_arrow() {
    let mut lexer = Lexer::new("=>");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::FatArrow);
}

#[test]
fn test_lexer_bang_equal() {
    let mut lexer = Lexer::new("!=");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::BangEqual);
}

#[test]
fn test_lexer_bang_standalone() {
    let mut lexer = Lexer::new("!");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::Bang);
}

#[test]
fn test_lexer_less_equal() {
    let mut lexer = Lexer::new("<=");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::LessEqual);
}

#[test]
fn test_lexer_greater_equal() {
    let mut lexer = Lexer::new(">=");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::GreaterEqual);
}

// ── Delimiters (lexer/mod.rs lines 592-620) ────────────────────────────

#[test]
fn test_lexer_lbrace() {
    let mut lexer = Lexer::new("{");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::LBrace);
}

#[test]
fn test_lexer_rbrace() {
    let mut lexer = Lexer::new("}");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::RBrace);
}

#[test]
fn test_lexer_lbracket() {
    let mut lexer = Lexer::new("[");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::LBracket);
}

#[test]
fn test_lexer_rbracket() {
    let mut lexer = Lexer::new("]");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::RBracket);
}

#[test]
fn test_lexer_comma() {
    let mut lexer = Lexer::new(",");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::Comma);
}

// ── Dot, range operators (lexer/mod.rs lines 620-634) ──────────────────

#[test]
fn test_lexer_dot() {
    let mut lexer = Lexer::new(".");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::Dot);
}

#[test]
fn test_lexer_dot_dot() {
    let mut lexer = Lexer::new("..");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::DotDot);
}

#[test]
fn test_lexer_dot_dot_dot() {
    let mut lexer = Lexer::new("...");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::DotDotDot);
}

// ── Colon, scope resolution (lexer/mod.rs lines 636-643) ───────────────

#[test]
fn test_lexer_colon() {
    let mut lexer = Lexer::new(":");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::Colon);
}

#[test]
fn test_lexer_colon_colon() {
    let mut lexer = Lexer::new("::");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::ColonColon);
}

// ── Semicolon (lexer/mod.rs line 645) ──────────────────────────────────

#[test]
fn test_lexer_semicolon() {
    let mut lexer = Lexer::new(";");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::Semicolon);
}

// ── Pipe, logical operators (lexer/mod.rs lines 649-665) ───────────────

#[test]
fn test_lexer_pipe() {
    let mut lexer = Lexer::new("|");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::Pipe);
}

#[test]
fn test_lexer_logical_or() {
    let mut lexer = Lexer::new("||");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::LogicalOr);
}

#[test]
fn test_lexer_ampersand() {
    let mut lexer = Lexer::new("&");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::Ampersand);
}

#[test]
fn test_lexer_logical_and() {
    let mut lexer = Lexer::new("&&");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::LogicalAnd);
}

// ── New token types ─────────────────────────────────────────────────────────

#[test]
fn test_lexer_tokens_display_new_types() {
    let tokens = Lexer::new("=~ !~ === ^ ||= &&=").tokenize();
    let displays: Vec<String> = tokens.iter().map(|t| format!("{}", t.kind)).collect();
    assert!(displays.contains(&"=~".to_string()));
    assert!(displays.contains(&"!~".to_string()));
    assert!(displays.contains(&"===".to_string()));
    assert!(displays.contains(&"^".to_string()));
    assert!(displays.contains(&"||=".to_string()));
    assert!(displays.contains(&"&&=".to_string()));
}

// ── Operators added during mspec work ──────────────────────────────────────

#[test]
fn test_lexer_match_op() {
    let mut lexer = Lexer::new("=~");
    assert_eq!(lexer.next_token().kind, TokenKind::Match);
}

#[test]
fn test_lexer_not_match_op() {
    let mut lexer = Lexer::new("!~");
    assert_eq!(lexer.next_token().kind, TokenKind::NotMatch);
}

#[test]
fn test_lexer_triple_equal() {
    let mut lexer = Lexer::new("===");
    assert_eq!(lexer.next_token().kind, TokenKind::TripleEqual);
}

#[test]
fn test_lexer_star_star() {
    let mut lexer = Lexer::new("**");
    assert_eq!(lexer.next_token().kind, TokenKind::StarStar);
}

#[test]
fn test_lexer_star_star_then_ident() {
    let tokens = Lexer::new("**h").tokenize();
    assert_eq!(tokens[0].kind, TokenKind::StarStar);
    assert!(matches!(&tokens[1].kind, TokenKind::Ident(s) if s == "h"));
}

#[test]
fn test_lexer_star_after_number_is_star() {
    // single * should not become **
    let tokens = Lexer::new("3 * 2").tokenize();
    let kinds: Vec<_> = tokens.iter().map(|t| &t.kind).collect();
    assert!(kinds.iter().any(|k| matches!(k, TokenKind::Star)));
}

#[test]
fn test_lexer_caret() {
    let mut lexer = Lexer::new("^");
    assert_eq!(lexer.next_token().kind, TokenKind::Caret);
}

#[test]
fn test_lexer_logical_or_assign() {
    let mut lexer = Lexer::new("||=");
    assert_eq!(lexer.next_token().kind, TokenKind::LogicalOrAssign);
}

#[test]
fn test_lexer_logical_and_assign() {
    let mut lexer = Lexer::new("&&=");
    assert_eq!(lexer.next_token().kind, TokenKind::LogicalAndAssign);
}

#[test]
fn test_lexer_pipe_alone() {
    let mut lexer = Lexer::new("|");
    assert_eq!(lexer.next_token().kind, TokenKind::Pipe);
}

#[test]
fn test_lexer_ampersand_alone() {
    let mut lexer = Lexer::new("&");
    assert_eq!(lexer.next_token().kind, TokenKind::Ampersand);
}

// ── Backtick command strings ──────────────────────────────────────────────

#[test]
fn test_lexer_backtick_command() {
    let tokens = Lexer::new("`ls -la`").tokenize();
    assert!(
        tokens
            .iter()
            .any(|t| matches!(&t.kind, TokenKind::String(s) if s == "ls -la"))
    );
}

#[test]
fn test_lexer_backtick_empty() {
    let tokens = Lexer::new("``").tokenize();
    assert!(
        tokens
            .iter()
            .any(|t| matches!(&t.kind, TokenKind::String(s) if s.is_empty()))
    );
}

// ── Character literal: ?x ─────────────────────────────────────────────────

#[test]
fn test_lexer_char_literal() {
    let tokens = Lexer::new("?a").tokenize();
    assert!(
        tokens
            .iter()
            .any(|t| matches!(&t.kind, TokenKind::String(s) if s == "a"))
    );
}

#[test]
fn test_lexer_question_with_space() {
    // ? followed by space is the ternary operator.
    let tokens = Lexer::new("? ").tokenize();
    assert_eq!(tokens[0].kind, TokenKind::Question);
}

// ── Unknown character is consumed and yields EOF ──────────────────────────

#[test]
fn test_lexer_unknown_character() {
    // \u{2603} (snowman) is not a valid identifier or operator char.
    let tokens = Lexer::new("\u{2603}").tokenize();
    assert!(tokens.iter().any(|t| t.kind == TokenKind::EOF));
}
