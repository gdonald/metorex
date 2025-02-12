// Identifier, keyword, and variable tests

use metorex::lexer::{Lexer, TokenKind};

// ===== Identifier Tests =====

#[test]
fn test_lexer_simple_identifier() {
    let mut lexer = Lexer::new("foo");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::Ident("foo".to_string()));
}

#[test]
fn test_lexer_identifier_with_underscores() {
    let mut lexer = Lexer::new("foo_bar_baz");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::Ident("foo_bar_baz".to_string()));
}

#[test]
fn test_lexer_identifier_starting_with_underscore() {
    let mut lexer = Lexer::new("_private");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::Ident("_private".to_string()));
}

#[test]
fn test_lexer_identifier_with_numbers() {
    let mut lexer = Lexer::new("var123");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::Ident("var123".to_string()));
}

#[test]
fn test_lexer_identifier_all_underscores() {
    let mut lexer = Lexer::new("___");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::Ident("___".to_string()));
}

#[test]
fn test_lexer_single_letter_identifier() {
    let mut lexer = Lexer::new("x");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::Ident("x".to_string()));
}

#[test]
fn test_lexer_multiple_identifiers() {
    let mut lexer = Lexer::new("foo bar baz");

    let token1 = lexer.next_token();
    assert_eq!(token1.kind, TokenKind::Ident("foo".to_string()));

    let token2 = lexer.next_token();
    assert_eq!(token2.kind, TokenKind::Ident("bar".to_string()));

    let token3 = lexer.next_token();
    assert_eq!(token3.kind, TokenKind::Ident("baz".to_string()));
}

#[test]
fn test_lexer_uppercase_identifier() {
    let mut lexer = Lexer::new("FOO");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::Ident("FOO".to_string()));
}

#[test]
fn test_lexer_mixed_case_identifier() {
    let mut lexer = Lexer::new("FooBar");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::Ident("FooBar".to_string()));
}

// ===== Keyword Tests =====

#[test]
fn test_lexer_keyword_def() {
    let mut lexer = Lexer::new("def");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::Def);
}

#[test]
fn test_lexer_keyword_class() {
    let mut lexer = Lexer::new("class");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::Class);
}

#[test]
fn test_lexer_keyword_if() {
    let mut lexer = Lexer::new("if");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::If);
}

#[test]
fn test_lexer_keyword_else() {
    let mut lexer = Lexer::new("else");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::Else);
}

#[test]
fn test_lexer_keyword_while() {
    let mut lexer = Lexer::new("while");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::While);
}

#[test]
fn test_lexer_keyword_end() {
    let mut lexer = Lexer::new("end");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::End);
}

#[test]
fn test_lexer_keyword_do() {
    let mut lexer = Lexer::new("do");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::Do);
}

#[test]
fn test_lexer_keyword_true() {
    let mut lexer = Lexer::new("true");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::True);
}

#[test]
fn test_lexer_keyword_false() {
    let mut lexer = Lexer::new("false");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::False);
}

#[test]
fn test_lexer_keyword_nil() {
    let mut lexer = Lexer::new("nil");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::Nil);
}

#[test]
fn test_lexer_keyword_not_partial_match() {
    let mut lexer = Lexer::new("definition");
    let token = lexer.next_token();
    // Should be identifier, not keyword
    assert_eq!(token.kind, TokenKind::Ident("definition".to_string()));
}

#[test]
fn test_lexer_keyword_with_suffix() {
    let mut lexer = Lexer::new("class_name");
    let token = lexer.next_token();
    // Should be identifier, not keyword
    assert_eq!(token.kind, TokenKind::Ident("class_name".to_string()));
}

#[test]
fn test_lexer_keyword_case_sensitive() {
    let mut lexer = Lexer::new("Class");
    let token = lexer.next_token();
    // Keywords are case-sensitive, so this should be an identifier
    assert_eq!(token.kind, TokenKind::Ident("Class".to_string()));
}

#[test]
fn test_lexer_multiple_keywords() {
    let mut lexer = Lexer::new("if else while end");

    let token1 = lexer.next_token();
    assert_eq!(token1.kind, TokenKind::If);

    let token2 = lexer.next_token();
    assert_eq!(token2.kind, TokenKind::Else);

    let token3 = lexer.next_token();
    assert_eq!(token3.kind, TokenKind::While);

    let token4 = lexer.next_token();
    assert_eq!(token4.kind, TokenKind::End);
}

// ===== Instance Variable Tests =====

#[test]
fn test_lexer_instance_variable() {
    let mut lexer = Lexer::new("@name");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::InstanceVar("name".to_string()));
}

#[test]
fn test_lexer_instance_variable_with_underscores() {
    let mut lexer = Lexer::new("@_private_var");
    let token = lexer.next_token();
    assert_eq!(
        token.kind,
        TokenKind::InstanceVar("_private_var".to_string())
    );
}

