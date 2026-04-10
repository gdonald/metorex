// Operator and delimiter tests

use metorex::lexer::{Lexer, TokenKind};

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
    // After an identifier, / is division not regex
    let tokens = Lexer::new("a / b").tokenize();
    assert_eq!(tokens[1].kind, TokenKind::Slash);
}

#[test]
fn test_lexer_regex_literal() {
    // At start of input, / begins a regex literal
    let tokens = Lexer::new("/pattern/").tokenize();
    assert_eq!(
        tokens[0].kind,
        TokenKind::Regex("pattern".to_string(), "".to_string())
    );
}

#[test]
fn test_lexer_regex_literal_with_flags() {
    let tokens = Lexer::new("/hello/im").tokenize();
    assert_eq!(
        tokens[0].kind,
        TokenKind::Regex("hello".to_string(), "im".to_string())
    );
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
    // After an identifier, /= is division-assignment
    let tokens = Lexer::new("a /= b").tokenize();
    assert_eq!(tokens[1].kind, TokenKind::SlashEqual);
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

// ===== Bang Operator Tests =====

#[test]
fn test_lexer_operator_bang() {
    let mut lexer = Lexer::new("!");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::Bang);
}

#[test]
fn test_lexer_operator_bang_not_bang_equal() {
    let mut lexer = Lexer::new("!");
    let token = lexer.next_token();
    assert_ne!(token.kind, TokenKind::BangEqual);
}

#[test]
fn test_lexer_bang_expression() {
    let mut lexer = Lexer::new("!x");

    let token1 = lexer.next_token();
    assert_eq!(token1.kind, TokenKind::Bang);

    let token2 = lexer.next_token();
    assert_eq!(token2.kind, TokenKind::Ident("x".to_string()));
}

// ===== Logical Operator Tests =====

#[test]
fn test_lexer_operator_logical_and() {
    let mut lexer = Lexer::new("&&");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::LogicalAnd);
}

#[test]
fn test_lexer_operator_logical_or() {
    let mut lexer = Lexer::new("||");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::LogicalOr);
}

#[test]
fn test_lexer_operator_colon_colon() {
    let mut lexer = Lexer::new("::");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::ColonColon);
}

#[test]
fn test_lexer_single_ampersand_not_logical_and() {
    let mut lexer = Lexer::new("&");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::Ampersand);
}

#[test]
fn test_lexer_single_pipe_not_logical_or() {
    let mut lexer = Lexer::new("|");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::Pipe);
}

#[test]
fn test_lexer_single_colon_not_colon_colon() {
    let mut lexer = Lexer::new(":");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::Colon);
}

#[test]
fn test_lexer_logical_and_expression() {
    let mut lexer = Lexer::new("x && y");

    let token1 = lexer.next_token();
    assert_eq!(token1.kind, TokenKind::Ident("x".to_string()));

    let token2 = lexer.next_token();
    assert_eq!(token2.kind, TokenKind::LogicalAnd);

    let token3 = lexer.next_token();
    assert_eq!(token3.kind, TokenKind::Ident("y".to_string()));
}

#[test]
fn test_lexer_logical_or_expression() {
    let mut lexer = Lexer::new("x || y");

    let token1 = lexer.next_token();
    assert_eq!(token1.kind, TokenKind::Ident("x".to_string()));

    let token2 = lexer.next_token();
    assert_eq!(token2.kind, TokenKind::LogicalOr);

    let token3 = lexer.next_token();
    assert_eq!(token3.kind, TokenKind::Ident("y".to_string()));
}

#[test]
fn test_lexer_scope_resolution_expression() {
    let mut lexer = Lexer::new("Math::PI");

    let token1 = lexer.next_token();
    assert_eq!(token1.kind, TokenKind::Ident("Math".to_string()));

    let token2 = lexer.next_token();
    assert_eq!(token2.kind, TokenKind::ColonColon);

    let token3 = lexer.next_token();
    assert_eq!(token3.kind, TokenKind::Ident("PI".to_string()));
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
