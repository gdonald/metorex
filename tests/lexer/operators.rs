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