#[test]
fn test_lexer_instance_variable_with_numbers() {
    let mut lexer = Lexer::new("@value123");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::InstanceVar("value123".to_string()));
}

#[test]
fn test_lexer_instance_variable_single_char() {
    let mut lexer = Lexer::new("@x");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::InstanceVar("x".to_string()));
}

#[test]
fn test_lexer_multiple_instance_variables() {
    let mut lexer = Lexer::new("@foo @bar @baz");

    let token1 = lexer.next_token();
    assert_eq!(token1.kind, TokenKind::InstanceVar("foo".to_string()));

    let token2 = lexer.next_token();
    assert_eq!(token2.kind, TokenKind::InstanceVar("bar".to_string()));

    let token3 = lexer.next_token();
    assert_eq!(token3.kind, TokenKind::InstanceVar("baz".to_string()));
}

// ===== Class Variable Tests =====

#[test]
fn test_lexer_class_variable() {
    let mut lexer = Lexer::new("@@count");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::ClassVar("count".to_string()));
}

#[test]
fn test_lexer_class_variable_with_underscores() {
    let mut lexer = Lexer::new("@@_shared_state");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::ClassVar("_shared_state".to_string()));
}

#[test]
fn test_lexer_class_variable_with_numbers() {
    let mut lexer = Lexer::new("@@version2");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::ClassVar("version2".to_string()));
}

#[test]
fn test_lexer_class_variable_single_char() {
    let mut lexer = Lexer::new("@@x");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::ClassVar("x".to_string()));
}

#[test]
fn test_lexer_multiple_class_variables() {
    let mut lexer = Lexer::new("@@foo @@bar @@baz");

    let token1 = lexer.next_token();
    assert_eq!(token1.kind, TokenKind::ClassVar("foo".to_string()));

    let token2 = lexer.next_token();
    assert_eq!(token2.kind, TokenKind::ClassVar("bar".to_string()));

    let token3 = lexer.next_token();
    assert_eq!(token3.kind, TokenKind::ClassVar("baz".to_string()));
}

// ===== Mixed Identifier, Keyword, and Variable Tests =====

#[test]
fn test_lexer_identifier_and_keyword() {
    let mut lexer = Lexer::new("var if name");

    let token1 = lexer.next_token();
    assert_eq!(token1.kind, TokenKind::Ident("var".to_string()));

    let token2 = lexer.next_token();
    assert_eq!(token2.kind, TokenKind::If);

    let token3 = lexer.next_token();
    assert_eq!(token3.kind, TokenKind::Ident("name".to_string()));
}

#[test]
fn test_lexer_all_variable_types() {
    let mut lexer = Lexer::new("local @instance @@class");

    let token1 = lexer.next_token();
    assert_eq!(token1.kind, TokenKind::Ident("local".to_string()));

    let token2 = lexer.next_token();
    assert_eq!(token2.kind, TokenKind::InstanceVar("instance".to_string()));

    let token3 = lexer.next_token();
    assert_eq!(token3.kind, TokenKind::ClassVar("class".to_string()));
}

#[test]
fn test_lexer_keywords_and_variables() {
    let mut lexer = Lexer::new("def @name class @@count end");

    let token1 = lexer.next_token();
    assert_eq!(token1.kind, TokenKind::Def);

    let token2 = lexer.next_token();
    assert_eq!(token2.kind, TokenKind::InstanceVar("name".to_string()));

    let token3 = lexer.next_token();
    assert_eq!(token3.kind, TokenKind::Class);

    let token4 = lexer.next_token();
    assert_eq!(token4.kind, TokenKind::ClassVar("count".to_string()));

    let token5 = lexer.next_token();
    assert_eq!(token5.kind, TokenKind::End);
}

#[test]
fn test_lexer_identifier_with_keyword_substring() {
    let mut lexer = Lexer::new("ifdef classify endgame");

    let token1 = lexer.next_token();
    assert_eq!(token1.kind, TokenKind::Ident("ifdef".to_string()));

    let token2 = lexer.next_token();
    assert_eq!(token2.kind, TokenKind::Ident("classify".to_string()));

    let token3 = lexer.next_token();
    assert_eq!(token3.kind, TokenKind::Ident("endgame".to_string()));
}

#[test]
fn test_lexer_empty_instance_variable() {
    let mut lexer = Lexer::new("@ ");
    let token = lexer.next_token();
    // Instance variable with empty name
    assert_eq!(token.kind, TokenKind::InstanceVar("".to_string()));
}

#[test]
fn test_lexer_empty_class_variable() {
    let mut lexer = Lexer::new("@@ ");
    let token = lexer.next_token();
    // Class variable with empty name
    assert_eq!(token.kind, TokenKind::ClassVar("".to_string()));
}
